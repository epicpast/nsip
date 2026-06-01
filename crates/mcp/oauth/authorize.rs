//! OAuth 2.0 authorization endpoint.

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use chrono::Utc;
use serde::Deserialize;

use crate::mcp::oauth::OAuthState;
use crate::mcp::oauth::error::OAuthError;
use crate::mcp::oauth::store::PendingAuth;

/// Query parameters for the authorization endpoint.
#[derive(Debug, Deserialize)]
pub struct AuthorizeParams {
    /// The registered client identifier.
    pub client_id: String,
    /// Must be `"code"`.
    pub response_type: String,
    /// Client's redirect URI (must match a registered URI).
    pub redirect_uri: String,
    /// PKCE code challenge.
    pub code_challenge: String,
    /// Must be `"S256"`.
    pub code_challenge_method: String,
    /// Client-provided opaque state.
    pub state: String,
}

/// GET `/authorize` -- initiate the OAuth 2.0 authorization code flow.
///
/// Validates the client, stores a pending authorization, and redirects the
/// user-agent to GitHub for authentication.
///
/// # Errors
///
/// Returns [`OAuthError::InvalidRequest`] for missing or invalid parameters,
/// [`OAuthError::InvalidClient`] if the client is not registered or the
/// redirect URI does not match.
pub async fn authorize(
    State(state): State<OAuthState>,
    Query(params): Query<AuthorizeParams>,
) -> Result<Response, OAuthError> {
    // Validate response_type
    if params.response_type != "code" {
        return Err(OAuthError::InvalidRequest(
            "response_type must be 'code'".into(),
        ));
    }

    // Validate code_challenge_method
    if params.code_challenge_method != "S256" {
        return Err(OAuthError::InvalidRequest(
            "code_challenge_method must be 'S256'".into(),
        ));
    }

    // Look up and validate the client
    let client = state
        .store
        .get_client(&params.client_id)
        .await
        .map_err(OAuthError::ServerError)?
        .ok_or_else(|| OAuthError::InvalidClient("unknown client_id".into()))?;

    if !redirect_uri_matches(&client.redirect_uris, &params.redirect_uri) {
        return Err(OAuthError::InvalidRequest(
            "redirect_uri does not match registered URIs".into(),
        ));
    }

    // Generate an internal state token to correlate the GitHub callback
    let internal_state = uuid::Uuid::new_v4().to_string();

    let pending = PendingAuth {
        client_id: params.client_id,
        redirect_uri: params.redirect_uri,
        code_challenge: params.code_challenge,
        original_state: params.state,
        created_at: Utc::now(),
    };

    state
        .store
        .store_pending(internal_state.clone(), pending)
        .await
        .map_err(OAuthError::ServerError)?;

    // Build the GitHub OAuth authorization URL
    let gh_client_id = &state.config.github_client_id;
    let callback_uri = format!("{}/callback", state.config.base_url);
    let github_url = format!(
        "https://github.com/login/oauth/authorize\
         ?client_id={gh_client_id}\
         &redirect_uri={callback_uri}\
         &state={internal_state}\
         &scope=read:user"
    );

    Ok(Redirect::to(&github_url).into_response())
}

