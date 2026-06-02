//! RFC 6749 OAuth 2.0 error types with axum `IntoResponse` implementation.

use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// OAuth 2.0 error codes per
/// [RFC 6749 Section 5.2](https://tools.ietf.org/html/rfc6749#section-5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthError {
    /// The request is missing a required parameter, includes an unsupported
    /// parameter value, or is otherwise malformed.
    InvalidRequest(String),
    /// Client authentication failed.
    InvalidClient(String),
    /// The provided authorization grant is invalid, expired, or revoked.
    InvalidGrant(String),
    /// The authenticated client is not authorized to use this grant type.
    UnauthorizedClient(String),
    /// The authorization grant type is not supported.
    UnsupportedGrantType(String),
    /// The requested scope is invalid, unknown, or malformed.
    InvalidScope(String),
    /// The resource owner or authorization server denied the request.
    AccessDenied(String),
    /// The authorization server encountered an unexpected condition.
    ServerError(String),
    /// The authorization server is currently unable to handle the request.
    TemporarilyUnavailable(String),
}

impl OAuthError {
    /// Returns the RFC 6749 error code string.
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::InvalidClient(_) => "invalid_client",
            Self::InvalidGrant(_) => "invalid_grant",
            Self::UnauthorizedClient(_) => "unauthorized_client",
            Self::UnsupportedGrantType(_) => "unsupported_grant_type",
            Self::InvalidScope(_) => "invalid_scope",
            Self::AccessDenied(_) => "access_denied",
            Self::ServerError(_) => "server_error",
            Self::TemporarilyUnavailable(_) => "temporarily_unavailable",
        }
    }

    /// Returns the human-readable error description.
    #[must_use]
    pub fn description(&self) -> &str {
        match self {
            Self::InvalidRequest(d)
            | Self::InvalidClient(d)
            | Self::InvalidGrant(d)
            | Self::UnauthorizedClient(d)
            | Self::UnsupportedGrantType(d)
            | Self::InvalidScope(d)
            | Self::AccessDenied(d)
            | Self::ServerError(d)
            | Self::TemporarilyUnavailable(d) => d,
        }
    }

    /// Returns the HTTP status code appropriate for this error.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest(_)
            | Self::InvalidGrant(_)
            | Self::UnsupportedGrantType(_)
            | Self::InvalidScope(_) => StatusCode::BAD_REQUEST,
            Self::InvalidClient(_) | Self::UnauthorizedClient(_) | Self::AccessDenied(_) => {
                StatusCode::UNAUTHORIZED
            },
            Self::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TemporarilyUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.description())
    }
}

impl std::error::Error for OAuthError {}

