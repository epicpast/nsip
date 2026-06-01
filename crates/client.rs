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
    problem::RetryAfter,
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

/// Upper bound (seconds) on an honored `Retry-After` delay, so a hostile or
/// misconfigured upstream cannot stall the client indefinitely.
const MAX_RETRY_DELAY_SECS: u64 = 60;

/// Parse the upstream `Retry-After` header (RFC 7231 §7.1.3) into a
/// [`RetryAfter`]. Handles the delta-seconds form directly; the HTTP-date form
/// is converted to an RFC 3339 timestamp (GMT normalized to `+0000` for the
/// RFC 2822 parser). Returns `None` when the header is absent or unparseable.
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<RetryAfter> {
    let raw = headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim();
    if let Ok(secs) = raw.parse::<u32>() {
        return Some(RetryAfter::Seconds(secs));
    }
    let normalized = raw.replace("GMT", "+0000");
    chrono::DateTime::parse_from_rfc2822(&normalized)
        .ok()
        .map(|dt| RetryAfter::Timestamp(dt.to_rfc3339()))
}

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
                        .map_err(|e| Error::connection(format!("invalid header: {e}")))?,
                );
                h
            })
            .build()
            .map_err(|e| Error::connection(format!("failed to build HTTP client: {e}")))?;

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
        // This constructor is infallible by contract (used by `Default` and
        // pervasively), so a `Result` is not surfaced here. The only realistic
        // `build()` failure is TLS-backend init — which `reqwest::Client::new()`
        // would also hit — so the fallback is genuinely best-effort. Callers
        // that must observe a construction failure use [`NsipClient::builder`],
        // whose `build()` returns a `Result`.
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
        // Infallible by contract; see [`NsipClient::new`] for why the fallback
        // is best-effort. The fallback preserves the caller's `base_url` (it is
        // not silently replaced with the production default).
        let base = base_url.into();
        Self::builder()
            .base_url(base.clone())
            .build()
            .unwrap_or_else(|_| Self {
                http: reqwest::Client::new(),
                base_url: base,
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
                tokio::time::sleep(Self::retry_delay(attempt, last_err.as_ref())).await;
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

        Err(last_err.unwrap_or_else(|| Error::connection("max retries exceeded")))
    }

    /// Compute the delay before the next retry attempt. Honors an upstream
    /// `Retry-After` (delta-seconds) hint from the previous error when present,
    /// capped at [`MAX_RETRY_DELAY_SECS`]; otherwise falls back to exponential
    /// backoff (`BACKOFF_FACTOR * 2^(attempt-1)`).
    fn retry_delay(attempt: u32, last_err: Option<&Error>) -> Duration {
        if let Some(RetryAfter::Seconds(secs)) = last_err.and_then(Error::retry_after) {
            return Duration::from_secs(u64::from(secs).min(MAX_RETRY_DELAY_SECS));
        }
        let delay_secs = BACKOFF_FACTOR * f64::from(1u32 << (attempt - 1));
        Duration::from_secs_f64(delay_secs)
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
                Error::Timeout {
                    message: format!("request timed out after {}s: {e}", self.timeout.as_secs()),
                    retry_after: None,
                    source: Some(Box::new(e)),
                }
            } else if e.is_connect() {
                Error::Connection {
                    message: format!("failed to connect to API: {e}"),
                    retry_after: None,
                    source: Some(Box::new(e)),
                }
            } else {
                Error::Connection {
                    message: format!("request failed: {e}"),
                    retry_after: None,
                    source: Some(Box::new(e)),
                }
            }
        })?;

        let status = response.status();

        if status == StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("resource not found at {url}")));
        }

        if !status.is_success() {
            let code = status.as_u16();
            // Capture the upstream Retry-After before consuming the body so
            // transient (429 / 5xx) errors carry an actionable delay.
            let retry_after = parse_retry_after(response.headers());
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: code,
                message: if text.is_empty() {
                    format!("HTTP {status}")
                } else {
                    text
                },
                retry_after,
                source: None,
            });
        }

        response.json().await.map_err(|e| Error::Parse {
            message: format!("failed to parse response: {e}"),
            source: Some(Box::new(e)),
        })
    }

    /// Determine whether an error warrants a retry. Transient transport errors
    /// and rate-limit (429) / retryable 5xx responses qualify.
    fn is_retryable(err: &Error) -> bool {
        match err {
            Error::Api { status, .. } => *status == 429 || RETRY_STATUS_CODES.contains(status),
            Error::Timeout { .. } | Error::Connection { .. } => true,
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
                    .map_err(|e| Error::parse(format!("failed to parse breed groups: {e}")))?
            } else {
                serde_json::from_value(data)
                    .map_err(|e| Error::parse(format!("failed to parse breed groups: {e}")))?
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
        assert!(NsipClient::is_retryable(&Error::api(500, "")));
        assert!(NsipClient::is_retryable(&Error::api(502, "")));
        assert!(NsipClient::is_retryable(&Error::api(429, "")));
        assert!(NsipClient::is_retryable(&Error::timeout("t")));
        assert!(NsipClient::is_retryable(&Error::connection("c")));
        assert!(!NsipClient::is_retryable(&Error::not_found("n")));
        assert!(!NsipClient::is_retryable(&Error::validation("v")));
        assert!(!NsipClient::is_retryable(&Error::api(400, "")));
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

    // ------------------------------------------------------------------
    // Wiremock-based async integration tests
    // ------------------------------------------------------------------

    mod wiremock_tests {
        use super::*;
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path, query_param},
        };

        /// Build a client pointing at the given mock server with retries disabled.
        fn mock_client(uri: &str) -> NsipClient {
            NsipClient::builder()
                .base_url(uri)
                .max_retries(0)
                .build()
                .unwrap()
        }

        // -- date_last_updated ------------------------------------------------

        #[tokio::test]
        async fn date_last_updated_returns_value() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(serde_json::json!("2024-12-15")),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let result = client.date_last_updated().await.unwrap();
            assert_eq!(result.data, serde_json::json!("2024-12-15"));
        }

        // -- breed_groups -----------------------------------------------------

        #[tokio::test]
        async fn breed_groups_wrapper_format() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getAvailableBreedGroups"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "success": true,
                    "data": [
                        {
                            "breedGroupId": 61,
                            "breedGroupName": "Range",
                            "breeds": [
                                { "breedId": 486, "breedName": "SA Meat Merino" },
                                { "breedId": 640, "breedName": "Katahdin" }
                            ]
                        }
                    ]
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let groups = client.breed_groups().await.unwrap();
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].id, 61);
            assert_eq!(groups[0].name, "Range");
            assert_eq!(groups[0].breeds.len(), 2);
            assert_eq!(groups[0].breeds[0].id, 486);
            assert_eq!(groups[0].breeds[0].name, "SA Meat Merino");
        }

        #[tokio::test]
        async fn breed_groups_direct_array_format() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getAvailableBreedGroups"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    {
                        "breedGroupId": 10,
                        "breedGroupName": "Wool",
                        "breeds": [
                            { "breedId": 100, "breedName": "Merino" }
                        ]
                    }
                ])))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let groups = client.breed_groups().await.unwrap();
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].id, 10);
            assert_eq!(groups[0].name, "Wool");
            assert_eq!(groups[0].breeds[0].name, "Merino");
        }

        // -- statuses ---------------------------------------------------------

        #[tokio::test]
        async fn statuses_returns_list() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getStatusesByBreedGroup"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(serde_json::json!(["CURRENT", "SOLD", "DEAD"])),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let statuses = client.statuses().await.unwrap();
            assert_eq!(statuses, vec!["CURRENT", "SOLD", "DEAD"]);
        }

        #[tokio::test]
        async fn statuses_non_array_returns_empty() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getStatusesByBreedGroup"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(serde_json::json!("not-an-array")),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let statuses = client.statuses().await.unwrap();
            assert!(statuses.is_empty());
        }

        // -- trait_ranges -----------------------------------------------------

        #[tokio::test]
        async fn trait_ranges_returns_json() {
            let server = MockServer::start().await;
            let body = serde_json::json!([
                { "traitName": "BWT", "min": -2.0, "max": 3.0 },
                { "traitName": "WWT", "min": 0.0, "max": 20.0 }
            ]);
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .and(query_param("breedId", "486"))
                .respond_with(ResponseTemplate::new(200).set_body_json(&body))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let result = client.trait_ranges(486).await.unwrap();
            assert_eq!(result, body);
        }

        // -- search_animals ---------------------------------------------------

        #[tokio::test]
        async fn search_animals_basic() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .and(query_param("page", "0"))
                .and(query_param("pageSize", "15"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "TotalCount": 42,
                    "Results": [
                        { "lpnId": "A1", "bwt": 0.5, "accbwt": 80 },
                        { "lpnId": "A2", "bwt": -0.3, "accbwt": 70 }
                    ]
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let results = client
                .search_animals(0, 15, None, None, None, None)
                .await
                .unwrap();
            assert_eq!(results.total_count, 42);
            assert_eq!(results.results.len(), 2);
            assert_eq!(results.page, 0);
            assert_eq!(results.page_size, 15);
        }

        #[tokio::test]
        async fn search_animals_with_criteria() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .and(query_param("breedId", "486"))
                .and(query_param("sortedBreedTrait", "BWT"))
                .and(query_param("reverse", "true"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "TotalCount": 10,
                    "Results": [{ "lpnId": "B1" }]
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let criteria = crate::SearchCriteria::new().with_breed_id(486);
            let results = client
                .search_animals(0, 15, Some(486), Some("BWT"), Some(true), Some(&criteria))
                .await
                .unwrap();
            assert_eq!(results.total_count, 10);
            assert_eq!(results.results.len(), 1);
        }

        // -- animal_details ---------------------------------------------------

        #[tokio::test]
        async fn animal_details_nested_format() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "LPN123"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "success": true,
                    "data": {
                        "progenyCount": 5,
                        "dateOfBirth": "01/15/2020",
                        "gender": "Female",
                        "breed": { "breedName": "Katahdin", "breedId": 640 },
                        "searchResultViewModel": {
                            "lpnId": "LPN123",
                            "status": "CURRENT",
                            "bwt": 0.3,
                            "accbwt": 0.75
                        }
                    }
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let details = client.animal_details("LPN123").await.unwrap();
            assert_eq!(details.lpn_id, "LPN123");
            assert_eq!(details.breed.as_deref(), Some("Katahdin"));
            assert_eq!(details.gender.as_deref(), Some("Female"));
            assert_eq!(details.total_progeny, Some(5));
            let bwt = details.traits.get("BWT").unwrap();
            assert!((bwt.value - 0.3).abs() < f64::EPSILON);
        }

        // -- lineage ----------------------------------------------------------

        #[tokio::test]
        async fn lineage_with_ancestors() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "LPN123"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "LPN123",
                    "content": "<div>Farm A</div><div>DOB: 1/1/2020</div>",
                    "children": [
                        {
                            "lpnId": "SIRE1",
                            "content": "<div>Sire Farm</div>",
                            "children": []
                        },
                        {
                            "lpnId": "DAM1",
                            "content": "<div>Dam Farm</div>",
                            "children": []
                        }
                    ]
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let lineage = client.lineage("LPN123").await.unwrap();
            let subject = lineage.subject.unwrap();
            assert_eq!(subject.lpn_id, "LPN123");
            assert_eq!(subject.farm_name.as_deref(), Some("Farm A"));
            assert_eq!(lineage.sire.unwrap().lpn_id, "SIRE1");
            assert_eq!(lineage.dam.unwrap().lpn_id, "DAM1");
        }

        // -- progeny ----------------------------------------------------------

        #[tokio::test]
        async fn progeny_pagination() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .and(query_param("lpnId", "LPN123"))
                .and(query_param("page", "0"))
                .and(query_param("pageSize", "5"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "recordCount": 12,
                    "records": [
                        { "lpnId": "P1", "sex": "Male", "dob": "03/10/2022" },
                        { "lpnId": "P2", "sex": "Female", "dob": "03/11/2022" }
                    ]
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let progeny = client.progeny("LPN123", 0, 5).await.unwrap();
            assert_eq!(progeny.total_count, 12);
            assert_eq!(progeny.animals.len(), 2);
            assert_eq!(progeny.animals[0].lpn_id, "P1");
            assert_eq!(progeny.animals[0].sex.as_deref(), Some("Male"));
            assert_eq!(progeny.page, 0);
            assert_eq!(progeny.page_size, 5);
        }

        // -- search_by_lpn (concurrent) --------------------------------------

        #[tokio::test]
        async fn search_by_lpn_combines_three_requests() {
            let server = MockServer::start().await;

            // Mount details
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "LPN_FULL"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "success": true,
                    "data": {
                        "gender": "Male",
                        "breed": { "breedName": "Suffolk" },
                        "searchResultViewModel": {
                            "lpnId": "LPN_FULL",
                            "status": "CURRENT"
                        }
                    }
                })))
                .mount(&server)
                .await;

            // Mount lineage
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "LPN_FULL"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "LPN_FULL",
                    "content": "<div>Test</div>",
                    "children": []
                })))
                .mount(&server)
                .await;

            // Mount progeny
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .and(query_param("lpnId", "LPN_FULL"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "recordCount": 0,
                    "records": []
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let profile = client.search_by_lpn("LPN_FULL").await.unwrap();
            assert_eq!(profile.details.lpn_id, "LPN_FULL");
            assert_eq!(profile.details.breed.as_deref(), Some("Suffolk"));
            assert_eq!(profile.lineage.subject.unwrap().lpn_id, "LPN_FULL");
            assert_eq!(profile.progeny.total_count, 0);
        }

        // -- Error handling ---------------------------------------------------

        #[tokio::test]
        async fn not_found_returns_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .respond_with(ResponseTemplate::new(404))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let err = client.animal_details("MISSING").await.unwrap_err();
            assert!(matches!(err, Error::NotFound(_)));
        }

        #[tokio::test]
        async fn server_error_returns_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let err = client.date_last_updated().await.unwrap_err();
            assert!(
                matches!(err, Error::Api { status: 500, .. }),
                "expected Api error with status 500, got {err:?}"
            );
        }

        #[tokio::test]
        async fn bad_request_returns_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let err = client.trait_ranges(999).await.unwrap_err();
            assert!(
                matches!(err, Error::Api { status: 400, .. }),
                "expected Api error with status 400, got {err:?}"
            );
        }

        #[tokio::test]
        async fn rate_limited_carries_retry_after_seconds() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(
                    ResponseTemplate::new(429)
                        .insert_header("Retry-After", "30")
                        .set_body_string("rate limited"),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let err = client.date_last_updated().await.unwrap_err();
            assert!(
                matches!(
                    &err,
                    Error::Api {
                        status: 429,
                        retry_after: Some(RetryAfter::Seconds(30)),
                        ..
                    }
                ),
                "expected 429 Api error with retry_after=30, got {err:?}"
            );
        }

        #[test]
        fn parse_retry_after_handles_both_forms() {
            use reqwest::header::{HeaderMap, HeaderValue};

            let mut secs = HeaderMap::new();
            secs.insert("Retry-After", HeaderValue::from_static("45"));
            assert_eq!(parse_retry_after(&secs), Some(RetryAfter::Seconds(45)));

            let mut date = HeaderMap::new();
            date.insert(
                "Retry-After",
                HeaderValue::from_static("Wed, 21 Oct 2026 07:28:00 GMT"),
            );
            let parsed = parse_retry_after(&date);
            assert!(
                matches!(&parsed, Some(RetryAfter::Timestamp(ts)) if ts.starts_with("2026-10-21T07:28:00")),
                "expected Timestamp form, got {parsed:?}"
            );

            assert_eq!(parse_retry_after(&HeaderMap::new()), None);
        }

        #[tokio::test]
        async fn invalid_json_returns_parse_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(ResponseTemplate::new(200).set_body_string("not-valid-json"))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let err = client.date_last_updated().await.unwrap_err();
            assert!(
                matches!(err, Error::Parse { .. }),
                "expected Parse error, got {err:?}"
            );
        }

        #[tokio::test]
        async fn empty_search_results() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "TotalCount": 0,
                    "Results": []
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let results = client
                .search_animals(0, 10, None, None, None, None)
                .await
                .unwrap();
            assert_eq!(results.total_count, 0);
            assert!(results.results.is_empty());
        }

        #[tokio::test]
        async fn progeny_validation_zero_page_size() {
            let client = NsipClient::new();
            let err = client.progeny("LPN1", 0, 0).await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));
        }

        #[tokio::test]
        async fn search_by_lpn_validation_empty_id() {
            let client = NsipClient::new();
            let err = client.search_by_lpn("").await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));

            let err = client.search_by_lpn("  ").await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));
        }

        // -- Retry behavior ---------------------------------------------------

        #[tokio::test]
        async fn retries_on_server_error_then_succeeds() {
            let server = MockServer::start().await;

            // First request returns 500, second returns 200
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(ResponseTemplate::new(500))
                .up_to_n_times(1)
                .mount(&server)
                .await;

            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(serde_json::json!("2024-12-20")),
                )
                .mount(&server)
                .await;

            let client = NsipClient::builder()
                .base_url(server.uri())
                .max_retries(2)
                .build()
                .unwrap();

            let result = client.date_last_updated().await.unwrap();
            assert_eq!(result.data, serde_json::json!("2024-12-20"));
        }

        #[tokio::test]
        async fn no_retry_on_client_error() {
            let server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .respond_with(ResponseTemplate::new(400).set_body_string("Bad"))
                .mount(&server)
                .await;

            // Even with retries enabled, 400 should NOT be retried
            let client = NsipClient::builder()
                .base_url(server.uri())
                .max_retries(3)
                .build()
                .unwrap();

            let err = client.animal_details("X").await.unwrap_err();
            assert!(matches!(err, Error::Api { status: 400, .. }));

            // Verify only one request was made (no retries)
            let requests = server.received_requests().await.unwrap();
            assert_eq!(requests.len(), 1, "should not retry on 400 status");
        }

        // -- parse_breed_value ------------------------------------------------

        #[test]
        fn parse_breed_value_with_aliases() {
            let val = serde_json::json!({ "id": 100, "name": "TestBreed" });
            let breed = NsipClient::parse_breed_value(&val);
            assert_eq!(breed.id, 100);
            assert_eq!(breed.name, "TestBreed");

            let val2 = serde_json::json!({ "breedId": 200, "breedName": "OtherBreed" });
            let breed2 = NsipClient::parse_breed_value(&val2);
            assert_eq!(breed2.id, 200);
            assert_eq!(breed2.name, "OtherBreed");
        }

        #[test]
        fn parse_breed_value_missing_fields() {
            let val = serde_json::json!({});
            let breed = NsipClient::parse_breed_value(&val);
            assert_eq!(breed.id, 0);
            assert_eq!(breed.name, "");
        }
    }
}
