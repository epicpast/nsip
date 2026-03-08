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
/// Since PATs do not carry JWT metadata, we synthesize minimal claims
/// with the GitHub login as the subject.
fn pat_claims(github_login: &str) -> NsipClaims {
    NsipClaims {
        sub: github_login.to_owned(),
        iss: "github-pat".to_owned(),
        aud: "nsip-mcp".to_owned(),
        exp: 0,
        iat: 0,
        jti: String::new(),
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
}
