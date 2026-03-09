//! Dynamic client registration endpoint (RFC 7591).

use axum::Json;
use axum::extract::State;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::mcp::oauth::OAuthState;
use crate::mcp::oauth::error::OAuthError;
use crate::mcp::oauth::store::RegisteredClient;

/// Client registration request body.
#[derive(Debug, Deserialize)]
pub struct RegistrationRequest {
    /// Redirect URIs for the client.
    pub redirect_uris: Vec<String>,
    /// Grant types the client intends to use.
    #[serde(default)]
    pub grant_types: Vec<String>,
    /// Human-readable client name.
    #[serde(default)]
    pub client_name: Option<String>,
}

/// Client registration response body.
#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    /// The assigned client identifier.
    pub client_id: String,
    /// Registered redirect URIs.
    pub redirect_uris: Vec<String>,
}

/// POST `/register` -- dynamic client registration.
///
/// Accepts a JSON body with `redirect_uris` and optional `grant_types` and
/// `client_name`. Generates a unique `client_id`, stores the client, and
/// returns the registration details.
///
/// # Errors
///
/// Returns [`OAuthError::InvalidRequest`] if `redirect_uris` is empty or
/// `grant_types` contains unsupported values.
pub async fn register_client(
    State(state): State<OAuthState>,
    Json(body): Json<RegistrationRequest>,
) -> Result<Json<RegistrationResponse>, OAuthError> {
    if body.redirect_uris.is_empty() {
        return Err(OAuthError::InvalidRequest(
            "redirect_uris must not be empty".into(),
        ));
    }

    // Validate grant_types if provided.
    for gt in &body.grant_types {
        if gt != "authorization_code" && gt != "refresh_token" {
            return Err(OAuthError::InvalidRequest(format!(
                "unsupported grant_type: {gt}"
            )));
        }
    }

    let client_id = uuid::Uuid::new_v4().to_string();
    let client = RegisteredClient {
        client_id: client_id.clone(),
        redirect_uris: body.redirect_uris.clone(),
        created_at: Utc::now(),
    };

    state
        .store
        .register_client(client)
        .await
        .map_err(|e| OAuthError::ServerError(format!("failed to store client: {e}")))?;

    Ok(Json(RegistrationResponse {
        client_id,
        redirect_uris: body.redirect_uris,
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::post,
    };
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
            issuer: "https://example.com".into(),
            auth_secret: "test-secret-key-that-is-long-enough-32".into(),
            token_ttl_secs: 3600,
            allowed_users: None,
        };
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        OAuthState::new(config, store)
    }

    fn register_app(state: OAuthState) -> Router {
        Router::new()
            .route("/register", post(register_client))
            .with_state(state)
    }

    #[tokio::test]
    async fn register_valid_client() {
        let state = test_state();
        let app = register_app(state);
        let body = serde_json::json!({
            "redirect_uris": ["http://localhost:8080/callback"],
            "grant_types": ["authorization_code"],
            "client_name": "Test Client"
        });
        let resp = app
            .oneshot(
                Request::post("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["client_id"].is_string());
        assert!(!json["client_id"].as_str().unwrap().is_empty());
        assert_eq!(
            json["redirect_uris"],
            serde_json::json!(["http://localhost:8080/callback"])
        );
    }

    #[tokio::test]
    async fn register_empty_redirect_uris_fails() {
        let state = test_state();
        let app = register_app(state);
        let body = serde_json::json!({
            "redirect_uris": [],
        });
        let resp = app
            .oneshot(
                Request::post("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn register_unsupported_grant_type_fails() {
        let state = test_state();
        let app = register_app(state);
        let body = serde_json::json!({
            "redirect_uris": ["http://localhost/cb"],
            "grant_types": ["client_credentials"],
        });
        let resp = app
            .oneshot(
                Request::post("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn register_minimal_request() {
        let state = test_state();
        let app = register_app(state);
        let body = serde_json::json!({
            "redirect_uris": ["http://localhost:3000/callback"],
        });
        let resp = app
            .oneshot(
                Request::post("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn register_multiple_redirect_uris() {
        let state = test_state();
        let app = register_app(state);
        let body = serde_json::json!({
            "redirect_uris": ["http://localhost:8080/cb", "http://localhost:9090/cb"],
        });
        let resp = app
            .oneshot(
                Request::post("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["redirect_uris"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn registration_request_deserialize() {
        let json = r#"{"redirect_uris":["http://localhost/cb"],"grant_types":["authorization_code"],"client_name":"Test"}"#;
        let req: RegistrationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.redirect_uris.len(), 1);
        assert_eq!(req.grant_types.len(), 1);
        assert_eq!(req.client_name.as_deref(), Some("Test"));
    }

    #[test]
    fn registration_request_defaults() {
        let json = r#"{"redirect_uris":["http://localhost/cb"]}"#;
        let req: RegistrationRequest = serde_json::from_str(json).unwrap();
        assert!(req.grant_types.is_empty());
        assert!(req.client_name.is_none());
    }

    #[test]
    fn registration_response_serialize() {
        let resp = RegistrationResponse {
            client_id: "abc-123".to_owned(),
            redirect_uris: vec!["http://localhost/cb".to_owned()],
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["client_id"], "abc-123");
    }
}
