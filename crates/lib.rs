#![doc = include_str!("../README.md")]
// miette-derive 7.6's generated `Diagnostic` impl for `Error` assigns to
// per-field locals it does not always read, tripping `unused_assignments` under
// the MSRV toolchain (1.92) with `-D warnings`. An item-level `#[allow]` on the
// enum does not reach the derive's generated impl on 1.92, so this is a
// crate-root inner attribute and is therefore CRATE-WIDE — it is not scoped to
// the generated code despite targeting it. Accepted trade-off: relocating
// `Error` into its own module to narrow the scope is disproportionate for a
// generated-code workaround, and `unused_assignments` rarely fires on
// hand-written code (the other dead-store lints remain active crate-wide).
#![allow(unused_assignments)]

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
// `ValidationKind` is defined below and re-exported here for a flat public path.

/// Boxed, thread-safe source error used to preserve the cause chain through
/// re-wraps (see [`std::error::Error::source`]).
type BoxSource = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Operation-specific classification for [`Error::Validation`].
///
/// Each kind selects a distinct RFC 9457 problem `type` URI and tailored
/// recovery guidance (see [`crate::problem`] and the error catalog under
/// `docs/reference/errors/`), so an agent can dispatch on the specific input
/// failure rather than a single generic "validation error".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationKind {
    /// An LPN ID argument was empty or blank.
    EmptyLpnId,
    /// A breed ID was non-positive or could not be parsed.
    InvalidBreedId,
    /// A pagination parameter (`page` / `page_size`) was out of range.
    PageRange,
    /// A search request carried no usable filter.
    EmptySearch,
    /// A comparison was given fewer than 2 or more than 5 animals.
    CompareArity,
    /// A required MCP argument was absent.
    MissingArgument,
    /// An MCP resource URI did not match any known resource or template.
    UnknownResource,
    /// An MCP pagination cursor could not be decoded or was out of range.
    InvalidCursor,
    /// An MCP transport name other than `stdio` / `http` was requested.
    UnknownTransport,
    /// Any other input validation failure (the generic fallback).
    Other,
}

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
    /// Invalid input parameters (validation failure). The `kind` selects the
    /// specific problem `type` and guidance; see [`ValidationKind`].
    #[error("validation error: {message}")]
    #[diagnostic(
        code("nsip::cli::validation"),
        url("{}/cli/validation.md", env!("NSIP_ERROR_TYPE_URI_BASE")),
        help("correct the input parameters and retry")
    )]
    Validation {
        /// Operation-specific classification selecting the problem `type`.
        kind: ValidationKind,
        /// Human-readable description of the validation failure.
        message: String,
    },

    /// The API returned a non-success HTTP status.
    #[error("API error (HTTP {status}): {message}")]
    #[diagnostic(
        code("nsip::api::error"),
        url("{}/api/error.md", env!("NSIP_ERROR_TYPE_URI_BASE"))
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
        url("{}/api/not-found.md", env!("NSIP_ERROR_TYPE_URI_BASE")),
        help("verify the identifier exists in the NSIP database")
    )]
    NotFound(String),

    /// The request timed out.
    #[error("request timed out: {message}")]
    #[diagnostic(
        code("nsip::api::timeout"),
        url("{}/api/timeout.md", env!("NSIP_ERROR_TYPE_URI_BASE")),
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
        url("{}/api/connection.md", env!("NSIP_ERROR_TYPE_URI_BASE")),
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
        url("{}/api/upstream-parse.md", env!("NSIP_ERROR_TYPE_URI_BASE"))
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
    /// Construct a generic [`Error::Validation`] ([`ValidationKind::Other`]).
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            kind: ValidationKind::Other,
            message: message.into(),
        }
    }

    /// Construct a [`Error::Validation`] with an explicit [`ValidationKind`].
    #[must_use]
    pub fn validation_kind(kind: ValidationKind, message: impl Into<String>) -> Self {
        Self::Validation {
            kind,
            message: message.into(),
        }
    }

    /// [`ValidationKind::EmptyLpnId`] — an LPN ID argument was empty.
    #[must_use]
    pub fn empty_lpn_id() -> Self {
        Self::validation_kind(ValidationKind::EmptyLpnId, "lpn_id cannot be empty")
    }

    /// [`ValidationKind::InvalidBreedId`] — a breed ID was non-positive or unparseable.
    #[must_use]
    pub fn invalid_breed_id(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::InvalidBreedId, message)
    }

    /// [`ValidationKind::PageRange`] — a pagination parameter was out of range.
    #[must_use]
    pub fn page_range(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::PageRange, message)
    }

    /// [`ValidationKind::EmptySearch`] — a search request carried no filter.
    #[must_use]
    pub fn empty_search(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::EmptySearch, message)
    }

    /// [`ValidationKind::CompareArity`] — comparison given <2 or >5 animals.
    #[must_use]
    pub fn compare_arity(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::CompareArity, message)
    }

    /// [`ValidationKind::MissingArgument`] — a required MCP argument was absent.
    #[must_use]
    pub fn missing_argument(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::MissingArgument, message)
    }

    /// [`ValidationKind::UnknownResource`] — an MCP resource URI was unknown.
    #[must_use]
    pub fn unknown_resource(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::UnknownResource, message)
    }

    /// [`ValidationKind::InvalidCursor`] — an MCP pagination cursor was bad.
    #[must_use]
    pub fn invalid_cursor(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::InvalidCursor, message)
    }

    /// [`ValidationKind::UnknownTransport`] — an unsupported MCP transport.
    #[must_use]
    pub fn unknown_transport(message: impl Into<String>) -> Self {
        Self::validation_kind(ValidationKind::UnknownTransport, message)
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
