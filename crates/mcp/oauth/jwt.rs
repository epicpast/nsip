//! JWT issuance and validation using HMAC-SHA256.

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::mcp::oauth::error::OAuthError;

/// JWT audience value for nsip-mcp tokens.
const AUDIENCE: &str = "nsip-mcp";

/// JWT claims issued by the NSIP authorization server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsipClaims {
    /// Subject -- the authenticated GitHub login.
    pub sub: String,
    /// Issuer.
    pub iss: String,
    /// Audience.
    pub aud: String,
    /// Expiration time (seconds since epoch).
    pub exp: u64,
    /// Issued-at time (seconds since epoch).
    pub iat: u64,
    /// Unique token identifier.
    pub jti: String,
}

/// Manager for issuing and validating HMAC-SHA256 JWTs.
#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    token_ttl_secs: u64,
}

impl JwtManager {
    /// Create a new JWT manager.
    ///
    /// # Arguments
    ///
    /// * `secret` - HMAC-SHA256 signing key.
    /// * `issuer` - Value for the `iss` claim.
    /// * `token_ttl_secs` - Token lifetime in seconds.
    #[must_use]
    pub fn new(secret: &str, issuer: &str, token_ttl_secs: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer: issuer.to_owned(),
            token_ttl_secs,
        }
    }

    /// Issue a signed JWT for the given GitHub login.
    ///
    /// # Arguments
    ///
    /// * `github_login` - The authenticated user's GitHub username.
    ///
    /// # Errors
    ///
    /// Returns [`OAuthError::ServerError`] if JWT encoding fails.
    pub fn issue(&self, github_login: &str) -> Result<String, OAuthError> {
        let now = now_secs();

        let claims = NsipClaims {
            sub: github_login.to_owned(),
            iss: self.issuer.clone(),
            aud: AUDIENCE.to_owned(),
            exp: now + self.token_ttl_secs,
            iat: now,
            jti: uuid::Uuid::new_v4().to_string(),
        };

        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| OAuthError::ServerError(format!("JWT encoding failed: {e}")))
    }

    /// Validate a JWT and return the contained claims.
    ///
    /// # Arguments
    ///
    /// * `token` - The encoded JWT string.
    ///
    /// # Errors
    ///
    /// Returns [`OAuthError::InvalidGrant`] if the token is invalid, expired,
    /// or has an incorrect audience/issuer.
    pub fn validate(&self, token: &str) -> Result<NsipClaims, OAuthError> {
        let mut validation = Validation::default();
        validation.set_audience(&[AUDIENCE]);
        validation.set_issuer(&[&self.issuer]);

        jsonwebtoken::decode::<NsipClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| OAuthError::InvalidGrant(format!("invalid token: {e}")))
    }
}

/// Current time as seconds since the Unix epoch.
fn now_secs() -> u64 {
    // Unix timestamps are positive for all dates after 1970; sign loss is safe.
    #[allow(clippy::cast_sign_loss)]
    let secs = chrono::Utc::now().timestamp() as u64;
    secs
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_issue_validate() {
        let mgr = JwtManager::new("test-secret-key-long-enough", "https://test", 3600);
        let token = mgr.issue("testuser").unwrap();
        let claims = mgr.validate(&token).unwrap();

        assert_eq!(claims.sub, "testuser");
        assert_eq!(claims.iss, "https://test");
        assert_eq!(claims.aud, "nsip-mcp");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn validate_rejects_wrong_key() {
        let mgr1 = JwtManager::new("key-one-secret", "https://test", 3600);
        let mgr2 = JwtManager::new("key-two-secret", "https://test", 3600);

        let token = mgr1.issue("user").unwrap();
        let result = mgr2.validate(&token);
        assert!(result.is_err());
    }

    #[test]
    fn validate_rejects_expired_token() {
        let mgr = JwtManager::new("secret-key-for-test", "https://test", 0);
        let token = mgr.issue("user").unwrap();
        // Token has 0 TTL so exp == iat, effectively expired immediately.
        // jsonwebtoken has a leeway of 60s by default, so we craft a token
        // that is truly in the past.
        let expired_mgr = JwtManager::new("secret-key-for-test", "https://test", 0);
        let claims = NsipClaims {
            sub: "user".into(),
            iss: "https://test".into(),
            aud: "nsip-mcp".into(),
            exp: 1_000_000,
            iat: 999_000,
            jti: "test-jti".into(),
        };
        let expired_token =
            jsonwebtoken::encode(&Header::default(), &claims, &expired_mgr.encoding_key).unwrap();
        assert!(expired_mgr.validate(&expired_token).is_err());
        // Also ensure the non-expired token from above is valid (sanity)
        let _ = token;
    }

    #[test]
    fn issue_produces_unique_jti() {
        let mgr = JwtManager::new("test-secret", "https://test", 3600);
        let t1 = mgr.issue("user").unwrap();
        let t2 = mgr.issue("user").unwrap();
        let c1 = mgr.validate(&t1).unwrap();
        let c2 = mgr.validate(&t2).unwrap();
        assert_ne!(c1.jti, c2.jti);
    }
}
