//! OAuth 2.1 configuration loaded from environment variables.

use std::env;

/// OAuth 2.1 configuration for GitHub identity provider.
///
/// All required fields are populated from environment variables.
/// Optional fields have sensible defaults.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// GitHub OAuth application client ID.
    pub github_client_id: String,
    /// GitHub OAuth application client secret.
    pub github_client_secret: String,
    /// HMAC-SHA256 secret key for signing JWTs.
    pub auth_secret: String,
    /// JWT issuer claim (default: `"https://localhost"`).
    pub issuer: String,
    /// External base URL for constructing redirect URIs.
    pub base_url: String,
    /// Access token time-to-live in seconds (default: 3600).
    pub token_ttl_secs: u64,
    /// Optional allowlist of GitHub usernames. When `Some`, only listed users
    /// are permitted to authenticate.
    pub allowed_users: Option<Vec<String>>,
}

/// Default token TTL in seconds (1 hour).
const DEFAULT_TOKEN_TTL: u64 = 3600;

/// Default JWT issuer.
const DEFAULT_ISSUER: &str = "https://localhost";

impl OAuthConfig {
    /// Attempt to construct an [`OAuthConfig`] from environment variables.
    ///
    /// Returns `None` if any required variable (`NSIP_GITHUB_CLIENT_ID`,
    /// `NSIP_GITHUB_CLIENT_SECRET`, `NSIP_AUTH_SECRET`,
    /// `NSIP_AUTH_BASE_URL`) is missing.
    #[must_use]
    pub fn try_from_env() -> Option<Self> {
        Self::try_from_lookup(|k| env::var(k).ok())
    }

    /// Construct an [`OAuthConfig`] using a custom variable lookup function.
    ///
    /// This allows testing without modifying process environment variables.
    /// Returns `None` if any required key is missing from the lookup.
    #[must_use]
    pub fn try_from_lookup<F>(lookup: F) -> Option<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        let github_client_id = lookup("NSIP_GITHUB_CLIENT_ID")?;
        let github_client_secret = lookup("NSIP_GITHUB_CLIENT_SECRET")?;
        let auth_secret = lookup("NSIP_AUTH_SECRET")?;
        let base_url = lookup("NSIP_AUTH_BASE_URL")?;

        let issuer = lookup("NSIP_AUTH_ISSUER").unwrap_or_else(|| DEFAULT_ISSUER.to_owned());

        let token_ttl_secs = lookup("NSIP_AUTH_TOKEN_TTL")
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_TOKEN_TTL);

        let allowed_users = lookup("NSIP_AUTH_ALLOWED_USERS").and_then(|v| {
            let users: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect();
            if users.is_empty() { None } else { Some(users) }
        });

        Some(Self {
            github_client_id,
            github_client_secret,
            auth_secret,
            issuer,
            base_url,
            token_ttl_secs,
            allowed_users,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// Build a lookup function from a set of key-value pairs.
    fn make_lookup(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    #[test]
    fn try_from_lookup_returns_some_with_required_vars() {
        let lookup = make_lookup(&[
            ("NSIP_GITHUB_CLIENT_ID", "cid"),
            ("NSIP_GITHUB_CLIENT_SECRET", "csecret"),
            ("NSIP_AUTH_SECRET", "hmac-key"),
            ("NSIP_AUTH_BASE_URL", "https://example.com"),
        ]);

        let config = OAuthConfig::try_from_lookup(lookup).unwrap();
        assert_eq!(config.github_client_id, "cid");
        assert_eq!(config.github_client_secret, "csecret");
        assert_eq!(config.auth_secret, "hmac-key");
        assert_eq!(config.base_url, "https://example.com");
        assert_eq!(config.issuer, "https://localhost");
        assert_eq!(config.token_ttl_secs, 3600);
        assert!(config.allowed_users.is_none());
    }

    #[test]
    fn try_from_lookup_returns_none_when_missing_required() {
        let lookup = make_lookup(&[]);
        assert!(OAuthConfig::try_from_lookup(lookup).is_none());
    }

    #[test]
    fn try_from_lookup_parses_allowed_users() {
        let lookup = make_lookup(&[
            ("NSIP_GITHUB_CLIENT_ID", "cid"),
            ("NSIP_GITHUB_CLIENT_SECRET", "csecret"),
            ("NSIP_AUTH_SECRET", "hmac-key"),
            ("NSIP_AUTH_BASE_URL", "https://example.com"),
            ("NSIP_AUTH_ALLOWED_USERS", "alice, bob, charlie"),
        ]);

        let config = OAuthConfig::try_from_lookup(lookup).unwrap();
        let users = config.allowed_users.unwrap();
        assert_eq!(users, vec!["alice", "bob", "charlie"]);
    }

    #[test]
    fn try_from_lookup_custom_ttl_and_issuer() {
        let lookup = make_lookup(&[
            ("NSIP_GITHUB_CLIENT_ID", "cid"),
            ("NSIP_GITHUB_CLIENT_SECRET", "csecret"),
            ("NSIP_AUTH_SECRET", "hmac-key"),
            ("NSIP_AUTH_BASE_URL", "https://example.com"),
            ("NSIP_AUTH_ISSUER", "https://custom.example"),
            ("NSIP_AUTH_TOKEN_TTL", "7200"),
        ]);

        let config = OAuthConfig::try_from_lookup(lookup).unwrap();
        assert_eq!(config.issuer, "https://custom.example");
        assert_eq!(config.token_ttl_secs, 7200);
    }

    #[test]
    fn try_from_lookup_missing_one_required_var() {
        // Missing NSIP_AUTH_SECRET
        let lookup = make_lookup(&[
            ("NSIP_GITHUB_CLIENT_ID", "cid"),
            ("NSIP_GITHUB_CLIENT_SECRET", "csecret"),
            ("NSIP_AUTH_BASE_URL", "https://example.com"),
        ]);
        assert!(OAuthConfig::try_from_lookup(lookup).is_none());
    }

    #[test]
    fn try_from_lookup_invalid_ttl_uses_default() {
        let lookup = make_lookup(&[
            ("NSIP_GITHUB_CLIENT_ID", "cid"),
            ("NSIP_GITHUB_CLIENT_SECRET", "csecret"),
            ("NSIP_AUTH_SECRET", "hmac-key"),
            ("NSIP_AUTH_BASE_URL", "https://example.com"),
            ("NSIP_AUTH_TOKEN_TTL", "not-a-number"),
        ]);

        let config = OAuthConfig::try_from_lookup(lookup).unwrap();
        assert_eq!(config.token_ttl_secs, DEFAULT_TOKEN_TTL);
    }
}
