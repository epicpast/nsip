//! MCP resource and resource template handlers for NSIP.
//!
//! Provides static resources (glossary, guides) and URI-template-based
//! dynamic resources backed by the NSIP API.

use rmcp::model::{
    AnnotateAble, ListResourceTemplatesResult, ListResourcesResult, RawResource,
    RawResourceTemplate, ReadResourceRequestParams, ReadResourceResult, ResourceContents, Role,
};

use crate::NsipClient;

use super::analytics::ebv_glossary;

/// Map a crate-level [`crate::Error`] into an MCP error with the RFC 9457
/// problem+json envelope in `data` and a class-appropriate JSON-RPC code.
/// See [`crate::mcp::problem_error`].
fn resource_err(context: &str, err: &crate::Error) -> rmcp::ErrorData {
    super::problem_error(context, err)
}

// ---------------------------------------------------------------------------
// URI parsing
// ---------------------------------------------------------------------------

/// Parsed NSIP resource URI.
#[derive(Debug)]
enum NsipUri {
    /// `nsip://glossary`
    Glossary,
    /// `nsip://breeds`
    Breeds,
    /// `nsip://guide/selection`
    SelectionGuide,
    /// `nsip://guide/inbreeding`
    InbreedingGuide,
    /// `nsip://status`
    Status,
    /// `nsip://animal/{lpn_id}`
    Animal { lpn_id: String },
    /// `nsip://animal/{lpn_id}/pedigree`
    AnimalPedigree { lpn_id: String },
    /// `nsip://animal/{lpn_id}/progeny`
    AnimalProgeny { lpn_id: String },
    /// `nsip://breed/{breed_id}/ranges`
    BreedRanges { breed_id: String },
    /// Unknown URI
    Unknown,
}

