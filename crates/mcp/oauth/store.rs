//! Thread-safe OAuth store with pluggable backends.
//!
//! Defines the [`OAuthStoreBackend`] trait and provides an in-memory
//! implementation ([`InMemoryOAuthStore`]) with automatic TTL pruning.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};

/// Maximum age for pending authorizations and issued codes before pruning.
const TTL_SECS: i64 = 600;

/// Maximum age for refresh tokens before pruning (24 hours).
const REFRESH_TTL_SECS: i64 = 86400;

/// Boxed future type alias for trait methods.
type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A dynamically registered OAuth client.
#[derive(Debug, Clone)]
pub struct RegisteredClient {
    /// Client identifier.
    pub client_id: String,
    /// Allowed redirect URIs.
    pub redirect_uris: Vec<String>,
    /// When this client was registered.
    pub created_at: DateTime<Utc>,
}

/// A pending authorization request awaiting the GitHub callback.
#[derive(Debug, Clone)]
pub struct PendingAuth {
    /// The client that initiated the authorization.
    pub client_id: String,
    /// The redirect URI the client specified.
    pub redirect_uri: String,
    /// PKCE code challenge (S256).
    pub code_challenge: String,
    /// The client's original `state` parameter.
    pub original_state: String,
    /// When this pending auth was created.
    pub created_at: DateTime<Utc>,
}

/// An issued authorization code ready for exchange.
#[derive(Debug, Clone)]
pub struct IssuedAuthCode {
    /// The authorization code.
    pub code: String,
    /// Client that this code was issued to.
    pub client_id: String,
    /// Redirect URI from the original request.
    pub redirect_uri: String,
    /// PKCE code challenge.
    pub code_challenge: String,
    /// Authenticated GitHub username.
    pub github_login: String,
    /// When this code was issued.
    pub created_at: DateTime<Utc>,
}

/// A stored refresh token that can be exchanged for a new access token.
#[derive(Debug, Clone)]
pub struct StoredRefreshToken {
    /// The opaque refresh token value.
    pub token: String,
    /// Client that this token was issued to.
    pub client_id: String,
    /// Authenticated GitHub username.
    pub github_login: String,
    /// When this token was issued.
    pub created_at: DateTime<Utc>,
}

/// Async trait for OAuth store backends.
///
/// Implementations must be `Send + Sync` to support shared state across
/// async tasks and threads. Methods return boxed futures for object safety.
pub trait OAuthStoreBackend: Send + Sync {
    /// Register a new OAuth client.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn register_client(&self, client: RegisteredClient) -> BoxFut<'_, Result<(), String>>;

    /// Look up a registered client by ID.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn get_client(&self, client_id: &str) -> BoxFut<'_, Result<Option<RegisteredClient>, String>>;

    /// Store a pending authorization keyed by the internal state token.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn store_pending(
        &self,
        internal_state: String,
        pending: PendingAuth,
    ) -> BoxFut<'_, Result<(), String>>;

    /// Consume (remove and return) a pending authorization by its internal
    /// state.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn consume_pending(
        &self,
        internal_state: &str,
    ) -> BoxFut<'_, Result<Option<PendingAuth>, String>>;

    /// Store an issued authorization code.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn store_auth_code(&self, auth_code: IssuedAuthCode) -> BoxFut<'_, Result<(), String>>;

    /// Consume (remove and return) an issued authorization code.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn consume_auth_code(&self, code: &str) -> BoxFut<'_, Result<Option<IssuedAuthCode>, String>>;

    /// Store a refresh token.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn store_refresh_token(&self, rt: StoredRefreshToken) -> BoxFut<'_, Result<(), String>>;

    /// Consume (remove and return) a refresh token.
    ///
    /// # Errors
    ///
    /// Returns `Err` on storage failure.
    fn consume_refresh_token(
        &self,
        token: &str,
    ) -> BoxFut<'_, Result<Option<StoredRefreshToken>, String>>;
}

/// Internal store data protected by a read-write lock.
#[derive(Default)]
struct StoreInner {
    clients: HashMap<String, RegisteredClient>,
    pending: HashMap<String, PendingAuth>,
    codes: HashMap<String, IssuedAuthCode>,
    refresh_tokens: HashMap<String, StoredRefreshToken>,
}

