//! MCP tool implementations for the NSIP server.
//!
//! Exposes 13 tools via the `#[tool_router]` macro, covering all NSIP API
//! endpoints plus analytics-powered breeding intelligence.

use std::collections::HashMap;

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
};

use crate::{NsipClient, SearchCriteria};

use super::analytics::{self, RankedAnimal};

// ---------------------------------------------------------------------------
// Parameter structs
// ---------------------------------------------------------------------------

/// Parameters for searching animals (enhanced with all filter fields).
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Breed group ID to filter by.
    #[schemars(description = "Breed group ID to filter by")]
    pub breed_group_id: Option<i64>,

    /// Breed ID to filter by.
    #[schemars(description = "Breed ID to filter by")]
    pub breed_id: Option<i64>,

    /// Animal status to filter by.
    #[schemars(description = "Animal status (e.g. CURRENT, SOLD, DEAD)")]
    pub status: Option<String>,

    /// Gender filter.
    #[schemars(description = "Gender filter (Male, Female, Both)")]
    pub gender: Option<String>,

    /// Only return animals born after this date (YYYY-MM-DD).
    #[schemars(description = "Only return animals born after this date (YYYY-MM-DD)")]
    pub born_after: Option<String>,

    /// Only return animals born before this date (YYYY-MM-DD).
    #[schemars(description = "Only return animals born before this date (YYYY-MM-DD)")]
    pub born_before: Option<String>,

    /// Only return proven animals.
    #[schemars(description = "Only return proven animals")]
    pub proven_only: Option<bool>,

    /// Flock ID to filter by.
    #[schemars(description = "Flock ID to filter by")]
    pub flock_id: Option<String>,

    /// Sort results by trait abbreviation (e.g. BWT, WWT).
    #[schemars(description = "Sort results by trait abbreviation (e.g. BWT, WWT)")]
    pub sort_by: Option<String>,

    /// Reverse the sort order.
    #[schemars(description = "Reverse the sort order")]
    pub reverse: Option<bool>,

    /// Page number (0-indexed).
    #[schemars(description = "Page number (0-indexed)")]
    pub page: Option<u32>,

    /// Number of results per page (1-100).
    #[schemars(description = "Number of results per page (1-100)")]
    pub page_size: Option<u32>,
}

/// Parameters for getting animal details or lineage by ID.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AnimalIdParams {
    /// LPN ID or registration number of the animal.
    #[schemars(description = "LPN ID or registration number of the animal")]
    pub lpn_id: String,
}

/// Parameters for getting animal progeny.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProgenyParams {
    /// LPN ID of the animal.
    #[schemars(description = "LPN ID of the animal")]
    pub lpn_id: String,

    /// Page number (0-indexed).
    #[schemars(description = "Page number (0-indexed)")]
    pub page: Option<u32>,

    /// Number of results per page.
    #[schemars(description = "Number of results per page")]
    pub page_size: Option<u32>,
}

/// Parameters for comparing multiple animals.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompareParams {
    /// LPN IDs of animals to compare (2-5).
    #[schemars(description = "LPN IDs of animals to compare (2-5)")]
    pub lpn_ids: Vec<String>,

    /// Only show specific traits (comma-separated, e.g. BWT,WWT,YWT).
    #[schemars(description = "Only show specific traits (comma-separated, e.g. BWT,WWT,YWT)")]
    pub traits: Option<String>,
}

/// Parameters for ranking animals.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RankParams {
    /// Breed ID to search within.
    #[schemars(description = "Breed ID to search within")]
    pub breed_id: i64,

    /// Trait weights for ranking (e.g. {\"BWT\": -1.0, \"WWT\": 2.0}).
    #[schemars(description = "Trait weights for ranking, e.g. {\"BWT\": -1.0, \"WWT\": 2.0}")]
    pub weights: HashMap<String, f64>,

    /// Gender filter (Male, Female, Both).
    #[schemars(description = "Gender filter (Male, Female, Both)")]
    pub gender: Option<String>,

    /// Animal status filter.
    #[schemars(description = "Animal status filter (e.g. CURRENT)")]
    pub status: Option<String>,

    /// Maximum number of results to return.
    #[schemars(description = "Maximum number of top-ranked results to return (default 10)")]
    pub top_n: Option<usize>,
}

/// Parameters for inbreeding check.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InbreedingParams {
    /// LPN ID of the sire.
    #[schemars(description = "LPN ID of the sire (father)")]
    pub sire_id: String,

    /// LPN ID of the dam.
    #[schemars(description = "LPN ID of the dam (mother)")]
    pub dam_id: String,
}

/// Parameters for mating recommendations.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MatingParams {
    /// LPN ID of the animal to find mates for.
    #[schemars(description = "LPN ID of the animal to find mates for")]
    pub lpn_id: String,

    /// Breed ID to search for potential mates.
    #[schemars(description = "Breed ID to search for potential mates")]
    pub breed_id: i64,

    /// Traits to optimize in offspring (comma-separated, e.g. WWT,NLB).
    #[schemars(description = "Traits to optimize in offspring (comma-separated, e.g. WWT,NLB)")]
    pub target_traits: Option<String>,

    /// Maximum number of recommendations.
    #[schemars(description = "Maximum number of recommendations (default 5)")]
    pub max_results: Option<usize>,
}

/// Parameters for flock summary.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FlockParams {
    /// Flock ID to summarize.
    #[schemars(description = "Flock ID to summarize")]
    pub flock_id: String,

    /// Breed ID to filter by.
    #[schemars(description = "Breed ID to filter within the flock")]
    pub breed_id: Option<i64>,
}

/// Parameters for breed-specific queries.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BreedIdParams {
    /// Breed ID.
    #[schemars(description = "Breed ID to query")]
    pub breed_id: i64,
}

// ---------------------------------------------------------------------------
// Tool router — all 13 tools on NsipServer
// ---------------------------------------------------------------------------

/// Serialize a value to a JSON `CallToolResult`.
fn json_result(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| internal_err("Serialization failed", &e))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(json)]))
}

