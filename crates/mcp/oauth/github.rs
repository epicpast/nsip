//! GitHub OAuth API client with trait-based abstraction for test mocking.

use std::future::Future;
use std::pin::Pin;

use serde::Deserialize;

use crate::mcp::oauth::error::OAuthError;

/// Boxed future type alias for trait methods.
type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// GitHub user profile information.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    /// GitHub username (login).
    pub login: String,
    /// Numeric GitHub user ID.
    pub id: u64,
    /// Display name (may be absent).
    pub name: Option<String>,
}

/// Abstraction over the GitHub OAuth API for testability.
///
/// Methods return boxed futures for object safety (`Arc<dyn GitHubApi>`).
pub trait GitHubApi: Send + Sync {
    /// Exchange a GitHub authorization code for an access token.
    ///
    /// # Errors
    ///
    /// Returns [`OAuthError::ServerError`] if the HTTP request or token
    /// exchange fails.
    fn exchange_code(&self, code: &str) -> BoxFut<'_, Result<String, OAuthError>>;

    /// Fetch the authenticated user's GitHub profile.
    ///
    /// # Errors
    ///
    /// Returns [`OAuthError::ServerError`] if the HTTP request fails or the
    /// response cannot be parsed.
    fn get_user(&self, token: &str) -> BoxFut<'_, Result<GitHubUser, OAuthError>>;
}

/// Live GitHub API client using `reqwest`.
pub struct GitHubClient {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
}

impl GitHubClient {
    /// Create a new GitHub API client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - GitHub OAuth application client ID.
    /// * `client_secret` - GitHub OAuth application client secret.
    ///
    /// # Errors
    ///
    /// Returns [`OAuthError::ServerError`] if the underlying HTTP client cannot
    /// be constructed (e.g. the TLS backend fails to initialize). Propagating
    /// rather than falling back keeps the required `User-Agent` (GitHub rejects
    /// requests without one) and avoids a hidden panic in the default client.
    pub fn new(client_id: String, client_secret: String) -> Result<Self, OAuthError> {
        let http = reqwest::Client::builder()
            .user_agent("nsip-mcp")
            .build()
            .map_err(|e| {
                OAuthError::ServerError(format!("failed to build GitHub HTTP client: {e}"))
            })?;
        Ok(Self {
            http,
            client_id,
            client_secret,
        })
    }
}

/// Response from GitHub's token exchange endpoint.
#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

