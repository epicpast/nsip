//! OAuth 2.0 token endpoint with PKCE verification and refresh token support.
//!
//! Per RFC 6749 Section 4.1.3 and OAuth 2.1, the token endpoint accepts
//! `application/x-www-form-urlencoded` requests. JSON is also accepted as
//! a convenience for testing.

use axum::{
    Json,
    extract::{FromRequest as _, State},
    http::header::CONTENT_TYPE,
    response::{IntoResponse as _, Response},
};
use base64::Engine as _;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::Digest as _;

use crate::mcp::oauth::{OAuthState, error::OAuthError, store::StoredRefreshToken};

/// Token request body (application/x-www-form-urlencoded or JSON).
///
/// Fields are optional because different grant types use different subsets.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// The grant type: `"authorization_code"` or `"refresh_token"`.
    pub grant_type: String,
    /// The authorization code to exchange (for `authorization_code` grant).
    #[serde(default)]
    pub code: Option<String>,
    /// Must match the `redirect_uri` from the authorization request
    /// (for `authorization_code` grant).
    #[serde(default)]
    pub redirect_uri: Option<String>,
    /// PKCE code verifier (for `authorization_code` grant).
    #[serde(default)]
    pub code_verifier: Option<String>,
    /// The client identifier.
    #[serde(default)]
    pub client_id: Option<String>,
    /// The refresh token to exchange (for `refresh_token` grant).
    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// Successful token response.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// The issued access token (JWT).
    pub access_token: String,
    /// Token type (always `"bearer"`).
    pub token_type: String,
    /// Token lifetime in seconds.
    pub expires_in: u64,
    /// Opaque refresh token for obtaining new access tokens.
    pub refresh_token: String,
}

/// Generate an opaque refresh token.
fn generate_refresh_token() -> String {
    use rand::Rng as _;
    let bytes: [u8; 32] = rand::rng().random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// POST `/token` -- exchange an authorization code or refresh token for an
/// access token.
///
/// Accepts both `application/x-www-form-urlencoded` (per OAuth 2.1 spec) and
/// `application/json` for convenience.
///
/// # Errors
///
/// Returns [`OAuthError::UnsupportedGrantType`] for unknown grant types,
/// [`OAuthError::InvalidGrant`] for invalid/expired codes or tokens,
/// [`OAuthError::InvalidRequest`] for missing required parameters.
pub async fn exchange_token(
    State(state): State<OAuthState>,
    req: axum::extract::Request,
) -> Response {
    let content_type = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();

    let body: Result<TokenRequest, Response> =
        if content_type.contains("application/x-www-form-urlencoded") {
            // OAuth 2.1 spec: form-encoded
            let axum::extract::Form(form) =
                match axum::extract::Form::<TokenRequest>::from_request(req, &state).await {
                    Ok(f) => f,
                    Err(e) => {
                        return OAuthError::InvalidRequest(format!("malformed form body: {e}"))
                            .into_response();
                    },
                };
            Ok(form)
        } else {
            // JSON fallback
            let Json(json) = match Json::<TokenRequest>::from_request(req, &state).await {
                Ok(j) => j,
                Err(e) => {
                    return OAuthError::InvalidRequest(format!("malformed JSON body: {e}"))
                        .into_response();
                },
            };
            Ok(json)
        };

    let body = match body {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let result = match body.grant_type.as_str() {
        "authorization_code" => handle_authorization_code(&state, &body).await,
        "refresh_token" => handle_refresh_token(&state, &body).await,
        other => Err(OAuthError::UnsupportedGrantType(format!(
            "unsupported grant_type: {other}"
        ))),
    };

    match result {
        Ok(json) => json.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Handle `grant_type=authorization_code`.
async fn handle_authorization_code(
    state: &OAuthState,
    body: &TokenRequest,
) -> Result<Json<TokenResponse>, OAuthError> {
    let code = body
        .code
        .as_deref()
        .ok_or_else(|| OAuthError::InvalidRequest("missing code".into()))?;
    let redirect_uri = body
        .redirect_uri
        .as_deref()
        .ok_or_else(|| OAuthError::InvalidRequest("missing redirect_uri".into()))?;
    let code_verifier = body
        .code_verifier
        .as_deref()
        .ok_or_else(|| OAuthError::InvalidRequest("missing code_verifier".into()))?;
    let client_id = body
        .client_id
        .as_deref()
        .ok_or_else(|| OAuthError::InvalidRequest("missing client_id".into()))?;

    // Consume the authorization code
    let issued = state
        .store
        .consume_auth_code(code)
        .await
        .map_err(OAuthError::ServerError)?
        .ok_or_else(|| OAuthError::InvalidGrant("unknown or expired code".into()))?;

    // Validate client_id and redirect_uri
    if issued.client_id != client_id {
        return Err(OAuthError::InvalidRequest("client_id mismatch".into()));
    }
    if issued.redirect_uri != redirect_uri {
        return Err(OAuthError::InvalidRequest("redirect_uri mismatch".into()));
    }

    // PKCE verification
    verify_pkce(code_verifier, &issued.code_challenge)?;

    // Issue JWT + refresh token
    issue_token_pair(state, &issued.github_login, client_id).await
}

/// Handle `grant_type=refresh_token`.
async fn handle_refresh_token(
    state: &OAuthState,
    body: &TokenRequest,
) -> Result<Json<TokenResponse>, OAuthError> {
    let refresh_token = body
        .refresh_token
        .as_deref()
        .ok_or_else(|| OAuthError::InvalidRequest("missing refresh_token".into()))?;

    // Consume the old refresh token (rotation: one-time use)
    let stored = state
        .store
        .consume_refresh_token(refresh_token)
        .await
        .map_err(OAuthError::ServerError)?
        .ok_or_else(|| OAuthError::InvalidGrant("invalid or expired refresh_token".into()))?;

    // Issue a new JWT + new refresh token
    issue_token_pair(state, &stored.github_login, &stored.client_id).await
}

/// Issue an access token (JWT) and a refresh token, storing the refresh token.
async fn issue_token_pair(
    state: &OAuthState,
    github_login: &str,
    client_id: &str,
) -> Result<Json<TokenResponse>, OAuthError> {
    let access_token = state.jwt.issue(github_login)?;

    let rt_value = generate_refresh_token();
    let stored_rt = StoredRefreshToken {
        token: rt_value.clone(),
        client_id: client_id.to_owned(),
        github_login: github_login.to_owned(),
        created_at: Utc::now(),
    };
    state
        .store
        .store_refresh_token(stored_rt)
        .await
        .map_err(OAuthError::ServerError)?;

    Ok(Json(TokenResponse {
        access_token,
        token_type: "bearer".into(),
        expires_in: state.config.token_ttl_secs,
        refresh_token: rt_value,
    }))
}

/// Verify a PKCE code verifier against a code challenge (S256 method).
///
/// # Errors
///
/// Returns [`OAuthError::InvalidGrant`] if the verifier does not match.
fn verify_pkce(code_verifier: &str, code_challenge: &str) -> Result<(), OAuthError> {
    let digest = sha2::Sha256::digest(code_verifier.as_bytes());
    let computed = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    if subtle::ConstantTimeEq::ct_eq(computed.as_bytes(), code_challenge.as_bytes()).into() {
        Ok(())
    } else {
        Err(OAuthError::InvalidGrant(
            "PKCE code_verifier does not match code_challenge".into(),
        ))
    }
}

#[cfg(test)]
#[path = "token_tests.rs"]
mod tests;
