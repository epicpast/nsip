//! Data models for NSIP Search API.

use serde::{Deserialize, Serialize};

/// Search criteria for querying animals.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchCriteria {
    /// Breed group to filter by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breed_group: Option<String>,

    /// Animal status to filter by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Search query string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// Page number for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Number of results per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

impl SearchCriteria {
    /// Creates a new empty search criteria.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            breed_group: None,
            status: None,
            query: None,
            page: None,
            per_page: None,
        }
    }

    /// Sets the breed group filter.
    #[must_use]
    pub fn with_breed_group(mut self, breed_group: impl Into<String>) -> Self {
        self.breed_group = Some(breed_group.into());
        self
    }

    /// Sets the status filter.
    #[must_use]
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Sets the search query.
    #[must_use]
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Sets the page number.
    #[must_use]
    pub const fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the number of results per page.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

/// Represents a breed group in the NSIP system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreedGroup {
    /// Unique identifier for the breed group.
    pub id: String,

    /// Name of the breed group.
    pub name: String,

    /// Description of the breed group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Represents an animal in the NSIP system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animal {
    /// Unique identifier for the animal.
    pub id: String,

    /// Name or registration number of the animal.
    pub name: String,

    /// Breed of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breed: Option<String>,

    /// Breed group of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breed_group: Option<String>,

    /// Status of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Date of birth of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,

    /// Sex of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,

    /// Sire of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sire: Option<String>,

    /// Dam of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dam: Option<String>,
}

/// Represents animal lineage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// The animal ID.
    pub animal_id: String,

    /// Ancestors of the animal.
    pub ancestors: Vec<Animal>,
}

/// Represents animal progeny information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progeny {
    /// The animal ID.
    pub animal_id: String,

    /// Offspring of the animal.
    pub offspring: Vec<Animal>,
}

/// Represents a status option in the NSIP system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Unique identifier for the status.
    pub id: String,

    /// Name of the status.
    pub name: String,
}

/// Represents a trait range in the NSIP system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRange {
    /// Name of the trait.
    pub trait_name: String,

    /// Minimum value for the trait.
    pub min_value: f64,

    /// Maximum value for the trait.
    pub max_value: f64,

    /// Unit of measurement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// Response wrapper for search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// List of animals matching the search criteria.
    pub animals: Vec<Animal>,

    /// Total number of results.
    pub total: usize,

    /// Current page number.
    pub page: u32,

    /// Number of results per page.
    pub per_page: u32,
}