/// Map a crate-level [`crate::Error`] into an MCP error with the RFC 9457
/// problem+json envelope in `data` and a class-appropriate JSON-RPC code.
/// See [`crate::mcp::problem_error`].
fn api_err(context: &str, err: &crate::Error) -> McpError {
    super::problem_error(context, err)
}

/// Build an `McpError::internal_error` for failures that are not a
/// [`crate::Error`] (e.g. JSON serialization), which carry no envelope.
fn internal_err(context: &str, e: impl std::fmt::Display) -> McpError {
    McpError::internal_error(format!("{context}: {e}"), None)
}

impl From<SearchParams> for SearchCriteria {
    fn from(p: SearchParams) -> Self {
        let mut c = Self::new();
        if let Some(bg) = p.breed_group_id {
            c = c.with_breed_group_id(bg);
        }
        if let Some(bid) = p.breed_id {
            c = c.with_breed_id(bid);
        }
        if let Some(s) = p.status {
            c = c.with_status(s);
        }
        if let Some(g) = p.gender {
            c = c.with_gender(g);
        }
        if let Some(date) = p.born_after {
            c = c.with_born_after(date);
        }
        if let Some(date) = p.born_before {
            c = c.with_born_before(date);
        }
        if p.proven_only == Some(true) {
            c = c.with_proven_only(true);
        }
        if let Some(fid) = p.flock_id {
            c = c.with_flock_id(fid);
        }
        c
    }
}

#[tool_router]
impl super::NsipServer {
    /// Creates a new NSIP MCP server with all tool sets enabled.
    pub fn new() -> Self {
        Self::with_tool_sets(super::tool_sets::EnabledToolSets::all())
    }

    /// Creates a new NSIP MCP server with the specified tool sets.
    pub fn with_tool_sets(sets: super::tool_sets::EnabledToolSets) -> Self {
        let mut router = Self::tool_router();
        for name in sets.disabled_tool_names() {
            router.remove_route(name);
        }
        Self {
            tool_router: router,
            client: NsipClient::new(),
            enabled_tools: sets,
        }
    }

