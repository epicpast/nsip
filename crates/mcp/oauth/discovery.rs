//! OAuth 2.0 discovery endpoint handlers (RFC 8414 and RFC 9728).

use axum::Json;
use axum::extract::State;

use crate::mcp::oauth::OAuthState;

/// GET `/.well-known/oauth-authorization-server` (RFC 8414).
///
/// Returns authorization server metadata including supported endpoints,
/// response types, grant types, and PKCE methods.
///
/// # Errors
///
/// This handler is infallible.
pub async fn authorization_server_metadata(
    State(state): State<OAuthState>,
) -> Json<serde_json::Value> {
    let base = &state.config.base_url;
    Json(serde_json::json!({
        "issuer": state.config.issuer,
        "authorization_endpoint": format!("{base}/authorize"),
        "token_endpoint": format!("{base}/token"),
        "registration_endpoint": format!("{base}/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "token_endpoint_auth_methods_supported": ["none"],
        "code_challenge_methods_supported": ["S256"],
    }))
}

/// GET `/.well-known/oauth-protected-resource` (RFC 9728).
///
/// Returns protected resource metadata describing where to obtain
/// authorization for this resource server.
///
/// # Errors
///
/// This handler is infallible.
pub async fn protected_resource_metadata(
    State(state): State<OAuthState>,
) -> Json<serde_json::Value> {
    let base = &state.config.base_url;
    Json(serde_json::json!({
        "resource": base,
        "authorization_servers": [base],
        "bearer_methods_supported": ["header"],
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt as _;

    use super::*;
    use crate::mcp::oauth::{
        OAuthState,
        config::OAuthConfig,
        store::{InMemoryOAuthStore, OAuthStoreBackend},
    };

    fn test_state() -> OAuthState {
        let config = OAuthConfig {
            github_client_id: "gh-id".into(),
            github_client_secret: "gh-secret".into(),
            base_url: "https://example.com".into(),
            issuer: "https://issuer.example.com".into(),
            auth_secret: "test-secret-key-that-is-long-enough-32".into(),
            token_ttl_secs: 3600,
            allowed_users: None,
        };
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        OAuthState::new(config, store)
    }

    #[tokio::test]
    async fn authorization_server_metadata_returns_endpoints() {
        let state = test_state();
        let app = Router::new()
            .route(
                "/.well-known/oauth-authorization-server",
                get(authorization_server_metadata),
            )
            .with_state(state);
        let resp = app
            .oneshot(
                Request::get("/.well-known/oauth-authorization-server")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["issuer"], "https://issuer.example.com");
        assert_eq!(
            json["authorization_endpoint"],
            "https://example.com/authorize"
        );
        assert_eq!(json["token_endpoint"], "https://example.com/token");
        assert_eq!(
            json["registration_endpoint"],
            "https://example.com/register"
        );
        assert_eq!(
            json["response_types_supported"],
            serde_json::json!(["code"])
        );
        assert_eq!(
            json["code_challenge_methods_supported"],
            serde_json::json!(["S256"])
        );
    }

    #[tokio::test]
    async fn protected_resource_metadata_returns_resource() {
        let state = test_state();
        let app = Router::new()
            .route(
                "/.well-known/oauth-protected-resource",
                get(protected_resource_metadata),
            )
            .with_state(state);
        let resp = app
            .oneshot(
                Request::get("/.well-known/oauth-protected-resource")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["resource"], "https://example.com");
        assert_eq!(
            json["bearer_methods_supported"],
            serde_json::json!(["header"])
        );
    }
}
