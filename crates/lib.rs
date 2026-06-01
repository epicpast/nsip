#![doc = include_str!("../README.md")]

use miette::Diagnostic;
use thiserror::Error;

pub mod client;
pub mod mcp;
pub mod models;
pub mod problem;

pub use client::NsipClient;
pub use models::{
    AnimalDetails, AnimalProfile, Breed, BreedGroup, ContactInfo, DateLastUpdated, Lineage,
    LineageAnimal, Progeny, ProgenyAnimal, SearchCriteria, SearchResults, Trait, TraitRange,
    TraitRangeFilter,
};
pub use problem::{CodeAction, ProblemDetails, RetryAfter};

/// Boxed, thread-safe source error used to preserve the cause chain through
/// re-wraps (see [`std::error::Error::source`]).
type BoxSource = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Error type for NSIP operations.
///
/// Every variant derives a [`miette::Diagnostic`] code and a stable `type` URI,
/// and maps to an RFC 9457 Problem Details envelope via
/// [`Error::to_problem_details`]. The fallible-API variants ([`Error::Api`],
/// [`Error::Timeout`], [`Error::Connection`], [`Error::Parse`]) carry an
/// optional `#[source]` so the originating `reqwest`/`serde_json` error stays
/// inspectable; the transient variants also carry an optional `retry_after`.
///
/// Maps to the Python exception hierarchy:
/// - [`Error::Validation`] — `NSIPValidationError`
/// - [`Error::Api`] — `NSIPAPIError`
/// - [`Error::NotFound`] — `NSIPNotFoundError`
/// - [`Error::Timeout`] — `NSIPTimeoutError`
/// - [`Error::Connection`] — `NSIPConnectionError`
/// - [`Error::Parse`] — deserialization failures
#[derive(Error, Debug, Diagnostic)]
#[non_exhaustive]
pub enum Error {
    /// Invalid input parameters (validation failure).
    #[error("validation error: {0}")]
    #[diagnostic(
        code("nsip::cli::validation"),
        url("https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/validation.md"),
        help("correct the input parameters and retry")
    )]
    Validation(String),

    /// The API returned a non-success HTTP status.
    #[error("API error (HTTP {status}): {message}")]
    #[diagnostic(
        code("nsip::api::error"),
        url("https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/error.md")
    )]
    Api {
        /// HTTP status code returned by the upstream API.
        status: u16,
        /// Human-readable error message (the response body where available).
        message: String,
        /// Retry delay parsed from the upstream `Retry-After` header, for
        /// transient (429 / 5xx) responses.
        retry_after: Option<RetryAfter>,
        /// Originating transport error, preserved for the cause chain.
        #[source]
        source: Option<BoxSource>,
    },

    /// The requested resource was not found (HTTP 404).
    #[error("not found: {0}")]
    #[diagnostic(
        code("nsip::api::not-found"),
        url("https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/not-found.md"),
        help("verify the identifier exists in the NSIP database")
    )]
    NotFound(String),

    /// The request timed out.
    #[error("request timed out: {message}")]
    #[diagnostic(
        code("nsip::api::timeout"),
        url("https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/timeout.md"),
        help("retry the request; increase the client timeout if this persists")
    )]
    Timeout {
        /// Human-readable description of the timeout.
        message: String,
        /// Suggested retry delay, where one is known.
        retry_after: Option<RetryAfter>,
        /// Originating transport error, preserved for the cause chain.
        #[source]
        source: Option<BoxSource>,
    },

    /// Failed to connect to the API.
    #[error("connection error: {message}")]
    #[diagnostic(
        code("nsip::api::connection"),
        url("https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/connection.md"),
        help("check network connectivity to nsipsearch.nsip.org and retry")
    )]
    Connection {
        /// Human-readable description of the connection failure.
        message: String,
        /// Suggested retry delay, where one is known.
        retry_after: Option<RetryAfter>,
        /// Originating transport error, preserved for the cause chain.
        #[source]
        source: Option<BoxSource>,
    },

    /// Failed to parse the API response.
    #[error("parse error: {message}")]
    #[diagnostic(
        code("nsip::api::upstream-parse"),
        url(
            "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/upstream-parse.md"
        )
    )]
    Parse {
        /// Human-readable description of the parse failure.
        message: String,
        /// Originating deserialization error, preserved for the cause chain.
        #[source]
        source: Option<BoxSource>,
    },
}

impl Error {
    /// Construct a [`Error::Validation`] from any string-like message.
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Construct a [`Error::NotFound`] from any string-like message.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Construct a [`Error::Api`] with no retry hint or source.
    #[must_use]
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
            retry_after: None,
            source: None,
        }
    }

    /// Construct a [`Error::Timeout`] with no retry hint or source.
    #[must_use]
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
            retry_after: None,
            source: None,
        }
    }

    /// Construct a [`Error::Connection`] with no retry hint or source.
    #[must_use]
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            retry_after: None,
            source: None,
        }
    }

    /// Construct a [`Error::Parse`] with no source.
    #[must_use]
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            source: None,
        }
    }
}

/// Result type alias for NSIP operations.
pub type Result<T> = std::result::Result<T, Error>;