    /// Search for animals in the NSIP database with optional filters.
    #[tool(
        description = "Search for animals in the NSIP database with filters for breed, gender, status, date range, flock, and sorting"
    )]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let page = params.page.unwrap_or(0);
        let page_size = params.page_size.unwrap_or(15);
        let breed_id = params.breed_id;
        let sort_by = params.sort_by.clone();
        let reverse = params.reverse;
        let criteria = SearchCriteria::from(params);

        let results = self
            .client
            .search_animals(
                page,
                page_size,
                breed_id,
                sort_by.as_deref(),
                reverse,
                Some(&criteria),
            )
            .await
            .map_err(|e| api_err("Search failed", &e))?;

        json_result(&results)
    }

    /// Get detailed information about a specific animal by ID.
    #[tool(
        description = "Get detailed EBV data, breed, contact info, and status for a specific animal by LPN ID"
    )]
    async fn details(
        &self,
        Parameters(params): Parameters<AnimalIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let animal = self
            .client
            .animal_details(&params.lpn_id)
            .await
            .map_err(|e| api_err("Failed to fetch details", &e))?;

        json_result(&animal)
    }

    /// Get lineage (ancestry) information for a specific animal.
    #[tool(
        description = "Get pedigree / ancestry tree for a specific animal including parents and grandparents"
    )]
    async fn lineage(
        &self,
        Parameters(params): Parameters<AnimalIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let lineage = self
            .client
            .lineage(&params.lpn_id)
            .await
            .map_err(|e| api_err("Failed to fetch lineage", &e))?;

        json_result(&lineage)
    }

    /// Get progeny (offspring) for a specific animal.
    #[tool(description = "Get paginated list of offspring for a specific animal")]
    async fn progeny(
        &self,
        Parameters(params): Parameters<ProgenyParams>,
    ) -> Result<CallToolResult, McpError> {
        let page = params.page.unwrap_or(0);
        let page_size = params.page_size.unwrap_or(10);

        let progeny = self
            .client
            .progeny(&params.lpn_id, page, page_size)
            .await
            .map_err(|e| api_err("Failed to fetch progeny", &e))?;

        json_result(&progeny)
    }

    /// Get full profile (details + lineage + progeny) for an animal.
    #[tool(
        description = "Get complete profile for an animal: details, pedigree, and offspring in one call"
    )]
    async fn profile(
        &self,
        Parameters(params): Parameters<AnimalIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let profile = self
            .client
            .search_by_lpn(&params.lpn_id)
            .await
            .map_err(|e| api_err("Failed to fetch profile", &e))?;

        json_result(&profile)
    }

    /// List all available breed groups and their breeds.
    #[tool(description = "List all breed groups and individual breeds in the NSIP database")]
    async fn breed_groups(&self) -> Result<CallToolResult, McpError> {
        let groups = self
            .client
            .breed_groups()
            .await
            .map_err(|e| api_err("Failed to fetch breed groups", &e))?;

        json_result(&groups)
    }

    /// Get trait value ranges for a specific breed.
    #[tool(
        description = "Get min/max EBV trait ranges for a specific breed — useful for understanding breed norms"
    )]
    async fn trait_ranges(
        &self,
        Parameters(params): Parameters<BreedIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let ranges = match self.client.trait_ranges(params.breed_id).await {
            Ok(r) => r,
            Err(crate::Error::Api { status: 400, .. }) => {
                let msg = format!(
                    "No trait range data available for breed {}. \
                     The breed may not have enough evaluated animals \
                     for range calculations. Use breed_groups to find \
                     valid breed IDs.",
                    params.breed_id
                );
                return Ok(CallToolResult::success(vec![ContentBlock::text(msg)]));
            },
            Err(e) => return Err(api_err("Failed to fetch trait ranges", &e)),
        };

        json_result(&ranges)
    }

    /// Compare two or more animals side-by-side.
    #[tool(
        description = "Compare 2-5 animals side-by-side on their EBV traits. Optionally filter to specific traits."
    )]
    async fn compare(
        &self,
        Parameters(params): Parameters<CompareParams>,
    ) -> Result<CallToolResult, McpError> {
        if params.lpn_ids.len() < 2 || params.lpn_ids.len() > 5 {
            return Err(api_err(
                "compare",
                &crate::Error::compare_arity("lpn_ids must contain 2-5 LPN IDs"),
            ));
        }

        let mut animals = Vec::new();
        for id in &params.lpn_ids {
            let details = self
                .client
                .animal_details(id)
                .await
                .map_err(|e| api_err(&format!("Failed to fetch {id}"), &e))?;
            animals.push(details);
        }

        let trait_filter: Option<Vec<String>> = params
            .traits
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

        let comparison = build_comparison(&animals, trait_filter.as_deref());
        json_result(&comparison)
    }

    /// Rank animals by weighted EBV composite score.
    #[tool(
        description = "Rank animals within a breed by weighted EBV traits. Specify trait weights to prioritize breeding goals. Note: ranking is based on a sample of up to 100 animals from a single search page; results may not reflect the full population."
    )]
    async fn rank(
        &self,
        Parameters(params): Parameters<RankParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut criteria = SearchCriteria::new().with_breed_id(params.breed_id);
        if let Some(g) = params.gender {
            criteria = criteria.with_gender(g);
        }
        if let Some(s) = params.status {
            criteria = criteria.with_status(s);
        }

        let results = self
            .client
            .search_animals(0, 100, Some(params.breed_id), None, None, Some(&criteria))
            .await
            .map_err(|e| api_err("Search failed", &e))?;

        let animals = parse_search_result_animals(&results.results);
        let mut ranked = analytics::rank_animals(&animals, &params.weights);

        let top_n = params.top_n.unwrap_or(10);
        ranked.truncate(top_n);

        json_result(&ranked)
    }

    /// Check inbreeding coefficient for a potential sire-dam pairing.
    #[tool(
        description = "Calculate Wright's coefficient of inbreeding (COI) for a potential sire-dam mating. Returns COI value, traffic-light rating (Green/Yellow/Red), and shared ancestors."
    )]
    async fn inbreeding_check(
        &self,
        Parameters(params): Parameters<InbreedingParams>,
    ) -> Result<CallToolResult, McpError> {
        let (sire_lineage, dam_lineage) = tokio::join!(
            self.client.lineage(&params.sire_id),
            self.client.lineage(&params.dam_id),
        );

        let sire_lineage = sire_lineage.map_err(|e| api_err("Failed to fetch sire lineage", &e))?;
        let dam_lineage = dam_lineage.map_err(|e| api_err("Failed to fetch dam lineage", &e))?;

        let result = analytics::calculate_coi(&sire_lineage, &dam_lineage);
        json_result(&result)
    }

    /// Get mating recommendations for an animal.
    #[tool(
        description = "Find optimal mates for an animal: searches the breed for candidates, checks inbreeding, and ranks by trait complementarity"
    )]
    async fn mating_recommendations(
        &self,
        Parameters(params): Parameters<MatingParams>,
    ) -> Result<CallToolResult, McpError> {
        let animal_details = self
            .client
            .animal_details(&params.lpn_id)
            .await
            .map_err(|e| api_err("Failed to fetch animal", &e))?;

        let mate_gender = match animal_details.gender.as_deref() {
            Some("Male") => "Female",
            Some("Female") => "Male",
            _ => "Both",
        };

        let weights = build_target_weights(params.target_traits.as_deref());

        let criteria = SearchCriteria::new()
            .with_breed_id(params.breed_id)
            .with_gender(mate_gender)
            .with_status("CURRENT");

        let candidates = self
            .client
            .search_animals(0, 50, Some(params.breed_id), None, None, Some(&criteria))
            .await
            .map_err(|e| api_err("Search failed", &e))?;

        let candidate_animals = parse_search_result_animals(&candidates.results);
        let ranked = analytics::rank_animals(&candidate_animals, &weights);

        let max_results = params.max_results.unwrap_or(5);
        let top_candidates: Vec<&RankedAnimal> = ranked.iter().take(max_results).collect();

        let animal_lineage = self
            .client
            .lineage(&params.lpn_id)
            .await
            .map_err(|e| api_err("Failed to fetch lineage", &e))?;

        let mut recommendations = Vec::new();
        for candidate in top_candidates {
            let mate_lineage_result = self.client.lineage(&candidate.lpn_id).await;
            let coi_reliable = mate_lineage_result.is_ok();
            let mate_lineage = mate_lineage_result.unwrap_or_else(|_| crate::Lineage {
                subject: None,
                sire: None,
                dam: None,
                generations: Vec::new(),
            });

            let coi = analytics::calculate_coi(&animal_lineage, &mate_lineage);
            let complementarity = candidate_animals
                .iter()
                .find(|a| a.lpn_id == candidate.lpn_id)
                .map(|mate| analytics::trait_complementarity(&animal_details, mate))
                .unwrap_or_default();

            recommendations.push(serde_json::json!({
                "mate_lpn_id": candidate.lpn_id,
                "rank_score": candidate.score,
                "coi": {
                    "coefficient": coi.coefficient,
                    "rating": coi.rating,
                    "reliable": coi_reliable,
                },
                "predicted_offspring_ebvs": complementarity,
            }));
        }

        json_result(&recommendations)
    }

    /// Get summary statistics for a flock.
    #[tool(
        description = "Summarize a flock's animals: count, gender breakdown, and average EBV traits. Averages are computed from a sample of up to 100 animals; total_count reflects the full population."
    )]
    async fn flock_summary(
        &self,
        Parameters(params): Parameters<FlockParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut criteria = SearchCriteria::new().with_flock_id(&params.flock_id);
        if let Some(bid) = params.breed_id {
            criteria = criteria.with_breed_id(bid);
        }

        let results = self
            .client
            .search_animals(0, 100, params.breed_id, None, None, Some(&criteria))
            .await
            .map_err(|e| api_err("Search failed", &e))?;

        let animals = parse_search_result_animals(&results.results);
        let summary = build_flock_summary(&params.flock_id, &animals, results.total_count);

        json_result(&summary)
    }

    /// Get NSIP database status and available statuses.
    #[tool(description = "Get NSIP database last-updated date and available animal statuses")]
    async fn database_status(&self) -> Result<CallToolResult, McpError> {
        let (updated, statuses) =
            tokio::join!(self.client.date_last_updated(), self.client.statuses(),);

        let updated = updated.map_err(|e| api_err("Failed to fetch date", &e))?;
        let statuses = statuses.map_err(|e| api_err("Failed to fetch statuses", &e))?;

        let result = serde_json::json!({
            "last_updated": updated.data,
            "statuses": statuses,
        });

        json_result(&result)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse animal details from raw search result JSON objects.
fn parse_search_result_animals(results: &[serde_json::Value]) -> Vec<crate::AnimalDetails> {
    results
        .iter()
        .filter_map(|r| crate::AnimalDetails::from_api_response(r).ok())
        .collect()
}

/// Build a comparison table from multiple animals.
fn build_comparison(
    animals: &[crate::AnimalDetails],
    trait_filter: Option<&[String]>,
) -> serde_json::Value {
    let animals_arr: Vec<serde_json::Value> = animals
        .iter()
        .map(|animal| {
            let mut traits_obj = serde_json::Map::new();
            for (name, t) in &animal.traits {
                if let Some(filter) = trait_filter
                    && !filter.iter().any(|f| f == name)
                {
                    continue;
                }
                traits_obj.insert(
                    name.clone(),
                    serde_json::json!({
                        "value": t.value,
                        "accuracy": t.accuracy,
                    }),
                );
            }

            serde_json::json!({
                "lpn_id": animal.lpn_id,
                "breed": animal.breed,
                "gender": animal.gender,
                "status": animal.status,
                "traits": traits_obj,
            })
        })
        .collect();

    serde_json::json!({
        "animal_count": animals.len(),
        "animals": animals_arr,
    })
}

/// Build default weights from a comma-separated list of target traits.
///
/// Traits where lower EBV values are desirable (birth weight, parasite egg
/// counts, fibre diameter) receive negative weights. The lower-is-better set is
/// derived from the canonical EBV glossary via
/// [`super::analytics::is_lower_is_better`] so it stays in sync as traits change.
fn build_target_weights(target_traits: Option<&str>) -> HashMap<String, f64> {
    let mut weights = HashMap::new();

    if let Some(traits_str) = target_traits {
        for trait_name in traits_str.split(',') {
            let name = trait_name.trim().to_uppercase();
            let weight = if super::analytics::is_lower_is_better(&name) {
                -1.0
            } else {
                1.0
            };
            weights.insert(name, weight);
        }
    } else {
        weights.insert("WWT".to_string(), 1.0);
        weights.insert("BWT".to_string(), -0.5);
        weights.insert("NLB".to_string(), 0.5);
    }

    weights
}

/// Build flock summary statistics.
#[allow(clippy::cast_precision_loss)]
fn build_flock_summary(
    flock_id: &str,
    animals: &[crate::AnimalDetails],
    total_count: i64,
) -> serde_json::Value {
    let males = animals
        .iter()
        .filter(|a| a.gender.as_deref() == Some("Male"))
        .count();
    let females = animals
        .iter()
        .filter(|a| a.gender.as_deref() == Some("Female"))
        .count();

    let mut trait_sums: HashMap<String, (f64, usize)> = HashMap::new();
    for animal in animals {
        for (name, t) in &animal.traits {
            let entry = trait_sums.entry(name.clone()).or_insert((0.0, 0));
            entry.0 += t.value;
            entry.1 += 1;
        }
    }

    let trait_averages: HashMap<String, f64> = trait_sums
        .into_iter()
        .map(|(name, (sum, count))| (name, sum / count as f64))
        .collect();

    serde_json::json!({
        "flock_id": flock_id,
        "total_count": total_count,
        "sample_size": animals.len(),
        "males": males,
        "females": females,
        "trait_averages": trait_averages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_target_weights_from_traits() {
        let weights = build_target_weights(Some("WWT, BWT, NLB"));
        assert_eq!(weights.len(), 3);
        assert!(*weights.get("WWT").unwrap() > 0.0);
        assert!(*weights.get("BWT").unwrap() < 0.0);
        assert!(*weights.get("NLB").unwrap() > 0.0);
    }

    #[test]
    fn build_target_weights_defaults() {
        let weights = build_target_weights(None);
        assert!(weights.contains_key("WWT"));
        assert!(weights.contains_key("BWT"));
        assert!(weights.contains_key("NLB"));
    }

    #[test]
    fn build_target_weights_negative_traits() {
        // All four lower-is-better glossary traits (incl. YFD) must be negative;
        // derived from EBV_TRAITS so this can't drift from the glossary.
        let weights = build_target_weights(Some("BWT, WFEC, PFEC, YFD"));
        assert_eq!(weights.len(), 4);
        assert!(*weights.get("BWT").unwrap() < 0.0);
        assert!(*weights.get("WFEC").unwrap() < 0.0);
        assert!(*weights.get("PFEC").unwrap() < 0.0);
        assert!(*weights.get("YFD").unwrap() < 0.0);
        // A higher-is-better trait stays positive.
        assert!(*build_target_weights(Some("YWT")).get("YWT").unwrap() > 0.0);
    }

    #[test]
    fn build_target_weights_case_insensitive() {
        let weights = build_target_weights(Some("bwt, wwt"));
        assert!(weights.contains_key("BWT"));
        assert!(weights.contains_key("WWT"));
    }

    #[test]
    fn server_creation() {
        let server = super::super::NsipServer::new();
        assert!(std::any::type_name_of_val(&server).contains("NsipServer"));
    }

    // ------------------------------------------------------------------
    // parse_search_result_animals
    // ------------------------------------------------------------------

    #[test]
    fn parse_search_result_animals_camel_case() {
        let results = vec![
            serde_json::json!({
                "lpnId": "A1",
                "gender": "Male",
                "status": "CURRENT",
                "bwt": 0.5,
                "accbwt": 80,
                "wwt": 10.0,
                "accwwt": 70
            }),
            serde_json::json!({
                "lpnId": "A2",
                "gender": "Female",
                "status": "SOLD",
                "bwt": -0.2,
                "accbwt": 60
            }),
        ];

        let animals = parse_search_result_animals(&results);
        assert_eq!(animals.len(), 2);
        assert_eq!(animals[0].lpn_id, "A1");
        assert_eq!(animals[0].gender.as_deref(), Some("Male"));
        assert!(animals[0].traits.contains_key("BWT"));
        assert!(animals[0].traits.contains_key("WWT"));
        assert_eq!(animals[1].lpn_id, "A2");
        assert_eq!(animals[1].traits.len(), 1);
    }

    #[test]
    fn parse_search_result_animals_empty_input() {
        let animals = parse_search_result_animals(&[]);
        assert!(animals.is_empty());
    }

    #[test]
    fn parse_search_result_animals_nested_format() {
        let results = vec![serde_json::json!({
            "success": true,
            "data": {
                "gender": "Male",
                "breed": { "breedName": "Katahdin" },
                "searchResultViewModel": {
                    "lpnId": "NESTED1",
                    "status": "CURRENT",
                    "bwt": 1.0,
                    "accbwt": 90
                }
            }
        })];

        let animals = parse_search_result_animals(&results);
        assert_eq!(animals.len(), 1);
        assert_eq!(animals[0].lpn_id, "NESTED1");
    }

    // ------------------------------------------------------------------
    // build_comparison
    // ------------------------------------------------------------------

    fn make_test_animal(
        lpn_id: &str,
        breed: Option<&str>,
        gender: Option<&str>,
        trait_data: &[(&str, f64, Option<i32>)],
    ) -> crate::AnimalDetails {
        let mut traits = HashMap::new();
        for &(name, value, accuracy) in trait_data {
            traits.insert(
                name.to_string(),
                crate::Trait {
                    name: name.to_string(),
                    value,
                    accuracy,
                    units: None,
                },
            );
        }
        crate::AnimalDetails {
            lpn_id: lpn_id.to_string(),
            breed: breed.map(String::from),
            breed_group: None,
            date_of_birth: None,
            gender: gender.map(String::from),
            status: Some("CURRENT".to_string()),
            sire: None,
            dam: None,
            registration_number: None,
            total_progeny: None,
            flock_count: None,
            genotyped: None,
            traits,
            contact_info: None,
        }
    }

    #[test]
    fn build_comparison_two_animals() {
        let animals = vec![
            make_test_animal(
                "A1",
                Some("Katahdin"),
                Some("Male"),
                &[("BWT", 0.5, Some(80)), ("WWT", 10.0, Some(70))],
            ),
            make_test_animal(
                "A2",
                Some("Suffolk"),
                Some("Female"),
                &[("BWT", -0.3, Some(60)), ("WWT", 15.0, Some(85))],
            ),
        ];

        let comparison = build_comparison(&animals, None);
        assert_eq!(comparison["animal_count"], 2);

        let arr = comparison["animals"].as_array().unwrap();
        assert_eq!(arr[0]["lpn_id"], "A1");
        assert_eq!(arr[0]["breed"], "Katahdin");
        assert_eq!(arr[1]["lpn_id"], "A2");

        // Both animals should have BWT and WWT traits
        assert!(arr[0]["traits"]["BWT"]["value"].is_number());
        assert!(arr[1]["traits"]["WWT"]["value"].is_number());
    }

    #[test]
    fn build_comparison_with_trait_filter() {
        let animals = vec![make_test_animal(
            "A1",
            None,
            None,
            &[
                ("BWT", 0.5, Some(80)),
                ("WWT", 10.0, Some(70)),
                ("NLB", 0.15, Some(40)),
            ],
        )];

        let filter = vec!["BWT".to_string(), "NLB".to_string()];
        let comparison = build_comparison(&animals, Some(&filter));

        let traits = &comparison["animals"][0]["traits"];
        assert!(traits.get("BWT").is_some());
        assert!(traits.get("NLB").is_some());
        assert!(traits.get("WWT").is_none(), "WWT should be filtered out");
    }

    #[test]
    fn build_comparison_empty_animals() {
        let comparison = build_comparison(&[], None);
        assert_eq!(comparison["animal_count"], 0);
        assert!(comparison["animals"].as_array().unwrap().is_empty());
    }

    // ------------------------------------------------------------------
    // build_flock_summary
    // ------------------------------------------------------------------

    #[test]
    fn build_flock_summary_basic() {
        let animals = vec![
            make_test_animal(
                "A1",
                None,
                Some("Male"),
                &[("BWT", 0.5, Some(80)), ("WWT", 10.0, Some(70))],
            ),
            make_test_animal(
                "A2",
                None,
                Some("Female"),
                &[("BWT", -0.3, Some(60)), ("WWT", 14.0, Some(85))],
            ),
            make_test_animal(
                "A3",
                None,
                Some("Female"),
                &[("BWT", 0.1, Some(50)), ("WWT", 12.0, Some(75))],
            ),
        ];

        let summary = build_flock_summary("FLOCK1", &animals, 50);
        assert_eq!(summary["flock_id"], "FLOCK1");
        assert_eq!(summary["total_count"], 50);
        assert_eq!(summary["sample_size"], 3);
        assert_eq!(summary["males"], 1);
        assert_eq!(summary["females"], 2);

        let averages = summary["trait_averages"].as_object().unwrap();
        let bwt_avg = averages["BWT"].as_f64().unwrap();
        // (0.5 + -0.3 + 0.1) / 3 = 0.1
        assert!((bwt_avg - 0.1).abs() < 1e-10);
        let wwt_avg = averages["WWT"].as_f64().unwrap();
        // (10 + 14 + 12) / 3 = 12.0
        assert!((wwt_avg - 12.0).abs() < 1e-10);
    }

    #[test]
    fn build_flock_summary_empty() {
        let summary = build_flock_summary("EMPTY_FLOCK", &[], 0);
        assert_eq!(summary["flock_id"], "EMPTY_FLOCK");
        assert_eq!(summary["total_count"], 0);
        assert_eq!(summary["sample_size"], 0);
        assert_eq!(summary["males"], 0);
        assert_eq!(summary["females"], 0);
        assert!(summary["trait_averages"].as_object().unwrap().is_empty());
    }

    #[test]
    fn build_flock_summary_single_animal() {
        let animals = vec![make_test_animal(
            "A1",
            None,
            Some("Male"),
            &[("BWT", 0.5, Some(80))],
        )];

        let summary = build_flock_summary("F1", &animals, 1);
        assert_eq!(summary["males"], 1);
        assert_eq!(summary["females"], 0);
        let bwt_avg = summary["trait_averages"]["BWT"].as_f64().unwrap();
        assert!((bwt_avg - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn build_flock_summary_unknown_gender() {
        let animals = vec![make_test_animal("A1", None, None, &[])];

        let summary = build_flock_summary("F1", &animals, 1);
        assert_eq!(summary["sample_size"], 1);
        assert_eq!(summary["males"], 0);
        assert_eq!(summary["females"], 0);
    }

    // ------------------------------------------------------------------
    // json_result helper
    // ------------------------------------------------------------------

    #[test]
    fn json_result_produces_success() {
        let data = serde_json::json!({"test": 42});
        let result = json_result(&data).unwrap();
        assert!(!result.is_error.unwrap_or(false));
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn api_err_attaches_problem_envelope() {
        let err = crate::Error::api(503, "upstream down");
        let mcp = api_err("Search failed", &err);
        let data = mcp
            .data
            .expect("MCP error must carry the problem+json envelope");
        assert_eq!(
            data["type"],
            "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/error.md"
        );
        assert_eq!(data["status"], 503);
        assert_eq!(data["exit_code"], 75);
        assert!(
            data["instance"]
                .as_str()
                .is_some_and(|s| s.starts_with("urn:nsip:"))
        );
    }

    #[test]
    fn internal_err_has_no_envelope() {
        // Non-`crate::Error` failures (e.g. serialization) carry no envelope.
        let mcp = internal_err("Serialization failed", "boom");
        assert!(mcp.data.is_none());
    }

    // ------------------------------------------------------------------
    // Wiremock-based async tests for tool handlers
    // ------------------------------------------------------------------

    mod handler_tests {
        use rmcp::handler::server::wrapper::Parameters;
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path, query_param},
        };

        use super::super::super::NsipServer;
        use super::*;

        /// Build an `NsipServer` pointing at the given mock URI.
        fn mock_server(url: &str) -> NsipServer {
            NsipServer {
                tool_router: NsipServer::tool_router(),
                client: NsipClient::with_base_url(url),
                enabled_tools: crate::mcp::tool_sets::EnabledToolSets::all(),
            }
        }

        /// JSON for a details endpoint response (nested format).
        fn details_response(lpn_id: &str, gender: &str) -> serde_json::Value {
            serde_json::json!({
                "success": true,
                "data": {
                    "gender": gender,
                    "breed": { "breedName": "Katahdin", "breedId": 640 },
                    "progenyCount": 3,
                    "searchResultViewModel": {
                        "lpnId": lpn_id,
                        "status": "CURRENT",
                        "bwt": 0.5,
                        "accbwt": 80,
                        "wwt": 12.0,
                        "accwwt": 70,
                        "nlb": 0.15,
                        "accnlb": 50
                    }
                }
            })
        }

        /// JSON for a lineage endpoint response.
        fn lineage_response(lpn_id: &str) -> serde_json::Value {
            serde_json::json!({
                "lpnId": lpn_id,
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
            })
        }

        /// JSON for a search endpoint response.
        fn search_response(animals: &[(&str, &str)]) -> serde_json::Value {
            let results: Vec<serde_json::Value> = animals
                .iter()
                .map(|(id, gender)| {
                    serde_json::json!({
                        "lpnId": id,
                        "gender": gender,
                        "status": "CURRENT",
                        "bwt": 0.5,
                        "accbwt": 80,
                        "wwt": 12.0,
                        "accwwt": 70,
                        "nlb": 0.15,
                        "accnlb": 50
                    })
                })
                .collect();
            serde_json::json!({
                "TotalCount": results.len(),
                "Results": results
            })
        }

        // -- search -------------------------------------------------------

        #[tokio::test]
        async fn search_with_all_params() {
            let mock = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(search_response(&[("A1", "Male"), ("A2", "Female")])),
                )
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .search(Parameters(SearchParams {
                    breed_group_id: Some(61),
                    breed_id: Some(640),
                    status: Some("CURRENT".to_string()),
                    gender: Some("Male".to_string()),
                    born_after: Some("2020-01-01".to_string()),
                    born_before: Some("2024-01-01".to_string()),
                    proven_only: Some(true),
                    flock_id: Some("F1".to_string()),
                    sort_by: Some("BWT".to_string()),
                    reverse: Some(true),
                    page: Some(0),
                    page_size: Some(15),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let text = &result.content[0];
            let json: serde_json::Value =
                serde_json::from_str(&text.as_text().unwrap().text).unwrap();
            assert_eq!(json["total_count"], 2);
        }

        #[tokio::test]
        async fn search_defaults() {
            let mock = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(search_response(&[("X1", "Male")])),
                )
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .search(Parameters(SearchParams {
                    breed_group_id: None,
                    breed_id: None,
                    status: None,
                    gender: None,
                    born_after: None,
                    born_before: None,
                    proven_only: None,
                    flock_id: None,
                    sort_by: None,
                    reverse: None,
                    page: None,
                    page_size: None,
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
        }

        #[tokio::test]
        async fn search_api_error() {
            let mock = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Internal"))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let err = server
                .search(Parameters(SearchParams {
                    breed_group_id: None,
                    breed_id: None,
                    status: None,
                    gender: None,
                    born_after: None,
                    born_before: None,
                    proven_only: None,
                    flock_id: None,
                    sort_by: None,
                    reverse: None,
                    page: None,
                    page_size: None,
                }))
                .await;

            assert!(err.is_err());
        }

        // -- details ------------------------------------------------------

        #[tokio::test]
        async fn details_success() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "LPN1"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("LPN1", "Male")),
                )
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .details(Parameters(AnimalIdParams {
                    lpn_id: "LPN1".to_string(),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["lpn_id"], "LPN1");
            assert_eq!(json["gender"], "Male");
        }

        // -- lineage ------------------------------------------------------

        #[tokio::test]
        async fn lineage_success() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "LPN1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(lineage_response("LPN1")))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .lineage(Parameters(AnimalIdParams {
                    lpn_id: "LPN1".to_string(),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["subject"]["lpn_id"], "LPN1");
        }

        // -- progeny ------------------------------------------------------

        #[tokio::test]
        async fn progeny_success() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .and(query_param("lpnId", "LPN1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "recordCount": 5,
                    "records": [
                        { "lpnId": "P1", "sex": "Male", "dob": "03/01/2023" },
                        { "lpnId": "P2", "sex": "Female", "dob": "03/02/2023" }
                    ]
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .progeny(Parameters(ProgenyParams {
                    lpn_id: "LPN1".to_string(),
                    page: Some(0),
                    page_size: Some(10),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["total_count"], 5);
            assert_eq!(json["animals"].as_array().unwrap().len(), 2);
        }

        #[tokio::test]
        async fn progeny_defaults() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "recordCount": 0,
                    "records": []
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .progeny(Parameters(ProgenyParams {
                    lpn_id: "LPN1".to_string(),
                    page: None,
                    page_size: None,
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
        }

        // -- profile ------------------------------------------------------

        #[tokio::test]
        async fn profile_success() {
            let mock = MockServer::start().await;

            // search_by_lpn calls details, lineage, and progeny concurrently
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("LPN1", "Female")),
                )
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .respond_with(ResponseTemplate::new(200).set_body_json(lineage_response("LPN1")))
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "recordCount": 0,
                    "records": []
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .profile(Parameters(AnimalIdParams {
                    lpn_id: "LPN1".to_string(),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["details"]["lpn_id"], "LPN1");
        }

        // -- breed_groups -------------------------------------------------

        #[tokio::test]
        async fn breed_groups_success() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getAvailableBreedGroups"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "success": true,
                    "data": [{
                        "breedGroupId": 61,
                        "breedGroupName": "Range",
                        "breeds": [
                            { "breedId": 640, "breedName": "Katahdin" }
                        ]
                    }]
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server.breed_groups().await.unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json[0]["id"], 61);
            assert_eq!(json[0]["breeds"][0]["name"], "Katahdin");
        }

        // -- trait_ranges -------------------------------------------------

        #[tokio::test]
        async fn trait_ranges_success() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .and(query_param("breedId", "640"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "rangeBWT": { "min": -2.0, "max": 3.0 },
                    "rangeWWT": { "min": 0.0, "max": 20.0 }
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .trait_ranges(Parameters(BreedIdParams { breed_id: 640 }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert!(json["rangeBWT"]["min"].is_number());
        }

        #[tokio::test]
        async fn trait_ranges_bad_breed_returns_friendly_message() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .and(query_param("breedId", "25"))
                .respond_with(ResponseTemplate::new(400).set_body_string("An Error Occured"))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .trait_ranges(Parameters(BreedIdParams { breed_id: 25 }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let text = &result.content[0].as_text().unwrap().text;
            assert!(text.contains("No trait range data available for breed 25"));
            assert!(text.contains("breed_groups"));
        }

        #[tokio::test]
        async fn trait_ranges_server_error_returns_mcp_error() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .trait_ranges(Parameters(BreedIdParams { breed_id: 640 }))
                .await;

            assert!(result.is_err());
        }

        // -- compare ------------------------------------------------------

        #[tokio::test]
        async fn compare_two_animals() {
            let mock = MockServer::start().await;

            // The compare handler calls animal_details for each ID
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "A1"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("A1", "Male")),
                )
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "A2"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("A2", "Female")),
                )
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .compare(Parameters(CompareParams {
                    lpn_ids: vec!["A1".to_string(), "A2".to_string()],
                    traits: None,
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["animal_count"], 2);
        }

        #[tokio::test]
        async fn compare_with_trait_filter() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "A1"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("A1", "Male")),
                )
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "A2"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("A2", "Female")),
                )
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .compare(Parameters(CompareParams {
                    lpn_ids: vec!["A1".to_string(), "A2".to_string()],
                    traits: Some("BWT".to_string()),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            // Only BWT should appear in traits
            let traits = &json["animals"][0]["traits"];
            assert!(traits.get("BWT").is_some());
            assert!(traits.get("WWT").is_none());
        }

        #[tokio::test]
        async fn compare_too_few_ids() {
            let server = mock_server("http://unused");
            let err = server
                .compare(Parameters(CompareParams {
                    lpn_ids: vec!["A1".to_string()],
                    traits: None,
                }))
                .await;

            assert!(err.is_err());
        }

        #[tokio::test]
        async fn compare_too_many_ids() {
            let server = mock_server("http://unused");
            let err = server
                .compare(Parameters(CompareParams {
                    lpn_ids: vec![
                        "A1".into(),
                        "A2".into(),
                        "A3".into(),
                        "A4".into(),
                        "A5".into(),
                        "A6".into(),
                    ],
                    traits: None,
                }))
                .await;

            assert!(err.is_err());
        }

        // -- rank ---------------------------------------------------------

        #[tokio::test]
        async fn rank_success() {
            let mock = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(200).set_body_json(search_response(&[
                    ("R1", "Male"),
                    ("R2", "Male"),
                    ("R3", "Male"),
                ])))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let mut weights = HashMap::new();
            weights.insert("BWT".to_string(), -1.0);
            weights.insert("WWT".to_string(), 2.0);

            let result = server
                .rank(Parameters(RankParams {
                    breed_id: 640,
                    weights,
                    gender: Some("Male".to_string()),
                    status: Some("CURRENT".to_string()),
                    top_n: Some(2),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            // top_n = 2, so only 2 results
            assert_eq!(json.as_array().unwrap().len(), 2);
        }

        #[tokio::test]
        async fn rank_defaults() {
            let mock = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(search_response(&[("R1", "Male")])),
                )
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let mut weights = HashMap::new();
            weights.insert("WWT".to_string(), 1.0);

            let result = server
                .rank(Parameters(RankParams {
                    breed_id: 640,
                    weights,
                    gender: None,
                    status: None,
                    top_n: None,
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
        }

        // -- inbreeding_check ---------------------------------------------

        #[tokio::test]
        async fn inbreeding_check_no_shared_ancestors() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "SIRE1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "SIRE1",
                    "content": "<div>Sire Farm</div>",
                    "children": [
                        { "lpnId": "GS1", "content": "<div>GS1</div>", "children": [] },
                        { "lpnId": "GD1", "content": "<div>GD1</div>", "children": [] }
                    ]
                })))
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "DAM1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "DAM1",
                    "content": "<div>Dam Farm</div>",
                    "children": [
                        { "lpnId": "GS2", "content": "<div>GS2</div>", "children": [] },
                        { "lpnId": "GD2", "content": "<div>GD2</div>", "children": [] }
                    ]
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .inbreeding_check(Parameters(InbreedingParams {
                    sire_id: "SIRE1".to_string(),
                    dam_id: "DAM1".to_string(),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["coefficient"], 0.0);
            assert_eq!(json["rating"], "Green");
        }

        #[tokio::test]
        async fn inbreeding_check_shared_ancestor() {
            let mock = MockServer::start().await;
            // Both sire and dam share grandparent COMMON
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "SIRE1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "SIRE1",
                    "content": "<div>Sire</div>",
                    "children": [
                        {
                            "lpnId": "GS1",
                            "content": "<div>GS1</div>",
                            "children": [
                                { "lpnId": "COMMON", "content": "<div>Common</div>", "children": [] },
                                { "lpnId": "X1", "content": "<div>X1</div>", "children": [] }
                            ]
                        },
                        { "lpnId": "GD1", "content": "<div>GD1</div>", "children": [] }
                    ]
                })))
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "DAM1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "DAM1",
                    "content": "<div>Dam</div>",
                    "children": [
                        {
                            "lpnId": "GS2",
                            "content": "<div>GS2</div>",
                            "children": [
                                { "lpnId": "COMMON", "content": "<div>Common</div>", "children": [] },
                                { "lpnId": "X2", "content": "<div>X2</div>", "children": [] }
                            ]
                        },
                        { "lpnId": "GD2", "content": "<div>GD2</div>", "children": [] }
                    ]
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .inbreeding_check(Parameters(InbreedingParams {
                    sire_id: "SIRE1".to_string(),
                    dam_id: "DAM1".to_string(),
                }))
                .await
                .unwrap();

            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert!(json["coefficient"].as_f64().unwrap() > 0.0);
            assert!(!json["shared_ancestors"].as_array().unwrap().is_empty());
        }

        // -- mating_recommendations ----------------------------------------

        #[tokio::test]
        async fn mating_recommendations_success() {
            let mock = MockServer::start().await;

            // 1. animal_details for the target animal
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "RAM1"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("RAM1", "Male")),
                )
                .mount(&mock)
                .await;

            // 2. search for candidates (females)
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(search_response(&[("EWE1", "Female")])),
                )
                .mount(&mock)
                .await;

            // 3. lineage for the target animal
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "RAM1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(lineage_response("RAM1")))
                .mount(&mock)
                .await;

            // 4. lineage for each candidate
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "EWE1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "EWE1",
                    "content": "<div>Ewe Farm</div>",
                    "children": []
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .mating_recommendations(Parameters(MatingParams {
                    lpn_id: "RAM1".to_string(),
                    breed_id: 640,
                    target_traits: Some("WWT,BWT".to_string()),
                    max_results: Some(1),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            let recs = json.as_array().unwrap();
            assert_eq!(recs.len(), 1);
            assert_eq!(recs[0]["mate_lpn_id"], "EWE1");
            assert!(recs[0]["coi"]["coefficient"].is_number());
        }

        #[tokio::test]
        async fn mating_recommendations_female_animal() {
            let mock = MockServer::start().await;

            // Target is Female, so candidates should be Male
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "EWE1"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(details_response("EWE1", "Female")),
                )
                .mount(&mock)
                .await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(search_response(&[("RAM1", "Male")])),
                )
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "EWE1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(lineage_response("EWE1")))
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "RAM1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "lpnId": "RAM1",
                    "content": "<div>Ram Farm</div>",
                    "children": []
                })))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .mating_recommendations(Parameters(MatingParams {
                    lpn_id: "EWE1".to_string(),
                    breed_id: 640,
                    target_traits: None,
                    max_results: None,
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
        }

        // -- flock_summary ------------------------------------------------

        #[tokio::test]
        async fn flock_summary_success() {
            let mock = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(200).set_body_json(search_response(&[
                    ("F1", "Male"),
                    ("F2", "Female"),
                    ("F3", "Female"),
                ])))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .flock_summary(Parameters(FlockParams {
                    flock_id: "FLOCK1".to_string(),
                    breed_id: Some(640),
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["flock_id"], "FLOCK1");
            assert_eq!(json["sample_size"], 3);
            assert_eq!(json["males"], 1);
            assert_eq!(json["females"], 2);
        }

        #[tokio::test]
        async fn flock_summary_no_breed() {
            let mock = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(200).set_body_json(search_response(&[])))
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server
                .flock_summary(Parameters(FlockParams {
                    flock_id: "EMPTY".to_string(),
                    breed_id: None,
                }))
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["total_count"], 0);
            assert_eq!(json["sample_size"], 0);
        }

        // -- database_status -----------------------------------------------

        #[tokio::test]
        async fn database_status_success() {
            let mock = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(serde_json::json!("2024-12-15")),
                )
                .mount(&mock)
                .await;
            Mock::given(method("GET"))
                .and(path("/search/getStatusesByBreedGroup"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(serde_json::json!(["CURRENT", "SOLD", "DEAD"])),
                )
                .mount(&mock)
                .await;

            let server = mock_server(&mock.uri());
            let result = server.database_status().await.unwrap();

            assert!(!result.is_error.unwrap_or(false));
            let json: serde_json::Value =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
            assert_eq!(json["last_updated"], "2024-12-15");
            assert_eq!(json["statuses"].as_array().unwrap().len(), 3);
        }
    }
}
