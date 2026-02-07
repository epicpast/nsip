#![doc = include_str!("../README.md")]

use thiserror::Error;

pub mod client;
pub mod mcp;
pub mod models;

pub use client::NsipClient;
pub use models::{
    Animal, BreedGroup, Lineage, Progeny, SearchCriteria, SearchResponse, Status, TraitRange,
};

/// Error type for NSIP operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid input was provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// An API request failed.
    #[error("API error: {0}")]
    ApiError(String),

    /// Failed to parse API response.
    #[error("parse error: {0}")]
    ParseError(String),

    /// An operation failed.
    #[error("operation '{operation}' failed: {cause}")]
    OperationFailed {
        /// The operation that failed.
        operation: String,
        /// The underlying cause.
        cause: String,
    },
}

/// Result type alias for NSIP operations.
pub type Result<T> = std::result::Result<T, Error>;
