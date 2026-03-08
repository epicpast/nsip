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
