//! Bearer token authentication middleware for axum.
//!
//! Supports both JWT tokens and GitHub Personal Access Tokens (PATs).
//! PAT tokens (prefixed with `ghp_`, `gho_`, or `github_pat_`) are validated
//! via the GitHub API and cached in-memory with a 5-minute TTL.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::mcp::oauth::OAuthState;
use crate::mcp::oauth::jwt::NsipClaims;

/// TTL for cached PAT validations (5 minutes).
const PAT_CACHE_TTL: Duration = Duration::from_secs(300);

/// In-memory cache for validated GitHub PATs.
///
/// Maps PAT token to `(github_login, validation_time)`.
pub type PatCache = Arc<RwLock<HashMap<String, (String, Instant)>>>;

/// Create a new empty PAT cache.
#[must_use]
pub fn new_pat_cache() -> PatCache {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Returns `true` if the token looks like a GitHub Personal Access Token.
#[must_use]
fn is_github_pat(token: &str) -> bool {
    token.starts_with("ghp_") || token.starts_with("gho_") || token.starts_with("github_pat_")
}

/// Axum middleware that validates `Authorization: Bearer <token>` headers.
///
/// When OAuth is disabled (`None` state), all requests pass through
/// unauthenticated. When enabled, a missing or invalid bearer token results
/// in a `401 Unauthorized` response with a `WWW-Authenticate` header pointing
/// to the resource metadata endpoint.
///
/// Supports two token types:
/// - **JWT tokens**: Validated via HMAC-SHA256 signature verification.
/// - **GitHub PATs**: Tokens starting with `ghp_`, `gho_`, or `github_pat_`
///   are validated by calling the GitHub API and cached with a 5-minute TTL.
///
/// On successful validation, the decoded [`NsipClaims`] are inserted into
/// the request extensions for downstream handlers to access.
///
/// # Errors
///
/// Returns `401 Unauthorized` with a JSON error body when the token is
/// missing or invalid.
pub async fn bearer_auth(
    State(oauth): State<Option<OAuthState>>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let Some(oauth_state) = oauth else {
        // OAuth disabled -- pass through
        return next.run(req).await;
    };

    // OAuth protocol endpoints must be accessible without a bearer token.
    let path = req.uri().path();
    if path.starts_with("/.well-known/")
        || path == "/register"
        || path == "/authorize"
        || path == "/callback"
        || path == "/token"
    {
        return next.run(req).await;
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            tracing::warn!(
                path = %req.uri().path(),
                has_auth = auth_header.is_some(),
                "bearer auth: no Bearer token in Authorization header"
            );
            return unauthorized_response(&oauth_state.config.base_url);
        },
    };

    if is_github_pat(token) {
        // Check PAT cache first
        let cached_login = lookup_pat_cache(&oauth_state.pat_cache, token);

        if let Some(login) = cached_login {
            let claims = pat_claims(&login);
            req.extensions_mut().insert(claims);
            return next.run(req).await;
        }

        // Cache miss -- validate via GitHub API
        match oauth_state.github.get_user(token).await {
            Ok(user) => {
                // Cache the validated PAT
                if let Ok(mut cache) = oauth_state.pat_cache.write() {
                    cache.insert(token.to_owned(), (user.login.clone(), Instant::now()));
                }
                let claims = pat_claims(&user.login);
                req.extensions_mut().insert(claims);
                next.run(req).await
            },
            Err(e) => {
                tracing::warn!(
                    path = %req.uri().path(),
                    error = %e,
                    "bearer auth: GitHub PAT validation failed"
                );
                unauthorized_response(&oauth_state.config.base_url)
            },
        }
    } else {
        // JWT validation
        match oauth_state.jwt.validate(token) {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
                next.run(req).await
            },
            Err(e) => {
                tracing::warn!(
                    path = %req.uri().path(),
                    error = %e,
                    token_len = token.len(),
                    "bearer auth: JWT validation failed"
                );
                unauthorized_response(&oauth_state.config.base_url)
            },
        }
    }
}

/// Look up a PAT in the cache, returning the login if valid and not expired.
fn lookup_pat_cache(cache: &PatCache, token: &str) -> Option<String> {
    let guard = cache.read().ok()?;
    let (login, created) = guard.get(token)?;
    let result = if created.elapsed() < PAT_CACHE_TTL {
        Some(login.clone())
    } else {
        None
    };
    drop(guard);
    result
}

/// Build `NsipClaims` for a PAT-authenticated user.
///
/// Since PATs do not carry JWT metadata, we synthesize claims with
/// the GitHub login as the subject, real timestamps, and a random JTI.
/// The `exp` is set to the PAT cache TTL from now (5 min), matching
/// the cache eviction window.
fn pat_claims(github_login: &str) -> NsipClaims {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    NsipClaims {
        sub: github_login.to_owned(),
        iss: "github-pat".to_owned(),
        aud: "nsip-mcp".to_owned(),
        exp: now + PAT_CACHE_TTL.as_secs(),
        iat: now,
        jti: format!("pat-{now}-{github_login}"),
    }
}

