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
