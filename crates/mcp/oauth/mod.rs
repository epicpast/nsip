//! OAuth 2.1 authorization module with GitHub identity provider.
//!
//! Implements the full OAuth 2.1 authorization code flow with PKCE (S256),
//! dynamic client registration (RFC 7591), authorization server metadata
//! (RFC 8414), and protected resource metadata (RFC 9728).

use std::sync::Arc;

pub mod authorize;
pub mod callback;
pub mod config;
pub mod discovery;
pub mod error;
pub mod github;
pub mod jwt;
pub mod middleware;
pub mod registration;
pub mod store;
pub mod token;

use config::OAuthConfig;
use github::{GitHubApi, GitHubClient};
use jwt::JwtManager;
use middleware::PatCache;
use store::OAuthStoreBackend;

/// Shared state for all OAuth endpoints.
#[derive(Clone)]
pub struct OAuthState {
    /// OAuth configuration.
    pub config: Arc<OAuthConfig>,
    /// Pluggable store for clients, pending auths, auth codes, and refresh
    /// tokens.
    pub store: Arc<dyn OAuthStoreBackend>,
    /// JWT manager for token issuance and validation.
    pub jwt: JwtManager,
    /// GitHub API client (trait object for testability).
    pub github: Arc<dyn GitHubApi>,
    /// In-memory cache for validated GitHub PATs.
    pub pat_cache: PatCache,
}

impl OAuthState {
    /// Create a new [`OAuthState`] from the given configuration and store
    /// backend.
    ///
    /// Uses the live [`GitHubClient`] for GitHub API calls.
    #[must_use]
    pub fn new(config: OAuthConfig, store: Arc<dyn OAuthStoreBackend>) -> Self {
        let jwt = JwtManager::new(&config.auth_secret, &config.issuer, config.token_ttl_secs);
        let github: Arc<dyn GitHubApi> = Arc::new(GitHubClient::new(
            config.github_client_id.clone(),
            config.github_client_secret.clone(),
        ));
        Self {
            config: Arc::new(config),
            store,
            jwt,
            github,
            pat_cache: middleware::new_pat_cache(),
        }
    }
}

/// Build an axum [`Router`](axum::Router) for all OAuth endpoints.
///
/// The returned router includes:
/// - `GET /.well-known/oauth-authorization-server`
/// - `GET /.well-known/oauth-protected-resource`
/// - `POST /register`
/// - `GET /authorize`
/// - `GET /callback`
/// - `POST /token`
pub fn oauth_router(state: OAuthState) -> axum::Router {
    use axum::routing::{get, post};

    axum::Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(discovery::authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(discovery::protected_resource_metadata),
        )
        .route("/register", post(registration::register_client))
        .route("/authorize", get(authorize::authorize))
        .route("/callback", get(callback::callback))
        .route("/token", post(token::exchange_token))
        .with_state(state)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::mcp::oauth::store::InMemoryOAuthStore;

    fn make_config() -> OAuthConfig {
        OAuthConfig {
            github_client_id: "gh-client-id".into(),
            github_client_secret: "gh-client-secret".into(),
            base_url: "https://example.com".into(),
            issuer: "https://example.com".into(),
            auth_secret: "test-secret-key-that-is-long-enough".into(),
            token_ttl_secs: 3600,
            allowed_users: None,
        }
    }

    #[test]
    fn oauth_state_new_construction() {
        let config = make_config();
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        let state = OAuthState::new(config, store);
        assert_eq!(state.config.github_client_id, "gh-client-id");
        assert_eq!(state.config.issuer, "https://example.com");
    }

    #[test]
    fn oauth_state_clone_shares_config() {
        let config = make_config();
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        let state = OAuthState::new(config, store);
        let cloned = state.clone();
        assert_eq!(
            state.config.github_client_id,
            cloned.config.github_client_id
        );
    }

    #[test]
    fn oauth_state_has_pat_cache() {
        let config = make_config();
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        let state = OAuthState::new(config, store);
        assert!(state.pat_cache.read().unwrap().is_empty());
    }

    #[test]
    fn oauth_router_builds_without_panic() {
        let config = make_config();
        let store = Arc::new(InMemoryOAuthStore::new()) as Arc<dyn OAuthStoreBackend>;
        let state = OAuthState::new(config, store);
        let _router = super::oauth_router(state);
    }
}
