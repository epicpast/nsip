//! NSIP Search API client implementation.
//!
//! HTTP client for `nsipsearch.nsip.org/api`. The upstream API is HTTP-only
//! (no valid TLS certificate), so the default base URL uses `http://`.

use std::time::Duration;

use reqwest::StatusCode;

use crate::{
    Error, Result,
    models::{
        AnimalDetails, AnimalProfile, Breed, BreedGroup, DateLastUpdated, Lineage, Progeny,
        RawBreedGroupResponse, SearchCriteria, SearchResults,
    },
};

/// Default base URL for the NSIP Search API (HTTP-only).
const DEFAULT_BASE_URL: &str = "http://nsipsearch.nsip.org/api";

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default maximum number of retries for server errors.
const DEFAULT_MAX_RETRIES: u32 = 3;

/// HTTP status codes that trigger an automatic retry.
const RETRY_STATUS_CODES: &[u16] = &[500, 502, 503, 504];

/// Backoff factor — retry delay is `factor * 2^attempt` seconds.
const BACKOFF_FACTOR: f64 = 0.5;

/// Client for the NSIP Search API at `nsipsearch.nsip.org/api`.
///
/// All methods are `async` and require a Tokio runtime.
///
/// # Examples
///
/// ```rust,no_run
/// # async fn example() -> Result<(), nsip::Error> {
/// use nsip::NsipClient;
///
/// let client = NsipClient::builder()
///     .timeout_secs(60)
///     .max_retries(5)
///     .build()?;
///
/// let groups = client.breed_groups().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NsipClient {
    http: reqwest::Client,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

/// Builder for [`NsipClient`].
#[derive(Debug, Clone)]
pub struct NsipClientBuilder {
    base_url: Option<String>,
    timeout_secs: u64,
    max_retries: u32,
}

impl Default for NsipClientBuilder {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
}

impl NsipClientBuilder {
    /// Sets the API base URL (defaults to `http://nsipsearch.nsip.org/api`).
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Sets the per-request timeout in seconds (default: 30).
    #[must_use]
    pub const fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Sets the maximum number of retries on server errors (default: 3).
    #[must_use]
    pub const fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Builds the [`NsipClient`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the HTTP client cannot be constructed.
    pub fn build(self) -> Result<NsipClient> {
        let timeout = Duration::from_secs(self.timeout_secs);
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("NSIP-Rust-Client/0.1")
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::ACCEPT,
                    "application/json, text/plain, */*"
                        .parse()
                        .map_err(|e| Error::Connection(format!("invalid header: {e}")))?,
                );
                h
            })
            .build()
            .map_err(|e| Error::Connection(format!("failed to build HTTP client: {e}")))?;

        Ok(NsipClient {
            http,
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            timeout,
            max_retries: self.max_retries,
        })
    }
}