impl GitHubApi for GitHubClient {
    fn exchange_code(&self, code: &str) -> BoxFut<'_, Result<String, OAuthError>> {
        let code = code.to_owned();
        Box::pin(async move {
            let resp = self
                .http
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .json(&serde_json::json!({
                    "client_id": self.client_id,
                    "client_secret": self.client_secret,
                    "code": code,
                }))
                .send()
                .await
                .map_err(|e| {
                    OAuthError::ServerError(format!("GitHub token request failed: {e}"))
                })?;

            let token_resp: TokenResponse = resp
                .json()
                .await
                .map_err(|e| OAuthError::ServerError(format!("GitHub token parse failed: {e}")))?;

            if let Some(err) = token_resp.error {
                let desc = token_resp.error_description.unwrap_or_else(|| err.clone());
                return Err(OAuthError::InvalidGrant(desc));
            }

            token_resp
                .access_token
                .ok_or_else(|| OAuthError::ServerError("no access_token in response".into()))
        })
    }

    fn get_user(&self, token: &str) -> BoxFut<'_, Result<GitHubUser, OAuthError>> {
        let token = token.to_owned();
        Box::pin(async move {
            let resp = self
                .http
                .get("https://api.github.com/user")
                .header("Authorization", format!("Bearer {token}"))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| OAuthError::ServerError(format!("GitHub user request failed: {e}")))?;

            if !resp.status().is_success() {
                return Err(OAuthError::ServerError(format!(
                    "GitHub API returned {}",
                    resp.status()
                )));
            }

            resp.json::<GitHubUser>()
                .await
                .map_err(|e| OAuthError::ServerError(format!("GitHub user parse failed: {e}")))
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Mock GitHub API for testing.
    pub(crate) struct MockGitHub {
        pub(crate) user: GitHubUser,
    }

    impl GitHubApi for MockGitHub {
        fn exchange_code(&self, _code: &str) -> BoxFut<'_, Result<String, OAuthError>> {
            Box::pin(async { Ok("mock-token".into()) })
        }

        fn get_user(&self, _token: &str) -> BoxFut<'_, Result<GitHubUser, OAuthError>> {
            let user = self.user.clone();
            Box::pin(async move { Ok(user) })
        }
    }

    #[test]
    fn github_client_construction() {
        let client =
            GitHubClient::new("client-id".into(), "client-secret".into()).expect("build client");
        assert_eq!(client.client_id, "client-id");
        assert_eq!(client.client_secret, "client-secret");
    }

    #[test]
    fn github_user_deserialize() {
        let json = r#"{"login":"alice","id":123,"name":"Alice Smith"}"#;
        let user: GitHubUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "alice");
        assert_eq!(user.id, 123);
        assert_eq!(user.name.as_deref(), Some("Alice Smith"));
    }

    #[test]
    fn github_user_deserialize_no_name() {
        let json = r#"{"login":"bob","id":456}"#;
        let user: GitHubUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "bob");
        assert!(user.name.is_none());
    }

    #[tokio::test]
    async fn mock_github_round_trip() {
        let mock = MockGitHub {
            user: GitHubUser {
                login: "testuser".into(),
                id: 42,
                name: Some("Test User".into()),
            },
        };
        let token = mock.exchange_code("code").await.unwrap();
        assert_eq!(token, "mock-token");
        let user = mock.get_user(&token).await.unwrap();
        assert_eq!(user.login, "testuser");
        assert_eq!(user.id, 42);
    }

    #[test]
    fn github_user_clone() {
        let user = GitHubUser {
            login: "alice".into(),
            id: 100,
            name: Some("Alice".into()),
        };
        #[allow(clippy::redundant_clone)]
        let cloned = user.clone();
        assert_eq!(cloned.login, "alice");
        assert_eq!(cloned.id, 100);
        assert_eq!(cloned.name.as_deref(), Some("Alice"));
    }

    #[test]
    fn github_user_debug_format() {
        let user = GitHubUser {
            login: "bob".into(),
            id: 1,
            name: None,
        };
        let debug = format!("{user:?}");
        assert!(debug.contains("bob"));
        assert!(debug.contains('1'));
    }

    #[test]
    fn github_user_deserialize_extra_fields_ignored() {
        let json = r#"{"login":"carol","id":789,"name":"Carol","avatar_url":"http://example.com/a.png","email":"c@example.com"}"#;
        let user: GitHubUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "carol");
        assert_eq!(user.id, 789);
    }

    #[test]
    fn github_client_fields_accessible() {
        let client = GitHubClient::new("id-123".into(), "secret-456".into()).expect("build client");
        assert_eq!(client.client_id, "id-123");
        assert_eq!(client.client_secret, "secret-456");
    }

    // -------------------------------------------------------------------
    // Mock error paths
    // -------------------------------------------------------------------

    struct MockGitHubError;

    impl GitHubApi for MockGitHubError {
        fn exchange_code(&self, _code: &str) -> BoxFut<'_, Result<String, OAuthError>> {
            Box::pin(async { Err(OAuthError::ServerError("exchange failed".into())) })
        }

        fn get_user(&self, _token: &str) -> BoxFut<'_, Result<GitHubUser, OAuthError>> {
            Box::pin(async { Err(OAuthError::ServerError("user fetch failed".into())) })
        }
    }

    #[tokio::test]
    async fn mock_github_exchange_error() {
        let mock = MockGitHubError;
        let result = mock.exchange_code("code").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code(), "server_error");
        assert!(err.description().contains("exchange failed"));
    }

    #[tokio::test]
    async fn mock_github_get_user_error() {
        let mock = MockGitHubError;
        let result = mock.get_user("token").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.description().contains("user fetch failed"));
    }

    // -------------------------------------------------------------------
    // TokenResponse internal deserialization
    // -------------------------------------------------------------------

    #[test]
    fn token_response_deserializes_success() {
        let json = r#"{"access_token":"gho_abc123"}"#;
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token.as_deref(), Some("gho_abc123"));
        assert!(resp.error.is_none());
    }

    #[test]
    fn token_response_deserializes_error() {
        let json = r#"{"error":"bad_verification_code","error_description":"The code passed is incorrect or expired."}"#;
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert!(resp.access_token.is_none());
        assert_eq!(resp.error.as_deref(), Some("bad_verification_code"));
        assert!(
            resp.error_description
                .as_deref()
                .unwrap()
                .contains("incorrect")
        );
    }

    #[test]
    fn token_response_deserializes_empty() {
        let json = r"{}";
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert!(resp.access_token.is_none());
        assert!(resp.error.is_none());
        assert!(resp.error_description.is_none());
    }

    #[test]
    fn github_user_deserialize_numeric_id_boundaries() {
        let json = r#"{"login":"max","id":18446744073709551615}"#;
        let user: GitHubUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, u64::MAX);
    }

    #[test]
    fn github_user_deserialize_empty_name() {
        let json = r#"{"login":"empty","id":1,"name":""}"#;
        let user: GitHubUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.name.as_deref(), Some(""));
    }

    #[tokio::test]
    async fn mock_github_user_without_name() {
        let mock = MockGitHub {
            user: GitHubUser {
                login: "noname".into(),
                id: 99,
                name: None,
            },
        };
        let user = mock.get_user("any-token").await.unwrap();
        assert!(user.name.is_none());
        assert_eq!(user.login, "noname");
    }

    #[test]
    fn github_client_http_client_is_valid() {
        let client = GitHubClient::new("cid".into(), "csec".into()).expect("build client");
        assert_eq!(client.client_id, "cid");
    }
}
