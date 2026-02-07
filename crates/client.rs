//! NSIP Search API client implementation.

use crate::{
    Error, Result,
    models::{
        Animal, BreedGroup, Lineage, Progeny, SearchCriteria, SearchResponse, Status, TraitRange,
    },
};

/// Base URL for the NSIP Search API.
const API_BASE_URL: &str = "https://nsipsearch.nsip.org/api";

/// Client for interacting with the NSIP Search API.
#[derive(Debug, Clone)]
pub struct NsipClient {
    /// HTTP client for making requests.
    client: reqwest::Client,

    /// Base URL for the API.
    base_url: String,
}

impl Default for NsipClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NsipClient {
    /// Creates a new NSIP API client with the default base URL.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nsip::NsipClient;
    ///
    /// let client = NsipClient::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: API_BASE_URL.to_string(),
        }
    }

    /// Creates a new NSIP API client with a custom base URL.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nsip::NsipClient;
    ///
    /// let client = NsipClient::with_base_url("https://custom.api.url");
    /// ```
    #[must_use]
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Retrieves the list of available breed groups.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nsip::NsipClient;
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = NsipClient::new();
    /// let breed_groups = client.breed_groups().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn breed_groups(&self) -> Result<Vec<BreedGroup>> {
        let url = format!("{}/breed_groups", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to fetch breed groups: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ParseError(format!("Failed to parse breed groups: {e}")))
    }

    /// Retrieves the list of available animal statuses.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nsip::NsipClient;
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = NsipClient::new();
    /// let statuses = client.statuses().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn statuses(&self) -> Result<Vec<Status>> {
        let url = format!("{}/statuses", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to fetch statuses: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ParseError(format!("Failed to parse statuses: {e}")))
    }

    /// Retrieves the available trait ranges.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nsip::NsipClient;
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = NsipClient::new();
    /// let trait_ranges = client.trait_ranges().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn trait_ranges(&self) -> Result<Vec<TraitRange>> {
        let url = format!("{}/trait_ranges", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to fetch trait ranges: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ParseError(format!("Failed to parse trait ranges: {e}")))
    }

    /// Searches for animals based on the provided criteria.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The search criteria to filter animals.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nsip::{NsipClient, SearchCriteria};
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = NsipClient::new();
    /// let criteria = SearchCriteria::new()
    ///     .with_breed_group("Sheep")
    ///     .with_page(1)
    ///     .with_per_page(20);
    /// let results = client.search_animals(&criteria).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_animals(&self, criteria: &SearchCriteria) -> Result<SearchResponse> {
        let url = format!("{}/search", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(criteria)
            .send()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to search animals: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ParseError(format!("Failed to parse search results: {e}")))
    }

    /// Retrieves detailed information about a specific animal.
    ///
    /// # Arguments
    ///
    /// * `animal_id` - The unique identifier of the animal.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nsip::NsipClient;
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = NsipClient::new();
    /// let animal = client.details("12345").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn details(&self, animal_id: &str) -> Result<Animal> {
        let url = format!("{}/animals/{animal_id}", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to fetch animal details: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ParseError(format!("Failed to parse animal details: {e}")))
    }

    /// Retrieves the lineage (ancestry) of a specific animal.
    ///
    /// # Arguments
    ///
    /// * `animal_id` - The unique identifier of the animal.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nsip::NsipClient;
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = NsipClient::new();
    /// let lineage = client.lineage("12345").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lineage(&self, animal_id: &str) -> Result<Lineage> {
        let url = format!("{}/animals/{animal_id}/lineage", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to fetch lineage: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ParseError(format!("Failed to parse lineage: {e}")))
    }

    /// Retrieves the progeny (offspring) of a specific animal.
    ///
    /// # Arguments
    ///
    /// * `animal_id` - The unique identifier of the animal.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nsip::NsipClient;
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = NsipClient::new();
    /// let progeny = client.progeny("12345").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn progeny(&self, animal_id: &str) -> Result<Progeny> {
        let url = format!("{}/animals/{animal_id}/progeny", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to fetch progeny: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::ParseError(format!("Failed to parse progeny: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = NsipClient::new();
        assert_eq!(client.base_url, API_BASE_URL);
    }

    #[test]
    fn test_client_with_custom_url() {
        let custom_url = "https://custom.api.url";
        let client = NsipClient::with_base_url(custom_url);
        assert_eq!(client.base_url, custom_url);
    }

    #[test]
    fn test_search_criteria_builder() {
        let criteria = SearchCriteria::new()
            .with_breed_group("Sheep")
            .with_status("Active")
            .with_query("test")
            .with_page(2)
            .with_per_page(50);

        assert_eq!(criteria.breed_group, Some("Sheep".to_string()));
        assert_eq!(criteria.status, Some("Active".to_string()));
        assert_eq!(criteria.query, Some("test".to_string()));
        assert_eq!(criteria.page, Some(2));
        assert_eq!(criteria.per_page, Some(50));
    }
}