/// Thread-safe in-memory OAuth store.
///
/// All entries are automatically pruned on access after exceeding their TTL.
#[derive(Clone, Default)]
pub struct InMemoryOAuthStore {
    inner: Arc<RwLock<StoreInner>>,
}

impl InMemoryOAuthStore {
    /// Create a new empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl OAuthStoreBackend for InMemoryOAuthStore {
    fn register_client(&self, client: RegisteredClient) -> BoxFut<'_, Result<(), String>> {
        Box::pin(async move {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            inner.clients.insert(client.client_id.clone(), client);
            drop(inner);
            Ok(())
        })
    }

    fn get_client(&self, client_id: &str) -> BoxFut<'_, Result<Option<RegisteredClient>, String>> {
        let client_id = client_id.to_owned();
        Box::pin(async move {
            let inner = self
                .inner
                .read()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            Ok(inner.clients.get(&client_id).cloned())
        })
    }

    fn store_pending(
        &self,
        internal_state: String,
        pending: PendingAuth,
    ) -> BoxFut<'_, Result<(), String>> {
        Box::pin(async move {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            prune_pending(&mut inner.pending);
            inner.pending.insert(internal_state, pending);
            drop(inner);
            Ok(())
        })
    }

    fn consume_pending(
        &self,
        internal_state: &str,
    ) -> BoxFut<'_, Result<Option<PendingAuth>, String>> {
        let internal_state = internal_state.to_owned();
        Box::pin(async move {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            prune_pending(&mut inner.pending);
            Ok(inner.pending.remove(&internal_state))
        })
    }

    fn store_auth_code(&self, auth_code: IssuedAuthCode) -> BoxFut<'_, Result<(), String>> {
        Box::pin(async move {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            prune_codes(&mut inner.codes);
            inner.codes.insert(auth_code.code.clone(), auth_code);
            drop(inner);
            Ok(())
        })
    }

    fn consume_auth_code(&self, code: &str) -> BoxFut<'_, Result<Option<IssuedAuthCode>, String>> {
        let code = code.to_owned();
        Box::pin(async move {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            prune_codes(&mut inner.codes);
            Ok(inner.codes.remove(&code))
        })
    }

    fn store_refresh_token(&self, rt: StoredRefreshToken) -> BoxFut<'_, Result<(), String>> {
        Box::pin(async move {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            prune_refresh_tokens(&mut inner.refresh_tokens);
            inner.refresh_tokens.insert(rt.token.clone(), rt);
            drop(inner);
            Ok(())
        })
    }

    fn consume_refresh_token(
        &self,
        token: &str,
    ) -> BoxFut<'_, Result<Option<StoredRefreshToken>, String>> {
        let token = token.to_owned();
        Box::pin(async move {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| format!("lock poisoned: {e}"))?;
            prune_refresh_tokens(&mut inner.refresh_tokens);
            Ok(inner.refresh_tokens.remove(&token))
        })
    }
}

/// Remove pending entries older than [`TTL_SECS`].
fn prune_pending(map: &mut HashMap<String, PendingAuth>) {
    let cutoff = Utc::now() - Duration::seconds(TTL_SECS);
    map.retain(|_, v| v.created_at > cutoff);
}

/// Remove auth code entries older than [`TTL_SECS`].
fn prune_codes(map: &mut HashMap<String, IssuedAuthCode>) {
    let cutoff = Utc::now() - Duration::seconds(TTL_SECS);
    map.retain(|_, v| v.created_at > cutoff);
}