/// Check whether `request_uri` matches any registered redirect URI.
///
/// Per [RFC 8252 Section 7.3](https://tools.ietf.org/html/rfc8252#section-7.3),
/// native apps using loopback redirect URIs (`127.0.0.1` / `[::1]`) may use
/// any port. We therefore compare scheme+host+path for loopback URIs while
/// requiring an exact match for all others.
fn redirect_uri_matches(registered: &[String], request_uri: &str) -> bool {
    if registered.contains(&request_uri.to_owned()) {
        return true;
    }

    // Loopback-aware comparison: strip port from both sides for
    // 127.0.0.1 / [::1].
    let Ok(req) = reqwest::Url::parse(request_uri) else {
        return false;
    };
    let req_host = req.host_str().unwrap_or("");
    let is_loopback = req_host == "127.0.0.1" || req_host == "[::1]" || req_host == "localhost";
    if !is_loopback {
        return false;
    }

    registered.iter().any(|r| {
        let Ok(reg) = reqwest::Url::parse(r) else {
            return false;
        };
        reg.scheme() == req.scheme() && reg.host_str() == Some(req_host) && reg.path() == req.path()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uris(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn exact_match_succeeds() {
        let registered = uris(&["http://localhost:8080/callback"]);
        assert!(redirect_uri_matches(
            &registered,
            "http://localhost:8080/callback"
        ));
    }

    #[test]
    fn non_matching_uri_fails() {
        let registered = uris(&["http://example.com/callback"]);
        assert!(!redirect_uri_matches(
            &registered,
            "http://example.com/other"
        ));
    }

    #[test]
    fn empty_registered_list_fails() {
        assert!(!redirect_uri_matches(&[], "http://localhost/callback"));
    }

    #[test]
    fn loopback_any_port_127_0_0_1() {
        let registered = uris(&["http://127.0.0.1:8080/callback"]);
        assert!(redirect_uri_matches(
            &registered,
            "http://127.0.0.1:9090/callback"
        ));
    }

    #[test]
    fn loopback_any_port_ipv6() {
        let registered = uris(&["http://[::1]:8080/callback"]);
        assert!(redirect_uri_matches(
            &registered,
            "http://[::1]:9999/callback"
        ));
    }

    #[test]
    fn loopback_different_path_fails() {
        let registered = uris(&["http://127.0.0.1:8080/callback"]);
        assert!(!redirect_uri_matches(
            &registered,
            "http://127.0.0.1:9090/other"
        ));
    }

    #[test]
    fn non_loopback_different_port_fails() {
        let registered = uris(&["http://example.com:8080/callback"]);
        assert!(!redirect_uri_matches(
            &registered,
            "http://example.com:9090/callback"
        ));
    }

    #[test]
    fn invalid_request_uri_fails() {
        let registered = uris(&["http://localhost/callback"]);
        assert!(!redirect_uri_matches(&registered, "not-a-url"));
    }

    #[test]
    fn multiple_registered_uris_first_match_wins() {
        let registered = uris(&["http://example.com/cb1", "http://localhost:3000/callback"]);
        assert!(redirect_uri_matches(
            &registered,
            "http://localhost:3000/callback"
        ));
    }

    #[test]
    fn scheme_mismatch_for_loopback_fails() {
        let registered = uris(&["http://127.0.0.1:8080/callback"]);
        assert!(!redirect_uri_matches(
            &registered,
            "https://127.0.0.1:9090/callback"
        ));
    }

    // --- Handler tests ---

    use std::sync::Arc;

    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get as get_route;
    use tower::ServiceExt as _;

    use crate::mcp::oauth::OAuthState;
    use crate::mcp::oauth::config::OAuthConfig;
    use crate::mcp::oauth::store::{InMemoryOAuthStore, OAuthStoreBackend, RegisteredClient};

    fn test_state() -> OAuthState {
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
        OAuthState::new(config, store).expect("build oauth state")
    }

    fn authorize_app(state: OAuthState) -> Router {
        Router::new()
            .route("/authorize", get_route(authorize))
            .with_state(state)
    }

    #[tokio::test]
    async fn authorize_invalid_response_type() {
        let state = test_state();
        let app = authorize_app(state);
        let resp = app
            .oneshot(
                Request::get("/authorize?client_id=c1&response_type=token&redirect_uri=http://localhost/cb&code_challenge=abc&code_challenge_method=S256&state=s1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn authorize_invalid_code_challenge_method() {
        let state = test_state();
        let app = authorize_app(state);
        let resp = app
            .oneshot(
                Request::get("/authorize?client_id=c1&response_type=code&redirect_uri=http://localhost/cb&code_challenge=abc&code_challenge_method=plain&state=s1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn authorize_unknown_client() {
        let state = test_state();
        let app = authorize_app(state);
        let resp = app
            .oneshot(
                Request::get("/authorize?client_id=nonexistent&response_type=code&redirect_uri=http://localhost/cb&code_challenge=abc&code_challenge_method=S256&state=s1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn authorize_redirect_uri_mismatch() {
        let state = test_state();
        // Register a client first
        state
            .store
            .register_client(RegisteredClient {
                client_id: "client-1".into(),
                redirect_uris: vec!["http://localhost:8080/cb".into()],
                created_at: chrono::Utc::now(),
            })
            .await
            .unwrap();

        let app = authorize_app(state);
        let resp = app
            .oneshot(
                Request::get("/authorize?client_id=client-1&response_type=code&redirect_uri=http://evil.com/cb&code_challenge=abc&code_challenge_method=S256&state=s1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn authorize_valid_request_redirects_to_github() {
        let state = test_state();
        state
            .store
            .register_client(RegisteredClient {
                client_id: "client-1".into(),
                redirect_uris: vec!["http://localhost:8080/cb".into()],
                created_at: chrono::Utc::now(),
            })
            .await
            .unwrap();

        let app = authorize_app(state);
        let resp = app
            .oneshot(
                Request::get("/authorize?client_id=client-1&response_type=code&redirect_uri=http://localhost:8080/cb&code_challenge=abc123&code_challenge_method=S256&state=client-state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            resp.status().is_redirection(),
            "expected redirect, got {}",
            resp.status()
        );
        let location = resp.headers().get("location").unwrap().to_str().unwrap();
        assert!(location.contains("github.com/login/oauth/authorize"));
        assert!(location.contains("client_id=gh-id"));
    }
}