/// Parse an `nsip://` URI into a structured variant.
fn parse_nsip_uri(uri: &str) -> NsipUri {
    let path = uri.strip_prefix("nsip://").unwrap_or(uri);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    match segments.as_slice() {
        ["glossary"] => NsipUri::Glossary,
        ["breeds"] => NsipUri::Breeds,
        ["guide", "selection"] => NsipUri::SelectionGuide,
        ["guide", "inbreeding"] => NsipUri::InbreedingGuide,
        ["status"] => NsipUri::Status,
        ["animal", lpn_id, "pedigree"] => NsipUri::AnimalPedigree {
            lpn_id: (*lpn_id).to_string(),
        },
        ["animal", lpn_id, "progeny"] => NsipUri::AnimalProgeny {
            lpn_id: (*lpn_id).to_string(),
        },
        ["animal", lpn_id] => NsipUri::Animal {
            lpn_id: (*lpn_id).to_string(),
        },
        ["breed", breed_id, "ranges"] => NsipUri::BreedRanges {
            breed_id: (*breed_id).to_string(),
        },
        _ => NsipUri::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Static content generators
// ---------------------------------------------------------------------------

/// Generate the EBV trait glossary as markdown.
fn glossary_content() -> String {
    use std::fmt::Write;

    let mut md = String::from("# NSIP EBV Trait Glossary\n\n");
    md.push_str("Estimated Breeding Values (EBVs) predict the genetic merit an animal passes to its offspring.\n\n");

    for t in ebv_glossary() {
        let _ = writeln!(md, "## {} — {}", t.abbreviation, t.name);
        let _ = writeln!(md, "- **Unit:** {}", t.unit);
        let _ = writeln!(md, "- **Description:** {}", t.description);
        let _ = writeln!(md, "- **Selection:** {}\n", t.selection_direction);
    }

    md
}

/// Generate the selection guide as markdown.
fn selection_guide_content() -> String {
    String::from(
        "# NSIP Selection Guide\n\n\
         ## Using EBVs for Breeding Decisions\n\n\
         EBVs (Estimated Breeding Values) predict the genetic merit an animal transmits \
         to its offspring. They are expressed as deviations from a breed average.\n\n\
         ### Key Principles\n\n\
         1. **Compare within breed** — EBVs are only meaningful within a breed or breed group.\n\
         2. **Check accuracy** — Higher accuracy (%) means more reliable predictions. \
         Accuracy above 60% is considered moderate; above 80% is high.\n\
         3. **Use multiple traits** — Don't select on a single trait. Use selection indexes \
         or weight multiple traits based on your breeding objectives.\n\
         4. **Consider trade-offs** — Increasing birth weight EBV may increase dystocia. \
         Balance growth traits with maternal traits.\n\n\
         ### Selection Steps\n\n\
         1. Define your breeding objective (e.g., terminal sire, maternal flock).\n\
         2. Identify the 3-5 most important traits for your goal.\n\
         3. Set minimum accuracy thresholds (typically 40%+).\n\
         4. Rank candidates using weighted trait scores.\n\
         5. Check inbreeding (COI) before finalizing matings.\n\n\
         ### Common Breeding Objectives\n\n\
         - **Terminal sire:** Emphasize WWT, YWT, EMD, FAT.\n\
         - **Maternal flock:** Emphasize NLB, NWT, PWT, and moderate BWT.\n\
         - **Dual purpose:** Balance growth (WWT, YWT) with maternal traits (NLB, NWT).\n\
         - **Parasite resistance:** Emphasize WEC/FEC (lower is better).\n",
    )
}

/// Generate the inbreeding guide as markdown.
fn inbreeding_guide_content() -> String {
    String::from(
        "# NSIP Inbreeding Guide\n\n\
         ## Coefficient of Inbreeding (COI)\n\n\
         The COI measures the probability that two alleles at any gene are identical by descent. \
         It ranges from 0% (completely unrelated) to 100% (completely inbred).\n\n\
         ### COI Thresholds\n\n\
         | Rating | COI Range | Recommendation |\n\
         |--------|-----------|----------------|\n\
         | Green  | < 6.25%   | Acceptable — proceed with mating |\n\
         | Yellow | 6.25-12.5%| Caution — consider alternatives |\n\
         | Red    | > 12.5%   | Avoid — high inbreeding depression risk |\n\n\
         ### Reference Values\n\n\
         - **6.25%** = equivalent to mating half-siblings\n\
         - **12.5%** = equivalent to mating full siblings or parent-offspring\n\
         - **25%** = equivalent to mating identical twins\n\n\
         ### Inbreeding Depression Effects\n\n\
         - Reduced fertility and reproductive performance\n\
         - Lower lamb survival rates\n\
         - Decreased growth rates and mature size\n\
         - Weakened immune response\n\
         - Increased susceptibility to parasites\n\n\
         ### Avoidance Strategies\n\n\
         1. Always check COI before planned matings.\n\
         2. Use unrelated sires from different flocks.\n\
         3. Rotate ram usage across years.\n\
         4. Maintain pedigree records for at least 4 generations.\n\
         5. Consider genetic diversity when selecting replacement ewes.\n",
    )
}

// ---------------------------------------------------------------------------
// Public handlers
// ---------------------------------------------------------------------------

/// Build the list of static resources.
#[must_use]
pub fn list_resources() -> ListResourcesResult {
    let audience = vec![Role::User, Role::Assistant];

    let resources = vec![
        RawResource {
            uri: "nsip://glossary".to_string(),
            name: "EBV Trait Glossary".to_string(),
            title: Some("EBV Trait Glossary".to_string()),
            description: Some(
                "Definitions of all 13 NSIP EBV traits with units, interpretation, and selection direction"
                    .to_string(),
            ),
            mime_type: Some("text/markdown".to_string()),
            size: None,
            icons: None,
            meta: None,
        }
        .with_priority(0.8)
        .with_audience(audience.clone()),
        RawResource {
            uri: "nsip://breeds".to_string(),
            name: "Breed Directory".to_string(),
            title: Some("Breed Directory".to_string()),
            description: Some(
                "Live directory of all breed groups and breeds from the NSIP database".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
            size: None,
            icons: None,
            meta: None,
        }
        .with_priority(0.7)
        .with_audience(audience.clone()),
        RawResource {
            uri: "nsip://guide/selection".to_string(),
            name: "Selection Guide".to_string(),
            title: Some("Selection Guide".to_string()),
            description: Some(
                "How to use EBVs for breeding decisions — selection steps, objectives, and trade-offs"
                    .to_string(),
            ),
            mime_type: Some("text/markdown".to_string()),
            size: None,
            icons: None,
            meta: None,
        }
        .with_priority(0.8)
        .with_audience(audience.clone()),
        RawResource {
            uri: "nsip://guide/inbreeding".to_string(),
            name: "Inbreeding Guide".to_string(),
            title: Some("Inbreeding Guide".to_string()),
            description: Some(
                "COI thresholds, inbreeding depression effects, and avoidance strategies".to_string(),
            ),
            mime_type: Some("text/markdown".to_string()),
            size: None,
            icons: None,
            meta: None,
        }
        .with_priority(0.8)
        .with_audience(audience.clone()),
        RawResource {
            uri: "nsip://status".to_string(),
            name: "Database Status".to_string(),
            title: Some("Database Status".to_string()),
            description: Some(
                "Live status of the NSIP database including last update date".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
            size: None,
            icons: None,
            meta: None,
        }
        .with_priority(0.5)
        .with_audience(audience),
    ];

    ListResourcesResult {
        meta: None,
        resources,
        next_cursor: None,
    }
}

/// Build the list of resource templates.
#[must_use]
pub fn list_resource_templates() -> ListResourceTemplatesResult {
    let templates = vec![
        RawResourceTemplate {
            uri_template: "nsip://animal/{lpn_id}".to_string(),
            name: "Animal Profile".to_string(),
            title: Some("Animal Profile".to_string()),
            description: Some(
                "Full profile for a specific animal by LPN ID (details, lineage, progeny)"
                    .to_string(),
            ),
            mime_type: Some("application/json".to_string()),
            icons: None,
        }
        .no_annotation(),
        RawResourceTemplate {
            uri_template: "nsip://animal/{lpn_id}/pedigree".to_string(),
            name: "Animal Pedigree".to_string(),
            title: Some("Animal Pedigree".to_string()),
            description: Some("Pedigree / lineage tree for a specific animal".to_string()),
            mime_type: Some("application/json".to_string()),
            icons: None,
        }
        .no_annotation(),
        RawResourceTemplate {
            uri_template: "nsip://animal/{lpn_id}/progeny".to_string(),
            name: "Animal Progeny".to_string(),
            title: Some("Animal Progeny".to_string()),
            description: Some("Offspring list for a specific animal".to_string()),
            mime_type: Some("application/json".to_string()),
            icons: None,
        }
        .no_annotation(),
        RawResourceTemplate {
            uri_template: "nsip://breed/{breed_id}/ranges".to_string(),
            name: "Breed Trait Ranges".to_string(),
            title: Some("Breed Trait Ranges".to_string()),
            description: Some("Min/max trait value ranges for a specific breed".to_string()),
            mime_type: Some("application/json".to_string()),
            icons: None,
        }
        .no_annotation(),
    ];

    ListResourceTemplatesResult {
        meta: None,
        resource_templates: templates,
        next_cursor: None,
    }
}

/// Helper to create a JSON text resource content entry.
///
/// # Errors
///
/// Returns an MCP internal error if `value` cannot be serialized (e.g. a
/// non-finite float), rather than silently emitting empty content.
fn json_resource_content(
    uri: &str,
    value: &impl serde::Serialize,
) -> Result<ResourceContents, rmcp::ErrorData> {
    let json = serde_json::to_string_pretty(value).map_err(|e| {
        rmcp::ErrorData::internal_error(format!("Failed to serialize resource: {e}"), None)
    })?;
    Ok(ResourceContents::TextResourceContents {
        uri: uri.to_string(),
        mime_type: Some("application/json".to_string()),
        text: json,
        meta: None,
    })
}

/// Helper to create a markdown text resource content entry.
fn markdown_resource_content(uri: &str, text: String) -> ResourceContents {
    ResourceContents::TextResourceContents {
        uri: uri.to_string(),
        mime_type: Some("text/markdown".to_string()),
        text,
        meta: None,
    }
}

/// Read a specific resource by URI.
///
/// # Errors
///
/// Returns `McpError` if the URI is unknown or the API call fails.
pub async fn read_resource(
    client: &NsipClient,
    request: &ReadResourceRequestParams,
) -> Result<ReadResourceResult, rmcp::ErrorData> {
    let uri = &request.uri;

    match parse_nsip_uri(uri) {
        NsipUri::Glossary => Ok(ReadResourceResult::new(vec![markdown_resource_content(
            uri,
            glossary_content(),
        )])),

        NsipUri::SelectionGuide => Ok(ReadResourceResult::new(vec![markdown_resource_content(
            uri,
            selection_guide_content(),
        )])),

        NsipUri::InbreedingGuide => Ok(ReadResourceResult::new(vec![markdown_resource_content(
            uri,
            inbreeding_guide_content(),
        )])),

        NsipUri::Breeds => {
            let groups = client
                .breed_groups()
                .await
                .map_err(|e| resource_err("Failed to fetch breeds", &e))?;
            Ok(ReadResourceResult::new(vec![json_resource_content(
                uri, &groups,
            )?]))
        },

        NsipUri::Status => {
            let updated = client
                .date_last_updated()
                .await
                .map_err(|e| resource_err("Failed to fetch status", &e))?;
            let statuses = client
                .statuses()
                .await
                .map_err(|e| resource_err("Failed to fetch statuses", &e))?;
            let status_obj = serde_json::json!({
                "last_updated": updated.data,
                "statuses": statuses,
            });
            Ok(ReadResourceResult::new(vec![json_resource_content(
                uri,
                &status_obj,
            )?]))
        },

        NsipUri::Animal { lpn_id } => {
            let profile = client
                .search_by_lpn(&lpn_id)
                .await
                .map_err(|e| resource_err("Failed to fetch animal", &e))?;
            Ok(ReadResourceResult::new(vec![json_resource_content(
                uri, &profile,
            )?]))
        },

        NsipUri::AnimalPedigree { lpn_id } => {
            let lineage = client
                .lineage(&lpn_id)
                .await
                .map_err(|e| resource_err("Failed to fetch lineage", &e))?;
            Ok(ReadResourceResult::new(vec![json_resource_content(
                uri, &lineage,
            )?]))
        },

        NsipUri::AnimalProgeny { lpn_id } => {
            let progeny = client
                .progeny(&lpn_id, 0, 25)
                .await
                .map_err(|e| resource_err("Failed to fetch progeny", &e))?;
            Ok(ReadResourceResult::new(vec![json_resource_content(
                uri, &progeny,
            )?]))
        },

        NsipUri::BreedRanges { breed_id } => {
            let id: i64 = breed_id.parse().map_err(|_| {
                resource_err(
                    "breed-id",
                    &crate::Error::invalid_breed_id(format!("Invalid breed_id: {breed_id}")),
                )
            })?;
            let ranges = client
                .trait_ranges(id)
                .await
                .map_err(|e| resource_err("Failed to fetch ranges", &e))?;
            Ok(ReadResourceResult::new(vec![json_resource_content(
                uri, &ranges,
            )?]))
        },

        NsipUri::Unknown => {
            // Keep the MCP `resource_not_found` code (clients expect it for an
            // unknown resource read) but attach the RFC 9457 envelope so agents
            // still get the structured contract.
            let err = crate::Error::unknown_resource(format!("Unknown resource URI: {uri}"));
            let data = serde_json::to_value(err.to_problem_details("read-resource")).ok();
            Err(rmcp::ErrorData::resource_not_found(
                format!("Unknown resource URI: {uri}"),
                data,
            ))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // URI parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_static_uris() {
        assert!(matches!(
            parse_nsip_uri("nsip://glossary"),
            NsipUri::Glossary
        ));
        assert!(matches!(parse_nsip_uri("nsip://breeds"), NsipUri::Breeds));
        assert!(matches!(
            parse_nsip_uri("nsip://guide/selection"),
            NsipUri::SelectionGuide
        ));
        assert!(matches!(
            parse_nsip_uri("nsip://guide/inbreeding"),
            NsipUri::InbreedingGuide
        ));
        assert!(matches!(parse_nsip_uri("nsip://status"), NsipUri::Status));
    }

    #[test]
    fn parse_template_uris() {
        let animal = parse_nsip_uri("nsip://animal/ABC123");
        assert!(
            matches!(animal, NsipUri::Animal { ref lpn_id } if lpn_id == "ABC123"),
            "expected Animal, got {animal:?}"
        );

        let pedigree = parse_nsip_uri("nsip://animal/ABC123/pedigree");
        assert!(
            matches!(pedigree, NsipUri::AnimalPedigree { ref lpn_id } if lpn_id == "ABC123"),
            "expected AnimalPedigree, got {pedigree:?}"
        );

        let progeny = parse_nsip_uri("nsip://animal/ABC123/progeny");
        assert!(
            matches!(progeny, NsipUri::AnimalProgeny { ref lpn_id } if lpn_id == "ABC123"),
            "expected AnimalProgeny, got {progeny:?}"
        );

        let ranges = parse_nsip_uri("nsip://breed/486/ranges");
        assert!(
            matches!(ranges, NsipUri::BreedRanges { ref breed_id } if breed_id == "486"),
            "expected BreedRanges, got {ranges:?}"
        );
    }

    #[test]
    fn parse_unknown_uri() {
        assert!(matches!(
            parse_nsip_uri("nsip://something/else"),
            NsipUri::Unknown
        ));
        assert!(matches!(
            parse_nsip_uri("http://example.com"),
            NsipUri::Unknown
        ));
    }

    #[test]
    fn parse_uri_with_realistic_lpn_id() {
        let uri = parse_nsip_uri("nsip://animal/6400012006BWR107");
        assert!(
            matches!(uri, NsipUri::Animal { ref lpn_id } if lpn_id == "6400012006BWR107"),
            "expected Animal with realistic LPN ID"
        );
    }

    #[test]
    fn parse_uri_with_realistic_breed_id() {
        let uri = parse_nsip_uri("nsip://breed/640/ranges");
        assert!(
            matches!(uri, NsipUri::BreedRanges { ref breed_id } if breed_id == "640"),
            "expected BreedRanges with Katahdin breed ID"
        );
    }

    #[test]
    fn parse_uri_empty_string() {
        assert!(matches!(parse_nsip_uri(""), NsipUri::Unknown));
    }

    #[test]
    fn parse_uri_bare_scheme() {
        assert!(matches!(parse_nsip_uri("nsip://"), NsipUri::Unknown));
    }

    #[test]
    fn parse_uri_extra_path_segments() {
        assert!(matches!(
            parse_nsip_uri("nsip://animal/ABC/pedigree/extra"),
            NsipUri::Unknown
        ));
    }

    #[test]
    fn parse_uri_without_scheme() {
        // Without nsip:// prefix, still parsed via strip_prefix fallback
        let uri = parse_nsip_uri("glossary");
        assert!(matches!(uri, NsipUri::Glossary));
    }

    #[test]
    fn parse_uri_breed_without_ranges() {
        assert!(matches!(
            parse_nsip_uri("nsip://breed/640"),
            NsipUri::Unknown
        ));
    }

    // -----------------------------------------------------------------------
    // Static resource lists
    // -----------------------------------------------------------------------

    #[test]
    fn static_resources_list() {
        let result = list_resources();
        assert_eq!(result.resources.len(), 5);
    }

    #[test]
    fn static_resources_have_no_cursor() {
        let result = list_resources();
        assert!(result.next_cursor.is_none());
    }

    #[test]
    fn static_resources_have_correct_uris() {
        let result = list_resources();
        let uris: Vec<&str> = result
            .resources
            .iter()
            .map(|r| r.raw.uri.as_str())
            .collect();
        assert!(uris.contains(&"nsip://glossary"));
        assert!(uris.contains(&"nsip://breeds"));
        assert!(uris.contains(&"nsip://guide/selection"));
        assert!(uris.contains(&"nsip://guide/inbreeding"));
        assert!(uris.contains(&"nsip://status"));
    }

    #[test]
    fn static_resources_have_names_and_descriptions() {
        let result = list_resources();
        for r in &result.resources {
            assert!(!r.raw.name.is_empty(), "Resource missing name");
            assert!(
                r.raw.description.is_some(),
                "Resource {} missing description",
                r.raw.name
            );
        }
    }

    #[test]
    fn static_resources_have_mime_types() {
        let result = list_resources();
        for r in &result.resources {
            assert!(
                r.raw.mime_type.is_some(),
                "Resource {} missing mime_type",
                r.raw.name
            );
        }
    }

    #[test]
    fn markdown_resources_have_correct_mime_type() {
        let result = list_resources();
        let markdown_uris = [
            "nsip://glossary",
            "nsip://guide/selection",
            "nsip://guide/inbreeding",
        ];
        for r in &result.resources {
            if markdown_uris.contains(&r.raw.uri.as_str()) {
                assert_eq!(
                    r.raw.mime_type.as_deref(),
                    Some("text/markdown"),
                    "Resource {} should be text/markdown",
                    r.raw.uri
                );
            }
        }
    }

    #[test]
    fn json_resources_have_correct_mime_type() {
        let result = list_resources();
        let json_uris = ["nsip://breeds", "nsip://status"];
        for r in &result.resources {
            if json_uris.contains(&r.raw.uri.as_str()) {
                assert_eq!(
                    r.raw.mime_type.as_deref(),
                    Some("application/json"),
                    "Resource {} should be application/json",
                    r.raw.uri
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Resource templates
    // -----------------------------------------------------------------------

    #[test]
    fn resource_templates_list() {
        let result = list_resource_templates();
        assert_eq!(result.resource_templates.len(), 4);
    }

    #[test]
    fn resource_templates_have_no_cursor() {
        let result = list_resource_templates();
        assert!(result.next_cursor.is_none());
    }

    #[test]
    fn resource_templates_have_correct_uri_templates() {
        let result = list_resource_templates();
        let uris: Vec<&str> = result
            .resource_templates
            .iter()
            .map(|t| t.raw.uri_template.as_str())
            .collect();
        assert!(uris.contains(&"nsip://animal/{lpn_id}"));
        assert!(uris.contains(&"nsip://animal/{lpn_id}/pedigree"));
        assert!(uris.contains(&"nsip://animal/{lpn_id}/progeny"));
        assert!(uris.contains(&"nsip://breed/{breed_id}/ranges"));
    }

    #[test]
    fn resource_templates_have_names_and_descriptions() {
        let result = list_resource_templates();
        for t in &result.resource_templates {
            assert!(!t.raw.name.is_empty(), "Template missing name");
            assert!(
                t.raw.description.is_some(),
                "Template {} missing description",
                t.raw.name
            );
        }
    }

    #[test]
    fn resource_templates_all_json_mime_type() {
        let result = list_resource_templates();
        for t in &result.resource_templates {
            assert_eq!(
                t.raw.mime_type.as_deref(),
                Some("application/json"),
                "Template {} should be application/json",
                t.raw.name
            );
        }
    }

    // -----------------------------------------------------------------------
    // Static content generators
    // -----------------------------------------------------------------------

    #[test]
    fn glossary_content_has_all_traits() {
        let content = glossary_content();
        assert!(content.contains("BWT"));
        assert!(content.contains("WWT"));
        assert!(content.contains("FEC"));
        assert!(content.contains("Birth Weight"));
    }

    #[test]
    fn glossary_content_has_header() {
        let content = glossary_content();
        assert!(content.starts_with("# NSIP EBV Trait Glossary"));
    }

    #[test]
    fn glossary_content_has_all_thirteen_traits() {
        let content = glossary_content();
        let abbreviations = [
            "BWT", "WWT", "PWWT", "YWT", "FAT", "EMD", "NLB", "NWT", "PWT", "DAG", "WGR", "WEC",
            "FEC",
        ];
        for abbrev in &abbreviations {
            assert!(content.contains(abbrev), "Glossary missing trait: {abbrev}");
        }
    }

    #[test]
    fn glossary_content_includes_units_and_directions() {
        let content = glossary_content();
        assert!(content.contains("**Unit:**"));
        assert!(content.contains("**Description:**"));
        assert!(content.contains("**Selection:**"));
    }

    #[test]
    fn selection_guide_has_key_sections() {
        let content = selection_guide_content();
        assert!(content.starts_with("# NSIP Selection Guide"));
        assert!(content.contains("EBV"));
        assert!(content.contains("Selection Steps"));
        assert!(content.contains("Breeding Objectives"));
        assert!(content.contains("Terminal sire"));
        assert!(content.contains("Maternal flock"));
    }

    #[test]
    fn selection_guide_mentions_accuracy() {
        let content = selection_guide_content();
        assert!(
            content.contains("accuracy"),
            "Selection guide should discuss accuracy"
        );
    }

    #[test]
    fn inbreeding_guide_has_key_sections() {
        let content = inbreeding_guide_content();
        assert!(content.starts_with("# NSIP Inbreeding Guide"));
        assert!(content.contains("COI"));
        assert!(content.contains("Thresholds"));
        assert!(content.contains("6.25%"));
        assert!(content.contains("12.5%"));
    }

    #[test]
    fn inbreeding_guide_has_traffic_light_ratings() {
        let content = inbreeding_guide_content();
        assert!(content.contains("Green"));
        assert!(content.contains("Yellow"));
        assert!(content.contains("Red"));
    }

    #[test]
    fn inbreeding_guide_has_avoidance_strategies() {
        let content = inbreeding_guide_content();
        assert!(content.contains("Avoidance Strategies"));
        assert!(content.contains("pedigree"));
    }

    #[test]
    fn inbreeding_guide_has_depression_effects() {
        let content = inbreeding_guide_content();
        assert!(content.contains("Inbreeding Depression"));
        assert!(content.contains("fertility"));
    }

    // -----------------------------------------------------------------------
    // Content helpers
    // -----------------------------------------------------------------------

    /// Extract text from a `TextResourceContents` variant for assertions.
    fn extract_text_content(content: &ResourceContents) -> (&str, Option<&str>, &str) {
        match content {
            ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
                ..
            } => (uri, mime_type.as_deref(), text),
            ResourceContents::BlobResourceContents { .. } => {
                unreachable!("Expected TextResourceContents, got BlobResourceContents")
            },
        }
    }

    #[test]
    fn json_resource_content_produces_json_mime_type() {
        let data = serde_json::json!({"key": "value"});
        let content = json_resource_content("nsip://test", &data).expect("serialize");
        let (uri, mime, text) = extract_text_content(&content);
        assert_eq!(uri, "nsip://test");
        assert_eq!(mime, Some("application/json"));
        assert!(text.contains("key"));
        assert!(text.contains("value"));
    }

    #[test]
    fn json_resource_content_pretty_printed() {
        let data = serde_json::json!({"a": 1, "b": 2});
        let content = json_resource_content("nsip://test", &data).expect("serialize");
        let (_, _, text) = extract_text_content(&content);
        assert!(text.contains('\n'), "JSON should be pretty-printed");
    }

    #[test]
    fn markdown_resource_content_produces_markdown_mime_type() {
        let content = markdown_resource_content("nsip://test", "# Hello".to_string());
        let (uri, mime, text) = extract_text_content(&content);
        assert_eq!(uri, "nsip://test");
        assert_eq!(mime, Some("text/markdown"));
        assert_eq!(text, "# Hello");
    }

    // -----------------------------------------------------------------------
    // read_resource for static resources (no API calls)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn read_resource_glossary() {
        let client = NsipClient::new();
        let request = ReadResourceRequestParams::new("nsip://glossary");
        let res = read_resource(&client, &request).await.unwrap();
        assert_eq!(res.contents.len(), 1);
        let (_, mime, text) = extract_text_content(&res.contents[0]);
        assert_eq!(mime, Some("text/markdown"));
        assert!(text.contains("BWT"));
        assert!(text.contains("NSIP EBV Trait Glossary"));
    }

    #[tokio::test]
    async fn read_resource_selection_guide() {
        let client = NsipClient::new();
        let request = ReadResourceRequestParams::new("nsip://guide/selection");
        let res = read_resource(&client, &request).await.unwrap();
        assert_eq!(res.contents.len(), 1);
        let (_, mime, text) = extract_text_content(&res.contents[0]);
        assert_eq!(mime, Some("text/markdown"));
        assert!(text.contains("Selection Guide"));
    }

    #[tokio::test]
    async fn read_resource_inbreeding_guide() {
        let client = NsipClient::new();
        let request = ReadResourceRequestParams::new("nsip://guide/inbreeding");
        let res = read_resource(&client, &request).await.unwrap();
        assert_eq!(res.contents.len(), 1);
        let (_, mime, text) = extract_text_content(&res.contents[0]);
        assert_eq!(mime, Some("text/markdown"));
        assert!(text.contains("Inbreeding Guide"));
    }

    #[tokio::test]
    async fn read_resource_unknown_uri_returns_error() {
        let client = NsipClient::new();
        let request = ReadResourceRequestParams::new("nsip://nonexistent");
        let result = read_resource(&client, &request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unknown resource URI"));
    }

    #[tokio::test]
    async fn read_resource_invalid_breed_id_returns_error() {
        let client = NsipClient::new();
        let request = ReadResourceRequestParams::new("nsip://breed/not_a_number/ranges");
        let result = read_resource(&client, &request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid breed_id"));
    }

    // -- wiremock-based dynamic resource handler tests -------------------------

    mod wiremock_tests {
        use super::*;
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers::*};

        fn mock_client(uri: &str) -> NsipClient {
            NsipClient::with_base_url(uri)
        }

        /// Extract text and `mime_type` from the first [`ReadResourceResult`] content.
        fn extract_text(result: &ReadResourceResult) -> (&str, Option<&str>) {
            let ResourceContents::TextResourceContents {
                mime_type, text, ..
            } = &result.contents[0]
            else {
                unreachable!("Expected TextResourceContents");
            };
            (text.as_str(), mime_type.as_deref())
        }

        #[tokio::test]
        async fn read_resource_breeds() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getAvailableBreedGroups"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    {
                        "breedGroupId": 61,
                        "breedGroupName": "Hair",
                        "breeds": [{"breedId": 640, "breedName": "Katahdin"}]
                    }
                ])))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let request = ReadResourceRequestParams::new("nsip://breeds");
            let result = read_resource(&client, &request).await.unwrap();
            assert_eq!(result.contents.len(), 1);
            let (text, mime) = extract_text(&result);
            assert_eq!(mime, Some("application/json"));
            assert!(text.contains("Katahdin"));
        }

        #[tokio::test]
        async fn read_resource_status() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(serde_json::json!("2024-12-15")),
                )
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/search/getStatusesByBreedGroup"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(serde_json::json!(["CURRENT", "SOLD"])),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let request = ReadResourceRequestParams::new("nsip://status");
            let result = read_resource(&client, &request).await.unwrap();
            assert_eq!(result.contents.len(), 1);
            let (text, _) = extract_text(&result);
            assert!(text.contains("2024-12-15"));
            assert!(text.contains("CURRENT"));
        }

        #[tokio::test]
        async fn read_resource_animal() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "success": true,
                    "data": {
                        "gender": "Female",
                        "breed": {"breedName": "Katahdin", "breedId": 640},
                        "searchResultViewModel": {
                            "lpnId": "TEST1", "status": "CURRENT",
                            "bwt": 0.5, "accbwt": 80
                        }
                    }
                })))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": {"lpnId": "TEST1", "content": "<div>Farm</div>", "children": []}
                })))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "recordCount": 0, "records": []
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let request = ReadResourceRequestParams::new("nsip://animal/TEST1");
            let result = read_resource(&client, &request).await.unwrap();
            assert_eq!(result.contents.len(), 1);
            let (text, mime) = extract_text(&result);
            assert_eq!(mime, Some("application/json"));
            assert!(text.contains("TEST1"));
        }

        #[tokio::test]
        async fn read_resource_animal_pedigree() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": {
                        "lpnId": "PED1",
                        "content": "<div>Farm</div><div>DOB: 1/1/2020</div>",
                        "children": [
                            {"lpnId": "SIRE1", "content": "<div>Sire</div>", "children": []},
                            {"lpnId": "DAM1", "content": "<div>Dam</div>", "children": []}
                        ]
                    }
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let request = ReadResourceRequestParams::new("nsip://animal/PED1/pedigree");
            let result = read_resource(&client, &request).await.unwrap();
            let (text, _) = extract_text(&result);
            assert!(text.contains("PED1"));
            assert!(text.contains("SIRE1"));
        }

        #[tokio::test]
        async fn read_resource_animal_progeny() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "recordCount": 2,
                    "records": [
                        {"lpnId": "OFF1", "sex": "Male", "dob": "03/10/2023"},
                        {"lpnId": "OFF2", "sex": "Female", "dob": "03/12/2023"}
                    ]
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let request = ReadResourceRequestParams::new("nsip://animal/PARENT1/progeny");
            let result = read_resource(&client, &request).await.unwrap();
            let (text, _) = extract_text(&result);
            assert!(text.contains("OFF1"));
            assert!(text.contains("OFF2"));
        }

        #[tokio::test]
        async fn read_resource_breed_ranges() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .and(query_param("breedId", "640"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "rangeBWT": "-0.891 - 1.299",
                    "rangeWWT": "-3.022 - 6.153"
                })))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let request = ReadResourceRequestParams::new("nsip://breed/640/ranges");
            let result = read_resource(&client, &request).await.unwrap();
            let (text, mime) = extract_text(&result);
            assert_eq!(mime, Some("application/json"));
            assert!(text.contains("rangeBWT"));
        }

        // -- Error propagation tests (cover .map_err closures) ----------------

        #[tokio::test]
        async fn read_resource_breeds_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getAvailableBreedGroups"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let req = ReadResourceRequestParams::new("nsip://breeds");
            let err = read_resource(&client, &req).await.unwrap_err();
            assert!(err.message.contains("Failed to fetch breeds"));
        }

        #[tokio::test]
        async fn read_resource_status_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getDateLastUpdated"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let req = ReadResourceRequestParams::new("nsip://status");
            let err = read_resource(&client, &req).await.unwrap_err();
            assert!(err.message.contains("Failed to fetch status"));
        }

        #[tokio::test]
        async fn read_resource_animal_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let req = ReadResourceRequestParams::new("nsip://animal/X1");
            let err = read_resource(&client, &req).await.unwrap_err();
            assert!(err.message.contains("Failed to fetch animal"));
        }

        #[tokio::test]
        async fn read_resource_pedigree_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let req = ReadResourceRequestParams::new("nsip://animal/X1/pedigree");
            let err = read_resource(&client, &req).await.unwrap_err();
            assert!(err.message.contains("Failed to fetch lineage"));
        }

        #[tokio::test]
        async fn read_resource_progeny_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getPageOfProgeny"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let req = ReadResourceRequestParams::new("nsip://animal/X1/progeny");
            let err = read_resource(&client, &req).await.unwrap_err();
            assert!(err.message.contains("Failed to fetch progeny"));
        }

        #[tokio::test]
        async fn read_resource_breed_ranges_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let req = ReadResourceRequestParams::new("nsip://breed/640/ranges");
            let err = read_resource(&client, &req).await.unwrap_err();
            assert!(err.message.contains("Failed to fetch ranges"));
        }
    }
}