impl Default for NsipClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NsipClient {
    /// Creates a new client with default settings.
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
        Self::builder().build().unwrap_or_else(|_| Self {
            http: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
        })
    }

    /// Creates a new client with a custom base URL (convenience constructor).
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
    /// let client = NsipClient::with_base_url("http://localhost:8080/api");
    /// ```
    #[must_use]
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self::builder()
            .base_url(base_url)
            .build()
            .unwrap_or_else(|_| Self {
                http: reqwest::Client::new(),
                base_url: DEFAULT_BASE_URL.to_string(),
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                max_retries: DEFAULT_MAX_RETRIES,
            })
    }

    /// Returns a builder for configuring the client.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), nsip::Error> {
    /// use nsip::NsipClient;
    ///
    /// let client = NsipClient::builder()
    ///     .timeout_secs(60)
    ///     .max_retries(5)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn builder() -> NsipClientBuilder {
        NsipClientBuilder::default()
    }

    /// Returns the configured base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // ---------------------------------------------------------------------
    // Internal HTTP helpers
    // ---------------------------------------------------------------------

    /// Execute a GET request with retry logic.
    async fn get(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<serde_json::Value> {
        let url = format!("{}/{endpoint}", self.base_url);
        self.request_with_retry(reqwest::Method::GET, &url, params, None)
            .await
    }

    /// Execute a POST request with retry logic.
    async fn post(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/{endpoint}", self.base_url);
        self.request_with_retry(reqwest::Method::POST, &url, params, Some(body))
            .await
    }

    /// Core request method with configurable retry and exponential backoff.
    async fn request_with_retry(
        &self,
        method: reqwest::Method,
        url: &str,
        params: &[(&str, &str)],
        body: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut last_err: Option<Error> = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay_secs = BACKOFF_FACTOR * f64::from(1u32 << (attempt - 1));
                tokio::time::sleep(Duration::from_secs_f64(delay_secs)).await;
            }

            let err = match self.do_request(&method, url, params, body).await {
                Ok(value) => return Ok(value),
                Err(e) => e,
            };

            if Self::is_retryable(&err) && attempt < self.max_retries {
                last_err = Some(err);
                continue;
            }
            return Err(err);
        }

        Err(last_err.unwrap_or_else(|| Error::Connection("max retries exceeded".to_string())))
    }

    /// Single HTTP request (no retry).
    async fn do_request(
        &self,
        method: &reqwest::Method,
        url: &str,
        params: &[(&str, &str)],
        body: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut builder = self.http.request(method.clone(), url).query(params);

        if let Some(json_body) = body {
            builder = builder.json(json_body);
        }

        let response = builder.send().await.map_err(|e| {
            if e.is_timeout() {
                Error::Timeout(format!(
                    "request timed out after {}s: {e}",
                    self.timeout.as_secs()
                ))
            } else if e.is_connect() {
                Error::Connection(format!("failed to connect to API: {e}"))
            } else {
                Error::Connection(format!("request failed: {e}"))
            }
        })?;

        let status = response.status();

        if status == StatusCode::NOT_FOUND {
            return Err(Error::NotFound(format!("resource not found at {url}")));
        }

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: if text.is_empty() {
                    format!("HTTP {status}")
                } else {
                    text
                },
            });
        }

        response
            .json()
            .await
            .map_err(|e| Error::Parse(format!("failed to parse response: {e}")))
    }

    /// Determine whether an error warrants a retry.
    fn is_retryable(err: &Error) -> bool {
        match err {
            Error::Api { status, .. } => RETRY_STATUS_CODES.contains(status),
            Error::Timeout(_) | Error::Connection(_) => true,
            _ => false,
        }
    }

    /// Parse a single [`Breed`] from a raw JSON value with flexible field names.
    fn parse_breed_value(b: &serde_json::Value) -> Breed {
        let id = b
            .get("breedId")
            .or_else(|| b.get("id"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let name = b
            .get("breedName")
            .or_else(|| b.get("name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        Breed { id, name }
    }

    // ---------------------------------------------------------------------
    // Public API methods
    // ---------------------------------------------------------------------

    /// Returns the date when the NSIP database was last updated.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let updated = client.date_last_updated().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn date_last_updated(&self) -> Result<DateLastUpdated> {
        let data = self.get("search/getDateLastUpdated", &[]).await?;
        Ok(DateLastUpdated { data })
    }

    /// Returns the list of available breed groups with their breeds.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let groups = client.breed_groups().await?;
    /// for g in &groups {
    ///     println!("{}: {}", g.id, g.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn breed_groups(&self) -> Result<Vec<BreedGroup>> {
        let data = self.get("search/getAvailableBreedGroups", &[]).await?;

        // The API may wrap the list in {"success": true, "data": [...]}
        let raw_groups: Vec<serde_json::Value> =
            if let Ok(wrapper) = serde_json::from_value::<RawBreedGroupResponse>(data.clone()) {
                if let Some(groups) = wrapper.data {
                    return Ok(groups
                        .into_iter()
                        .map(|g| BreedGroup {
                            id: g.breed_group_id.unwrap_or(0),
                            name: g.breed_group_name.unwrap_or_default(),
                            breeds: g
                                .breeds
                                .into_iter()
                                .map(|b| Breed {
                                    id: b.breed_id.unwrap_or(0),
                                    name: b.breed_name.unwrap_or_default(),
                                })
                                .collect(),
                        })
                        .collect());
                }
                // Fallback: try as direct array
                serde_json::from_value(data)
                    .map_err(|e| Error::Parse(format!("failed to parse breed groups: {e}")))?
            } else {
                serde_json::from_value(data)
                    .map_err(|e| Error::Parse(format!("failed to parse breed groups: {e}")))?
            };

        // Direct array of raw objects — normalize field names
        Ok(raw_groups
            .into_iter()
            .map(|g| {
                let id = g
                    .get("breedGroupId")
                    .or_else(|| g.get("Id"))
                    .or_else(|| g.get("id"))
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                let name = g
                    .get("breedGroupName")
                    .or_else(|| g.get("Name"))
                    .or_else(|| g.get("name"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let breeds = g
                    .get("breeds")
                    .and_then(serde_json::Value::as_array)
                    .map(|arr| arr.iter().map(Self::parse_breed_value).collect())
                    .unwrap_or_default();
                BreedGroup { id, name, breeds }
            })
            .collect())
    }

    /// Returns the list of available animal statuses.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let statuses = client.statuses().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn statuses(&self) -> Result<Vec<String>> {
        let data = self.get("search/getStatusesByBreedGroup", &[]).await?;
        Ok(data.as_array().map_or_else(Vec::new, |arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        }))
    }

    /// Returns trait value ranges for a specific breed.
    ///
    /// # Arguments
    ///
    /// * `breed_id` - The breed identifier (must be > 0).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `breed_id` is not positive.
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let ranges = client.trait_ranges(486).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn trait_ranges(&self, breed_id: i64) -> Result<serde_json::Value> {
        if breed_id <= 0 {
            return Err(Error::Validation(format!("invalid breed_id: {breed_id}")));
        }

        let id_str = breed_id.to_string();
        self.get("search/getTraitRangesByBreed", &[("breedId", &id_str)])
            .await
    }

    /// Searches for animals matching the given criteria.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number (0-indexed).
    /// * `page_size` - Results per page (1–100).
    /// * `breed_id` - Optional breed filter.
    /// * `sorted_trait` - Optional trait to sort by (e.g. `"BWT"`).
    /// * `reverse` - Sort in reverse order.
    /// * `criteria` - Optional [`SearchCriteria`] body.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] for invalid `page`/`page_size` values.
    /// Returns an error if the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// use nsip::{NsipClient, SearchCriteria};
    ///
    /// let client = NsipClient::new();
    /// let criteria = SearchCriteria::new().with_breed_id(486);
    /// let results = client
    ///     .search_animals(0, 15, Some(486), None, None, Some(&criteria))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub async fn search_animals(
        &self,
        page: u32,
        page_size: u32,
        breed_id: Option<i64>,
        sorted_trait: Option<&str>,
        reverse: Option<bool>,
        criteria: Option<&SearchCriteria>,
    ) -> Result<SearchResults> {
        if page_size == 0 || page_size > 100 {
            return Err(Error::Validation(format!(
                "page_size must be 1-100, got {page_size}"
            )));
        }

        let page_str = page.to_string();
        let page_size_str = page_size.to_string();
        let mut params: Vec<(&str, String)> = vec![("page", page_str), ("pageSize", page_size_str)];

        if let Some(bid) = breed_id {
            params.push(("breedId", bid.to_string()));
        }
        if let Some(trait_name) = sorted_trait {
            params.push(("sortedBreedTrait", trait_name.to_string()));
        }
        if let Some(rev) = reverse {
            params.push(("reverse", rev.to_string()));
        }

        let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let body = criteria.map_or_else(
            || serde_json::json!({}),
            |c| serde_json::to_value(c).unwrap_or_default(),
        );

        let data = self
            .post("search/getPageOfSearchResults", &param_refs, &body)
            .await?;

        SearchResults::from_api_response(&data, page, page_size)
    }

    /// Returns detailed information about a specific animal.
    ///
    /// # Arguments
    ///
    /// * `search_string` - LPN ID or registration number.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `search_string` is empty.
    /// Returns [`Error::NotFound`] if the animal is not found.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let details = client.animal_details("6####92020###249").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn animal_details(&self, search_string: &str) -> Result<AnimalDetails> {
        if search_string.trim().is_empty() {
            return Err(Error::Validation(
                "search_string cannot be empty".to_string(),
            ));
        }

        let data = self
            .get(
                "details/getAnimalDetails",
                &[("searchString", search_string)],
            )
            .await?;

        AnimalDetails::from_api_response(&data)
    }

    /// Returns the pedigree / lineage tree for an animal.
    ///
    /// # Arguments
    ///
    /// * `lpn_id` - The LPN identifier.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `lpn_id` is empty.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let lineage = client.lineage("6####92020###249").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lineage(&self, lpn_id: &str) -> Result<Lineage> {
        if lpn_id.trim().is_empty() {
            return Err(Error::Validation("lpn_id cannot be empty".to_string()));
        }

        let data = self.get("details/getLineage", &[("lpnId", lpn_id)]).await?;

        Lineage::from_api_response(&data)
    }

    /// Returns paginated progeny (offspring) for an animal.
    ///
    /// # Arguments
    ///
    /// * `lpn_id` - The LPN identifier.
    /// * `page` - Page number (0-indexed).
    /// * `page_size` - Results per page (must be >= 1).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] for invalid parameters.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let progeny = client.progeny("6####92020###249", 0, 10).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn progeny(&self, lpn_id: &str, page: u32, page_size: u32) -> Result<Progeny> {
        if lpn_id.trim().is_empty() {
            return Err(Error::Validation("lpn_id cannot be empty".to_string()));
        }
        if page_size == 0 {
            return Err(Error::Validation(format!(
                "page_size must be >= 1, got {page_size}"
            )));
        }

        let page_str = page.to_string();
        let page_size_str = page_size.to_string();
        let data = self
            .get(
                "details/getPageOfProgeny",
                &[
                    ("lpnId", lpn_id),
                    ("page", &page_str),
                    ("pageSize", &page_size_str),
                ],
            )
            .await?;

        Progeny::from_api_response(&data, page, page_size)
    }

    /// Fetches details, lineage, and progeny for an animal concurrently.
    ///
    /// This is equivalent to Python's `search_by_lpn` — three independent API
    /// calls are executed in parallel via `tokio::join!`.
    ///
    /// # Arguments
    ///
    /// * `lpn_id` - The LPN identifier.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered from any of the three requests.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), nsip::Error> {
    /// let client = nsip::NsipClient::new();
    /// let profile = client.search_by_lpn("6####92020###249").await?;
    /// println!("Breed: {:?}", profile.details.breed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_by_lpn(&self, lpn_id: &str) -> Result<AnimalProfile> {
        if lpn_id.trim().is_empty() {
            return Err(Error::Validation("lpn_id cannot be empty".to_string()));
        }

        let (details_res, lineage_res, progeny_res) = tokio::join!(
            self.animal_details(lpn_id),
            self.lineage(lpn_id),
            self.progeny(lpn_id, 0, 10),
        );

        Ok(AnimalProfile {
            details: details_res?,
            lineage: lineage_res?,
            progeny: progeny_res?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation_default() {
        let client = NsipClient::new();
        assert_eq!(client.base_url(), DEFAULT_BASE_URL);
    }

    #[test]
    fn client_with_custom_url() {
        let client = NsipClient::with_base_url("http://localhost:9999");
        assert_eq!(client.base_url(), "http://localhost:9999");
    }

    #[test]
    fn builder_configures_all_fields() {
        let client = NsipClient::builder()
            .base_url("http://test.local/api")
            .timeout_secs(60)
            .max_retries(5)
            .build()
            .unwrap();

        assert_eq!(client.base_url(), "http://test.local/api");
        assert_eq!(client.timeout, Duration::from_secs(60));
        assert_eq!(client.max_retries, 5);
    }

    #[test]
    fn retryable_errors() {
        assert!(NsipClient::is_retryable(&Error::Api {
            status: 500,
            message: String::new(),
        }));
        assert!(NsipClient::is_retryable(&Error::Api {
            status: 502,
            message: String::new(),
        }));
        assert!(NsipClient::is_retryable(&Error::Timeout("t".to_string())));
        assert!(NsipClient::is_retryable(&Error::Connection(
            "c".to_string()
        )));
        assert!(!NsipClient::is_retryable(&Error::NotFound("n".to_string())));
        assert!(!NsipClient::is_retryable(&Error::Validation(
            "v".to_string()
        )));
        assert!(!NsipClient::is_retryable(&Error::Api {
            status: 400,
            message: String::new(),
        }));
    }

    #[test]
    fn validation_empty_search_string() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let client = NsipClient::new();

            let err = client.animal_details("").await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));

            let err = client.animal_details("   ").await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));

            let err = client.lineage("").await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));

            let err = client.progeny("", 0, 10).await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));
        });
    }

    #[test]
    fn validation_page_size() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let client = NsipClient::new();

            let err = client
                .search_animals(0, 0, None, None, None, None)
                .await
                .unwrap_err();
            assert!(matches!(err, Error::Validation(_)));

            let err = client
                .search_animals(0, 101, None, None, None, None)
                .await
                .unwrap_err();
            assert!(matches!(err, Error::Validation(_)));
        });
    }

    #[test]
    fn validation_breed_id() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let client = NsipClient::new();
            let err = client.trait_ranges(0).await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));

            let err = client.trait_ranges(-5).await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));
        });
    }
}
