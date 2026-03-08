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