/// Remove refresh token entries older than [`REFRESH_TTL_SECS`].
fn prune_refresh_tokens(map: &mut HashMap<String, StoredRefreshToken>) {
    let cutoff = Utc::now() - Duration::seconds(REFRESH_TTL_SECS);
    map.retain(|_, v| v.created_at > cutoff);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_get_client() {
        let store = InMemoryOAuthStore::new();
        let client = RegisteredClient {
            client_id: "c1".into(),
            redirect_uris: vec!["http://localhost/cb".into()],
            created_at: Utc::now(),
        };
        store.register_client(client).await.unwrap();
        let found = store.get_client("c1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().client_id, "c1");
    }

    #[tokio::test]
    async fn get_nonexistent_client() {
        let store = InMemoryOAuthStore::new();
        assert!(store.get_client("missing").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn store_and_consume_pending() {
        let store = InMemoryOAuthStore::new();
        let pending = PendingAuth {
            client_id: "c1".into(),
            redirect_uri: "http://localhost/cb".into(),
            code_challenge: "challenge".into(),
            original_state: "orig".into(),
            created_at: Utc::now(),
        };
        store
            .store_pending("state-key".into(), pending)
            .await
            .unwrap();
        let consumed = store.consume_pending("state-key").await.unwrap();
        assert!(consumed.is_some());
        assert_eq!(consumed.unwrap().client_id, "c1");
        // Second consume returns None
        assert!(store.consume_pending("state-key").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn store_and_consume_auth_code() {
        let store = InMemoryOAuthStore::new();
        let code = IssuedAuthCode {
            code: "code-123".into(),
            client_id: "c1".into(),
            redirect_uri: "http://localhost/cb".into(),
            code_challenge: "challenge".into(),
            github_login: "testuser".into(),
            created_at: Utc::now(),
        };
        store.store_auth_code(code).await.unwrap();
        let consumed = store.consume_auth_code("code-123").await.unwrap();
        assert!(consumed.is_some());
        assert_eq!(consumed.unwrap().github_login, "testuser");
        // Second consume returns None
        assert!(store.consume_auth_code("code-123").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn ttl_expiry_pending() {
        let store = InMemoryOAuthStore::new();
        let pending = PendingAuth {
            client_id: "c1".into(),
            redirect_uri: "http://localhost/cb".into(),
            code_challenge: "challenge".into(),
            original_state: "orig".into(),
            created_at: Utc::now() - Duration::seconds(700),
        };
        {
            let mut inner = store.inner.write().unwrap();
            inner.pending.insert("old-state".into(), pending);
        }
        assert!(store.consume_pending("old-state").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn ttl_expiry_auth_codes() {
        let store = InMemoryOAuthStore::new();
        let code = IssuedAuthCode {
            code: "old-code".into(),
            client_id: "c1".into(),
            redirect_uri: "http://localhost/cb".into(),
            code_challenge: "challenge".into(),
            github_login: "user".into(),
            created_at: Utc::now() - Duration::seconds(700),
        };
        {
            let mut inner = store.inner.write().unwrap();
            inner.codes.insert("old-code".into(), code);
        }
        assert!(store.consume_auth_code("old-code").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn store_and_consume_refresh_token() {
        let store = InMemoryOAuthStore::new();
        let rt = StoredRefreshToken {
            token: "rt-abc".into(),
            client_id: "c1".into(),
            github_login: "user1".into(),
            created_at: Utc::now(),
        };
        store.store_refresh_token(rt).await.unwrap();
        let consumed = store.consume_refresh_token("rt-abc").await.unwrap();
        assert!(consumed.is_some());
        let rt = consumed.unwrap();
        assert_eq!(rt.github_login, "user1");
        assert_eq!(rt.client_id, "c1");
        // Second consume returns None (rotation: one-time use)
        assert!(
            store
                .consume_refresh_token("rt-abc")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn consume_nonexistent_refresh_token_returns_none() {
        let store = InMemoryOAuthStore::new();
        assert!(
            store
                .consume_refresh_token("nonexistent")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn ttl_expiry_refresh_tokens() {
        let store = InMemoryOAuthStore::new();
        let rt = StoredRefreshToken {
            token: "old-rt".into(),
            client_id: "c1".into(),
            github_login: "user".into(),
            created_at: Utc::now() - Duration::seconds(REFRESH_TTL_SECS + 1),
        };
        {
            let mut inner = store.inner.write().unwrap();
            inner.refresh_tokens.insert("old-rt".into(), rt);
        }
        assert!(
            store
                .consume_refresh_token("old-rt")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn concurrent_access() {
        let store = InMemoryOAuthStore::new();
        let store2 = store.clone();
        let handle = tokio::spawn(async move {
            let client = RegisteredClient {
                client_id: "thread-client".into(),
                redirect_uris: vec![],
                created_at: Utc::now(),
            };
            store2.register_client(client).await.unwrap();
        });
        handle.await.unwrap();
        assert!(store.get_client("thread-client").await.unwrap().is_some());
    }
}