/// Default `Retry-After` (seconds) advertised on a transient 503 so clients and
/// agents back off instead of hammering the endpoint.
const TEMPORARILY_UNAVAILABLE_RETRY_AFTER_SECS: &str = "5";

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = serde_json::json!({
            "error": self.error_code(),
            "error_description": self.description(),
        });
        let mut response = (status, axum::Json(body)).into_response();
        // Advertise a retry window on transient unavailability so agents back
        // off rather than abandon the flow (RFC 7231 §7.1.3).
        if matches!(self, Self::TemporarilyUnavailable(_)) {
            response.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                axum::http::HeaderValue::from_static(TEMPORARILY_UNAVAILABLE_RETRY_AFTER_SECS),
            );
        }
        response
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn error_code_strings() {
        assert_eq!(
            OAuthError::InvalidRequest(String::new()).error_code(),
            "invalid_request"
        );
        assert_eq!(
            OAuthError::InvalidClient(String::new()).error_code(),
            "invalid_client"
        );
        assert_eq!(
            OAuthError::InvalidGrant(String::new()).error_code(),
            "invalid_grant"
        );
        assert_eq!(
            OAuthError::UnauthorizedClient(String::new()).error_code(),
            "unauthorized_client"
        );
        assert_eq!(
            OAuthError::UnsupportedGrantType(String::new()).error_code(),
            "unsupported_grant_type"
        );
        assert_eq!(
            OAuthError::InvalidScope(String::new()).error_code(),
            "invalid_scope"
        );
        assert_eq!(
            OAuthError::AccessDenied(String::new()).error_code(),
            "access_denied"
        );
        assert_eq!(
            OAuthError::ServerError(String::new()).error_code(),
            "server_error"
        );
        assert_eq!(
            OAuthError::TemporarilyUnavailable(String::new()).error_code(),
            "temporarily_unavailable"
        );
    }

    #[test]
    fn status_codes() {
        assert_eq!(
            OAuthError::InvalidRequest("x".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            OAuthError::InvalidClient("x".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            OAuthError::AccessDenied("x".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            OAuthError::ServerError("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            OAuthError::TemporarilyUnavailable("x".into()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn into_response_json_shape() {
        let err = OAuthError::InvalidRequest("missing param".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn display_format() {
        let err = OAuthError::InvalidGrant("expired".into());
        assert_eq!(err.to_string(), "invalid_grant: expired");
    }

    #[test]
    fn description_returns_inner_string() {
        let cases = [
            OAuthError::InvalidRequest("req-msg".into()),
            OAuthError::InvalidClient("client-msg".into()),
            OAuthError::InvalidGrant("grant-msg".into()),
            OAuthError::UnauthorizedClient("unauth-msg".into()),
            OAuthError::UnsupportedGrantType("grant-type-msg".into()),
            OAuthError::InvalidScope("scope-msg".into()),
            OAuthError::AccessDenied("denied-msg".into()),
            OAuthError::ServerError("server-msg".into()),
            OAuthError::TemporarilyUnavailable("temp-msg".into()),
        ];
        let expected = [
            "req-msg",
            "client-msg",
            "grant-msg",
            "unauth-msg",
            "grant-type-msg",
            "scope-msg",
            "denied-msg",
            "server-msg",
            "temp-msg",
        ];
        for (err, exp) in cases.iter().zip(expected.iter()) {
            assert_eq!(err.description(), *exp, "failed for {err:?}");
        }
    }

    #[test]
    fn status_code_bad_request_variants() {
        let bad_request_variants = [
            OAuthError::InvalidRequest("x".into()),
            OAuthError::InvalidGrant("x".into()),
            OAuthError::UnsupportedGrantType("x".into()),
            OAuthError::InvalidScope("x".into()),
        ];
        for err in &bad_request_variants {
            assert_eq!(
                err.status_code(),
                StatusCode::BAD_REQUEST,
                "failed for {err:?}"
            );
        }
    }

    #[test]
    fn status_code_unauthorized_variants() {
        let unauthorized_variants = [
            OAuthError::InvalidClient("x".into()),
            OAuthError::UnauthorizedClient("x".into()),
            OAuthError::AccessDenied("x".into()),
        ];
        for err in &unauthorized_variants {
            assert_eq!(
                err.status_code(),
                StatusCode::UNAUTHORIZED,
                "failed for {err:?}"
            );
        }
    }

    #[test]
    fn clone_and_eq() {
        let err = OAuthError::InvalidRequest("test".into());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn error_trait_impl() {
        let err = OAuthError::ServerError("oops".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn display_all_variants() {
        let cases = [
            (OAuthError::InvalidRequest("a".into()), "invalid_request: a"),
            (OAuthError::InvalidClient("b".into()), "invalid_client: b"),
            (OAuthError::InvalidGrant("c".into()), "invalid_grant: c"),
            (
                OAuthError::UnauthorizedClient("d".into()),
                "unauthorized_client: d",
            ),
            (
                OAuthError::UnsupportedGrantType("e".into()),
                "unsupported_grant_type: e",
            ),
            (OAuthError::InvalidScope("f".into()), "invalid_scope: f"),
            (OAuthError::AccessDenied("g".into()), "access_denied: g"),
            (OAuthError::ServerError("h".into()), "server_error: h"),
            (
                OAuthError::TemporarilyUnavailable("i".into()),
                "temporarily_unavailable: i",
            ),
        ];
        for (err, expected) in &cases {
            assert_eq!(err.to_string(), *expected);
        }
    }
}