/// Build a 401 Unauthorized response with `WWW-Authenticate` header.
fn unauthorized_response(base_url: &str) -> Response {
    let www_auth =
        format!("Bearer resource_metadata=\"{base_url}/.well-known/oauth-protected-resource\"");
    let body = serde_json::json!({
        "error": "invalid_token",
        "error_description": "missing or invalid bearer token",
    });
    (
        axum::http::StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, www_auth)],
        axum::Json(body),
    )
        .into_response()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_response_has_www_authenticate() {
        let resp = unauthorized_response("https://example.com");
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
        assert!(resp.headers().contains_key(header::WWW_AUTHENTICATE));
        let www_auth = resp.headers().get(header::WWW_AUTHENTICATE).unwrap();
        let value = www_auth.to_str().unwrap();
        assert!(value.contains("resource_metadata="));
        assert!(value.contains("example.com"));
    }

    #[test]
    fn is_github_pat_detects_ghp_prefix() {
        assert!(is_github_pat("ghp_abc123def456"));
    }

    #[test]
    fn is_github_pat_detects_gho_prefix() {
        assert!(is_github_pat("gho_tokenvalue"));
    }

    #[test]
    fn is_github_pat_detects_github_pat_prefix() {
        assert!(is_github_pat("github_pat_11AAA_something"));
    }

    #[test]
    fn is_github_pat_rejects_jwt() {
        assert!(!is_github_pat("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.xxx"));
    }

    #[test]
    fn is_github_pat_rejects_empty() {
        assert!(!is_github_pat(""));
    }

    #[test]
    fn pat_claims_sets_subject() {
        let claims = pat_claims("testuser");
        assert_eq!(claims.sub, "testuser");
        assert_eq!(claims.iss, "github-pat");
        assert_eq!(claims.aud, "nsip-mcp");
        assert!(claims.iat > 0, "iat should be a real timestamp");
        assert!(claims.exp > claims.iat, "exp should be after iat");
        assert!(
            claims.jti.starts_with("pat-"),
            "jti should have pat- prefix, got: {}",
            claims.jti
        );
        assert!(
            claims.jti.contains("testuser"),
            "jti should contain the login"
        );
    }

    #[test]
    fn pat_cache_creation() {
        let cache = new_pat_cache();
        assert!(cache.read().unwrap().is_empty());
    }

    #[test]
    fn pat_cache_insert_and_retrieve() {
        let cache = new_pat_cache();
        cache
            .write()
            .unwrap()
            .insert("ghp_test".to_owned(), ("alice".to_owned(), Instant::now()));
        let result = lookup_pat_cache(&cache, "ghp_test");
        assert_eq!(result.as_deref(), Some("alice"));
    }

    use std::future::Future;
    use std::pin::Pin;

    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware as axum_middleware;
    use axum::routing::get;
    use tower::ServiceExt as _;

    use crate::mcp::oauth::OAuthState;
    use crate::mcp::oauth::config::OAuthConfig;
    use crate::mcp::oauth::error::OAuthError;
    use crate::mcp::oauth::github::{GitHubApi, GitHubUser};
    use crate::mcp::oauth::jwt::JwtManager;
    use crate::mcp::oauth::store::{InMemoryOAuthStore, OAuthStoreBackend};

    type TestBoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    struct MockGitHub {
        user: GitHubUser,
    }

    impl GitHubApi for MockGitHub {
        fn exchange_code(&self, _code: &str) -> TestBoxFut<'_, Result<String, OAuthError>> {
            Box::pin(async { Ok("mock-token".into()) })
        }
        fn get_user(&self, _token: &str) -> TestBoxFut<'_, Result<GitHubUser, OAuthError>> {
            let user = self.user.clone();
            Box::pin(async move { Ok(user) })
        }
    }

    struct MockGitHubError;
    impl GitHubApi for MockGitHubError {
        fn exchange_code(&self, _code: &str) -> TestBoxFut<'_, Result<String, OAuthError>> {
            Box::pin(async { Err(OAuthError::ServerError("fail".into())) })
        }
        fn get_user(&self, _token: &str) -> TestBoxFut<'_, Result<GitHubUser, OAuthError>> {
            Box::pin(async { Err(OAuthError::ServerError("fail".into())) })
        }
    }

    fn make_oauth_state(github: Arc<dyn GitHubApi>) -> OAuthState {
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
        OAuthState {
            config: Arc::new(config),
            store,
            jwt,
            github,
            pat_cache: new_pat_cache(),
        }
    }

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn bearer_app(oauth_state: Option<OAuthState>) -> Router {
        Router::new()
            .route("/mcp", get(ok_handler))
            .route("/register", get(ok_handler))
            .layer(axum_middleware::from_fn_with_state(
                oauth_state,
                bearer_auth,
            ))
    }

    #[tokio::test]
    async fn bearer_auth_none_state_passes_through() {
        let app = bearer_app(None);
        let resp = app
            .oneshot(Request::get("/mcp").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_oauth_endpoints_bypass() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        // /register should bypass auth
        let resp = app
            .oneshot(Request::get("/register").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_missing_token_returns_401() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(Request::get("/mcp").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn bearer_auth_valid_jwt_passes() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let token = state.jwt.issue("testuser").unwrap();
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_invalid_jwt_returns_401() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Bearer invalid-jwt-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn bearer_auth_valid_pat_passes() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "patuser".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Bearer ghp_validtoken123456789")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_invalid_pat_returns_401() {
        let github = Arc::new(MockGitHubError) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Bearer ghp_badtoken")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn bearer_auth_pat_cache_hit() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "cached".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        // Pre-populate cache
        state.pat_cache.write().unwrap().insert(
            "ghp_cachedtoken".to_owned(),
            ("cached-user".to_owned(), Instant::now()),
        );
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Bearer ghp_cachedtoken")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn lookup_pat_cache_expired_returns_none() {
        let cache = new_pat_cache();
        // Insert with an instant far enough in the past to exceed PAT_CACHE_TTL
        // (300s). Use checked_sub because on Windows, Instant is relative to
        // boot time and freshly provisioned CI runners may have < 600s uptime.
        let past = Instant::now().checked_sub(Duration::from_secs(600));
        if let Some(expired_at) = past {
            cache
                .write()
                .unwrap()
                .insert("ghp_expired".to_owned(), ("user".to_owned(), expired_at));
            let result = lookup_pat_cache(&cache, "ghp_expired");
            assert!(result.is_none(), "expired PAT should not be found");
        }
        // On platforms where checked_sub returns None, the test is a no-op —
        // the missing-entry and fresh-entry paths are covered by other tests.
    }

    #[test]
    fn lookup_pat_cache_missing_returns_none() {
        let cache = new_pat_cache();
        let result = lookup_pat_cache(&cache, "ghp_nothere");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn bearer_auth_well_known_endpoint_bypasses() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = Router::new()
            .route("/.well-known/oauth-authorization-server", get(ok_handler))
            .route("/.well-known/oauth-protected-resource", get(ok_handler))
            .layer(axum_middleware::from_fn_with_state(
                Some(state),
                bearer_auth,
            ));
        let resp = app
            .oneshot(
                Request::get("/.well-known/oauth-authorization-server")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_authorize_endpoint_bypasses() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = Router::new().route("/authorize", get(ok_handler)).layer(
            axum_middleware::from_fn_with_state(Some(state), bearer_auth),
        );
        let resp = app
            .oneshot(Request::get("/authorize").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_callback_endpoint_bypasses() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = Router::new().route("/callback", get(ok_handler)).layer(
            axum_middleware::from_fn_with_state(Some(state), bearer_auth),
        );
        let resp = app
            .oneshot(Request::get("/callback").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_token_endpoint_bypasses() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = Router::new().route("/token", get(ok_handler)).layer(
            axum_middleware::from_fn_with_state(Some(state), bearer_auth),
        );
        let resp = app
            .oneshot(Request::get("/token").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_basic_auth_header_returns_401() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "u".into(),
                id: 1,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Basic dXNlcjpwYXNz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn bearer_auth_gho_pat_passes() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "ghouser".into(),
                id: 2,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Bearer gho_validtoken123456789")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_github_pat_prefix_passes() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "ghpatuser".into(),
                id: 3,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let app = bearer_app(Some(state));
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Bearer github_pat_11AAA_something")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_auth_pat_populates_cache() {
        let github = Arc::new(MockGitHub {
            user: GitHubUser {
                login: "cacheuser".into(),
                id: 10,
                name: None,
            },
        }) as Arc<dyn GitHubApi>;
        let state = make_oauth_state(github);
        let pat_cache = state.pat_cache.clone();
        let app = bearer_app(Some(state));
        // First request should cache the PAT
        let resp = app
            .oneshot(
                Request::get("/mcp")
                    .header("Authorization", "Bearer ghp_newtoken999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // Verify the PAT was cached
        let login = pat_cache
            .read()
            .unwrap()
            .get("ghp_newtoken999")
            .map(|(l, _)| l.clone());
        assert_eq!(login.as_deref(), Some("cacheuser"));
    }
}
