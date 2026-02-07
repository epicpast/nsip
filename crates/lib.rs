#![doc = include_str!("../README.md")]

use thiserror::Error;

pub mod client;
pub mod mcp;
pub mod models;

pub use client::NsipClient;
pub use models::{
    AnimalDetails, AnimalProfile, Breed, BreedGroup, ContactInfo, DateLastUpdated, Lineage,
    LineageAnimal, Progeny, ProgenyAnimal, SearchCriteria, SearchResults, Trait, TraitRange,
    TraitRangeFilter,
};

/// Error type for NSIP operations.
///
/// Maps to the Python exception hierarchy:
/// - [`Error::Validation`] — `NSIPValidationError`
/// - [`Error::Api`] — `NSIPAPIError`
/// - [`Error::NotFound`] — `NSIPNotFoundError`
/// - [`Error::Timeout`] — `NSIPTimeoutError`
/// - [`Error::Connection`] — `NSIPConnectionError`
/// - [`Error::Parse`] — deserialization failures
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid input parameters (validation failure).
    #[error("validation error: {0}")]
    Validation(String),

    /// The API returned a non-success HTTP status.
    #[error("API error (HTTP {status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Human-readable error message.
        message: String,
    },

    /// The requested resource was not found (HTTP 404).
    #[error("not found: {0}")]
    NotFound(String),

    /// The request timed out.
    #[error("request timed out: {0}")]
    Timeout(String),

    /// Failed to connect to the API.
    #[error("connection error: {0}")]
    Connection(String),

    /// Failed to parse the API response.
    #[error("parse error: {0}")]
    Parse(String),
}

/// Result type alias for NSIP operations.
pub type Result<T> = std::result::Result<T, Error>;
