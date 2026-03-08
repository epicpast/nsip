//! GitHub OAuth callback handler.

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use chrono::Utc;
use serde::Deserialize;

use crate::mcp::oauth::OAuthState;
use crate::mcp::oauth::error::OAuthError;
use crate::mcp::oauth::store::IssuedAuthCode;

/// Query parameters received from the GitHub OAuth callback.
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    /// Authorization code from GitHub.
    pub code: String,
    /// Internal state token for correlating with the pending authorization.
    pub state: String,
}

/// GET `/callback` -- handle the GitHub OAuth callback.
///
/// Exchanges the GitHub authorization code for an access token, fetches the
/// user profile, optionally checks the allowed-users list, generates an
/// authorization code, and redirects back to the client application.
///
/// # Errors
///
/// Returns [`OAuthError::InvalidGrant`] if the state token is unknown or
/// expired, [`OAuthError::AccessDenied`] if the user is not in the allowlist,
/// or [`OAuthError::ServerError`] for upstream GitHub API failures.
pub async fn callback(
    State(state): State<OAuthState>,
    Query(params): Query<CallbackParams>,
) -> Result<Response, OAuthError> {
    // Look up the pending authorization by internal state
    let pending = state
        .store
        .consume_pending(&params.state)
        .await
        .map_err(OAuthError::ServerError)?
        .ok_or_else(|| OAuthError::InvalidGrant("unknown or expired state".into()))?;

    // Exchange the GitHub code for a GitHub access token
    let gh_token = state.github.exchange_code(&params.code).await?;

    // Fetch the authenticated user's profile
    let user = state.github.get_user(&gh_token).await?;

    // Check allowed_users if configured
    if let Some(ref allowed) = state.config.allowed_users
        && !allowed.contains(&user.login)
    {
        return Err(OAuthError::AccessDenied(format!(
            "user '{}' is not in the allowed list",
            user.login
        )));
    }

    // Generate an authorization code for the client
    let auth_code = uuid::Uuid::new_v4().to_string();
    let issued = IssuedAuthCode {
        code: auth_code.clone(),
        client_id: pending.client_id,
        redirect_uri: pending.redirect_uri.clone(),
        code_challenge: pending.code_challenge,
        github_login: user.login,
        created_at: Utc::now(),
    };

    state
        .store
        .store_auth_code(issued)
        .await
        .map_err(|e| OAuthError::ServerError(format!("failed to store auth code: {e}")))?;

    // Redirect back to the client with the authorization code
    let separator = if pending.redirect_uri.contains('?') {
        '&'
    } else {
        '?'
    };
    let redirect_url = format!(
        "{}{separator}code={auth_code}&state={}",
        pending.redirect_uri, pending.original_state
    );

    Ok(Redirect::to(&redirect_url).into_response())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use tower::ServiceExt as _;

    use crate::mcp::oauth::config::OAuthConfig;
    use crate::mcp::oauth::error::OAuthError;
    use crate::mcp::oauth::github::{GitHubApi, GitHubUser};
    use crate::mcp::oauth::jwt::JwtManager;
    use crate::mcp::oauth::middleware::new_pat_cache;
    use crate::mcp::oauth::store::{InMemoryOAuthStore, OAuthStoreBackend, PendingAuth};

    type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    struct MockGitHub {
        user: GitHubUser,
    }

    impl GitHubApi for MockGitHub {
        fn exchange_code(&self, _code: &str) -> BoxFut<'_, Result<String, OAuthError>> {
            Box::pin(async { Ok("mock-token".into()) })
        }
        fn get_user(&self, _token: &str) -> BoxFut<'_, Result<GitHubUser, OAuthError>> {
            let user = self.user.clone();
            Box::pin(async move { Ok(user) })
        }
    }

    fn test_state_with_mock() -> OAuthState {
        let config = OAuthConfig {
            github_client_id: "gh-id".into(),
            github_client_secret: "gh-secret".into(),
            base_url: "https://example.com".into(),
            issuer: "https://example.com".into(),
            auth_secret: "test-secret-key-that-is-long-enough-32".into(),
            token_ttl_secs: 3600,
            allowed_users: None,
        };
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        let jwt = JwtManager::new(&config.auth_secret, &config.issuer, config.token_ttl_secs);
        let github: Arc<dyn GitHubApi> = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "testuser".into(),
                id: 42,
                name: Some("Test".into()),
            },
        });
        OAuthState {
            config: Arc::new(config),
            store,
            jwt,
            github,
            pat_cache: new_pat_cache(),
        }
    }

    fn callback_app(state: OAuthState) -> Router {
        Router::new()
            .route("/callback", get(callback))
            .with_state(state)
    }

    #[tokio::test]
    async fn callback_unknown_state_returns_error() {
        let state = test_state_with_mock();
        let app = callback_app(state);
        let resp = app
            .oneshot(
                Request::get("/callback?code=gh-code&state=unknown-state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn callback_valid_state_redirects() {
        let state = test_state_with_mock();
        // Seed a pending auth
        state
            .store
            .store_pending(
                "internal-state-123".into(),
                PendingAuth {
                    client_id: "client-1".into(),
                    redirect_uri: "http://localhost:8080/cb".into(),
                    code_challenge: "challenge".into(),
                    original_state: "client-original-state".into(),
                    created_at: Utc::now(),
                },
            )
            .await
            .unwrap();

        let app = callback_app(state);
        let resp = app
            .oneshot(
                Request::get("/callback?code=gh-code-123&state=internal-state-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Should redirect (3xx)
        assert!(
            resp.status().is_redirection(),
            "expected redirect, got {}",
            resp.status()
        );
        let location = resp.headers().get("location").unwrap().to_str().unwrap();
        assert!(location.starts_with("http://localhost:8080/cb"));
        assert!(location.contains("code="));
        assert!(location.contains("state=client-original-state"));
    }

    #[tokio::test]
    async fn callback_with_allowed_users_denies_unlisted() {
        let config = OAuthConfig {
            github_client_id: "gh-id".into(),
            github_client_secret: "gh-secret".into(),
            base_url: "https://example.com".into(),
            issuer: "https://example.com".into(),
            auth_secret: "test-secret-key-that-is-long-enough-32".into(),
            token_ttl_secs: 3600,
            allowed_users: Some(vec!["allowed-user".into()]),
        };
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        let jwt = JwtManager::new(&config.auth_secret, &config.issuer, config.token_ttl_secs);
        let github: Arc<dyn GitHubApi> = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "testuser".into(),
                id: 42,
                name: None,
            },
        });
        let state = OAuthState {
            config: Arc::new(config),
            store: store.clone(),
            jwt,
            github,
            pat_cache: new_pat_cache(),
        };

        store
            .store_pending(
                "state-deny".into(),
                PendingAuth {
                    client_id: "client-1".into(),
                    redirect_uri: "http://localhost/cb".into(),
                    code_challenge: "challenge".into(),
                    original_state: "orig".into(),
                    created_at: Utc::now(),
                },
            )
            .await
            .unwrap();

        let app = callback_app(state);
        let resp = app
            .oneshot(
                Request::get("/callback?code=code&state=state-deny")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[test]
    fn callback_params_deserialize() {
        let json = r#"{"code":"abc","state":"xyz"}"#;
        let params: CallbackParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.code, "abc");
        assert_eq!(params.state, "xyz");
    }
}
