//! MCP prompt implementations for guided breeding workflows.
//!
//! Each prompt fetches live data from the NSIP API and returns structured
//! `PromptMessage` arrays that give an LLM the context needed to provide
//! breeding advice.

use std::{fmt::Write, hash::BuildHasher};

use rmcp::model::{
    GetPromptResult, ListPromptsResult, Prompt, PromptArgument, PromptMessage, PromptMessageRole,
};

use crate::NsipClient;

// ---------------------------------------------------------------------------
// Prompt definitions
// ---------------------------------------------------------------------------

/// Build the list of available prompts.
#[must_use]
pub fn list_prompts() -> ListPromptsResult {
    let prompts = vec![
        prompt_evaluate_ram(),
        prompt_evaluate_ewe(),
        prompt_compare_breeding_stock(),
        prompt_plan_mating(),
        prompt_flock_improvement(),
        prompt_select_replacement(),
        prompt_interpret_ebvs(),
    ];

    ListPromptsResult {
        meta: None,
        prompts,
        next_cursor: None,
    }
}

fn prompt_evaluate_ram() -> Prompt {
    Prompt::new(
        "evaluate-ram",
        Some(
            "Evaluate a ram's breeding value — fetches EBVs, breed ranges, and constructs a comprehensive assessment",
        ),
        Some(vec![
            PromptArgument::new("lpn_id")
                .with_title("LPN ID")
                .with_description("LPN ID of the ram to evaluate")
                .with_required(true),
        ]),
    )
}

fn prompt_evaluate_ewe() -> Prompt {
    Prompt::new(
        "evaluate-ewe",
        Some("Evaluate a ewe's breeding value — emphasizes maternal traits (NLB, NWT, PWT)"),
        Some(vec![
            PromptArgument::new("lpn_id")
                .with_title("LPN ID")
                .with_description("LPN ID of the ewe to evaluate")
                .with_required(true),
        ]),
    )
}

fn prompt_compare_breeding_stock() -> Prompt {
    Prompt::new(
        "compare-breeding-stock",
        Some(
            "Compare multiple animals side-by-side with trait analysis and breeding recommendations",
        ),
        Some(vec![
            PromptArgument::new("animal_ids")
                .with_title("Animal IDs")
                .with_description("Comma-separated LPN IDs of animals to compare (2-5)")
                .with_required(true),
        ]),
    )
}

fn prompt_plan_mating() -> Prompt {
    Prompt::new(
        "plan-mating",
        Some("Plan a specific mating — COI check, trait complementarity, and offspring prediction"),
        Some(vec![
            PromptArgument::new("sire_id")
                .with_title("Sire LPN ID")
                .with_description("LPN ID of the sire")
                .with_required(true),
            PromptArgument::new("dam_id")
                .with_title("Dam LPN ID")
                .with_description("LPN ID of the dam")
                .with_required(true),
        ]),
    )
}

fn prompt_flock_improvement() -> Prompt {
    Prompt::new(
        "flock-improvement",
        Some("Analyze a breed or flock for trait gaps and improvement opportunities"),
        Some(vec![
            PromptArgument::new("breed_id")
                .with_title("Breed ID")
                .with_description("Breed ID to analyze")
                .with_required(true),
            PromptArgument::new("flock_id")
                .with_title("Flock ID")
                .with_description("Optional flock ID to narrow analysis")
                .with_required(false),
        ]),
    )
}

fn prompt_select_replacement() -> Prompt {
    Prompt::new(
        "select-replacement",
        Some("Find top replacement candidates within a breed by gender and target trait"),
        Some(vec![
            PromptArgument::new("breed_id")
                .with_title("Breed ID")
                .with_description("Breed ID to search within")
                .with_required(true),
            PromptArgument::new("gender")
                .with_title("Gender")
                .with_description("Gender of replacement animals (Male or Female)")
                .with_required(true),
            PromptArgument::new("target_trait")
                .with_title("Target Trait")
                .with_description("Primary trait to optimize (e.g. WWT, NLB)")
                .with_required(true),
        ]),
    )
}

fn prompt_interpret_ebvs() -> Prompt {
    Prompt::new(
        "interpret-ebvs",
        Some("Plain-language explanation of an animal's EBVs with breed-relative context"),
        Some(vec![
            PromptArgument::new("lpn_id")
                .with_title("LPN ID")
                .with_description("LPN ID of the animal to interpret")
                .with_required(true),
        ]),
    )
}

// ---------------------------------------------------------------------------
// Prompt execution
// ---------------------------------------------------------------------------

/// Type alias for the MCP request context used by elicitation-capable prompts.
pub(crate) type PromptContext<'a> =
    Option<&'a rmcp::service::RequestContext<rmcp::service::RoleServer>>;

/// Execute a prompt by name with the given arguments.
///
/// The `context` parameter is forwarded to prompt handlers that support
/// elicitation, allowing them to request structured input from the user.
///
/// # Errors
///
/// Returns `McpError` if the prompt name is unknown or API calls fail.
pub async fn get_prompt<S: BuildHasher + Sync>(
    client: &NsipClient,
    name: &str,
    arguments: &std::collections::HashMap<String, String, S>,
    context: PromptContext<'_>,
) -> Result<GetPromptResult, rmcp::ErrorData> {
    match name {
        "evaluate-ram" => evaluate_animal(client, arguments, AnimalType::Ram).await,
        "evaluate-ewe" => evaluate_animal(client, arguments, AnimalType::Ewe).await,
        "compare-breeding-stock" => compare_breeding_stock(client, arguments, context).await,
        "plan-mating" => plan_mating(client, arguments, context).await,
        "flock-improvement" => flock_improvement(client, arguments, context).await,
        "select-replacement" => select_replacement(client, arguments, context).await,
        "interpret-ebvs" => interpret_ebvs(client, arguments).await,
        _ => Err(rmcp::ErrorData::invalid_params(
            format!("Unknown prompt: {name}"),
            None,
        )),
    }
}

// ---------------------------------------------------------------------------
// Internal prompt builders
// ---------------------------------------------------------------------------

enum AnimalType {
    Ram,
    Ewe,
}

/// Evaluate a ram or ewe — fetch profile and breed ranges, build assessment prompt.
async fn evaluate_animal<S: BuildHasher + Sync>(
    client: &NsipClient,
    args: &std::collections::HashMap<String, String, S>,
    animal_type: AnimalType,
) -> Result<GetPromptResult, rmcp::ErrorData> {
    let lpn_id = require_arg(args, "lpn_id")?;

    let details = client.animal_details(lpn_id).await.map_err(|e| {
        rmcp::ErrorData::internal_error(format!("Failed to fetch animal: {e}"), None)
    })?;

    let details_json = serde_json::to_string_pretty(&details).unwrap_or_default();

    let (type_name, emphasis) = match animal_type {
        AnimalType::Ram => (
            "ram",
            "Focus on growth traits (WWT, YWT, EMD) and carcass quality (FAT). \
             Consider his value as a terminal sire vs. maternal sire.",
        ),
        AnimalType::Ewe => (
            "ewe",
            "Focus on maternal traits (NLB, NWT, PWT) and moderate birth weight (BWT). \
             Consider her prolificacy, lamb-rearing ability, and longevity potential.",
        ),
    };

    let system_msg = format!(
        "You are a sheep breeding advisor evaluating a {type_name}. \
         Analyze the EBV data below and provide a comprehensive breeding assessment.\n\n\
         {emphasis}\n\n\
         Rate each trait relative to breed averages. \
         Identify strengths, weaknesses, and recommended uses.\n\n\
         ## Animal Data\n\n```json\n{details_json}\n```"
    );

    let user_msg = format!(
        "Please evaluate this {type_name} ({lpn_id}) for breeding. \
         What are its key strengths and weaknesses? \
         How should it be used in a breeding program?"
    );

    Ok(GetPromptResult::new(vec![
        PromptMessage::new_text(PromptMessageRole::User, system_msg),
        PromptMessage::new_text(PromptMessageRole::User, user_msg),
    ])
    .with_description(format!("Breeding evaluation for {type_name} {lpn_id}")))
}

/// Compare multiple animals side-by-side.
async fn compare_breeding_stock<S: BuildHasher + Sync>(
    client: &NsipClient,
    args: &std::collections::HashMap<String, String, S>,
    context: PromptContext<'_>,
) -> Result<GetPromptResult, rmcp::ErrorData> {
    let ids_str = require_arg(args, "animal_ids")?;
    let ids: Vec<&str> = ids_str.split(',').map(str::trim).collect();

    if ids.len() < 2 || ids.len() > 5 {
        return Err(rmcp::ErrorData::invalid_params(
            "animal_ids must contain 2-5 comma-separated LPN IDs",
            None,
        ));
    }

    // Elicit trait preferences if the client supports it.
    let prefs = if let Some(ctx) = context {
        super::elicitation::try_elicit::<super::elicitation::ComparePreferences>(
            ctx,
            "Which traits should the comparison focus on?",
        )
        .await
    } else {
        None
    };

    let mut animals_data = Vec::new();
    for id in &ids {
        let details = client.animal_details(id).await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to fetch {id}: {e}"), None)
        })?;
        animals_data.push(details);
    }

    let data_json = serde_json::to_string_pretty(&animals_data).unwrap_or_default();

    let trait_focus = prefs.and_then(|p| p.traits).map_or_else(String::new, |t| {
        format!("\n\nFocus especially on these traits: {t}")
    });

    let system_msg = format!(
        "You are a sheep breeding advisor comparing {} animals side-by-side. \
         Analyze the EBV data below and provide:\n\
         1. A trait-by-trait comparison highlighting differences\n\
         2. Each animal's relative strengths and weaknesses\n\
         3. Which animal is best suited for different breeding goals\n\
         4. Any trade-offs to consider{trait_focus}\n\n\
         ## Animal Data\n\n```json\n{data_json}\n```",
        ids.len()
    );

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        system_msg,
    )])
    .with_description(format!("Comparison of {} breeding animals", ids.len())))
}

/// Plan a specific mating.
async fn plan_mating<S: BuildHasher + Sync>(
    client: &NsipClient,
    args: &std::collections::HashMap<String, String, S>,
    context: PromptContext<'_>,
) -> Result<GetPromptResult, rmcp::ErrorData> {
    let sire_id = require_arg(args, "sire_id")?;
    let dam_id = require_arg(args, "dam_id")?;

    let (sire_details, dam_details, sire_lineage, dam_lineage) = tokio::join!(
        client.animal_details(sire_id),
        client.animal_details(dam_id),
        client.lineage(sire_id),
        client.lineage(dam_id),
    );

    let sire_details = sire_details
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to fetch sire: {e}"), None))?;
    let dam_details = dam_details
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to fetch dam: {e}"), None))?;
    let sire_lineage = sire_lineage
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Sire lineage failed: {e}"), None))?;
    let dam_lineage = dam_lineage
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Dam lineage failed: {e}"), None))?;

    let coi = super::analytics::calculate_coi(&sire_lineage, &dam_lineage);
    let complementarity = super::analytics::trait_complementarity(&sire_details, &dam_details);

    let constraints = if let Some(ctx) = context {
        super::elicitation::try_elicit::<super::elicitation::MatingConstraints>(
            ctx,
            "Any breeding constraints for this mating? (max COI, breeding objective)",
        )
        .await
    } else {
        None
    };

    let mating_data = serde_json::json!({
        "sire": sire_details,
        "dam": dam_details,
        "inbreeding": {
            "coefficient": coi.coefficient,
            "percentage": format!("{:.2}%", coi.coefficient * 100.0),
            "rating": coi.rating,
            "shared_ancestors": coi.shared_ancestors,
        },
        "predicted_offspring_ebvs": complementarity,
        "user_constraints": constraints.map(|c| serde_json::json!({
            "max_coi": c.max_coi,
            "breeding_objective": c.breeding_objective,
        })),
    });

    let data_json = serde_json::to_string_pretty(&mating_data).unwrap_or_default();

    let system_msg = format!(
        "You are a sheep breeding advisor evaluating a planned mating. \
         Analyze the sire, dam, inbreeding coefficient, and predicted offspring EBVs below.\n\n\
         Provide:\n\
         1. Inbreeding assessment — is this mating safe?\n\
         2. Predicted offspring quality based on midparent EBVs\n\
         3. Strengths this pairing would combine\n\
         4. Weaknesses or risks to watch for\n\
         5. Overall recommendation: proceed, caution, or avoid\n\n\
         ## Mating Analysis Data\n\n```json\n{data_json}\n```"
    );

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        system_msg,
    )])
    .with_description(format!("Mating plan: {sire_id} x {dam_id}")))
}

/// Analyze a breed or flock for improvement opportunities.
async fn flock_improvement<S: BuildHasher + Sync>(
    client: &NsipClient,
    args: &std::collections::HashMap<String, String, S>,
    context: PromptContext<'_>,
) -> Result<GetPromptResult, rmcp::ErrorData> {
    let breed_id_str = require_arg(args, "breed_id")?;
    let breed_id: i64 = breed_id_str.parse().map_err(|_| {
        rmcp::ErrorData::invalid_params(format!("Invalid breed_id: {breed_id_str}"), None)
    })?;

    let flock_id = args.get("flock_id");

    let mut criteria = crate::SearchCriteria::new()
        .with_breed_id(breed_id)
        .with_status("CURRENT");
    if let Some(fid) = flock_id {
        criteria = criteria.with_flock_id(fid);
    }

    let (results, ranges) = tokio::join!(
        client.search_animals(0, 50, Some(breed_id), None, None, Some(&criteria)),
        client.trait_ranges(breed_id),
    );

    let results = results
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Search failed: {e}"), None))?;
    let ranges = ranges
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Trait ranges failed: {e}"), None))?;

    let flock_ctx = if let Some(ctx) = context {
        super::elicitation::try_elicit::<super::elicitation::FlockContext>(
            ctx,
            "Tell us about your flock goals (breeding objective, flock size)",
        )
        .await
    } else {
        None
    };

    let flock_data = serde_json::json!({
        "total_animals": results.total_count,
        "sample_size": results.results.len(),
        "breed_trait_ranges": ranges,
        "animals_sample": results.results,
        "user_context": flock_ctx.map(|c| serde_json::json!({
            "breeding_objective": c.breeding_objective,
            "flock_size": c.flock_size,
        })),
    });

    let data_json = serde_json::to_string_pretty(&flock_data).unwrap_or_default();
    let scope = flock_id.map_or_else(|| "breed".to_string(), |f| format!("flock {f}"));

    let system_msg = format!(
        "You are a sheep breeding advisor analyzing a {scope} for improvement opportunities. \
         Review the animal data and breed trait ranges below.\n\n\
         Provide:\n\
         1. Current trait averages vs breed ranges\n\
         2. Trait gaps — where the group is below breed average\n\
         3. Strengths — where the group excels\n\
         4. Prioritized improvement recommendations\n\
         5. Suggested sire selection criteria\n\n\
         ## Flock/Breed Analysis Data\n\n```json\n{data_json}\n```"
    );

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        system_msg,
    )])
    .with_description(format!("Flock improvement analysis for {scope}")))
}

/// Find top replacement candidates.
async fn select_replacement<S: BuildHasher + Sync>(
    client: &NsipClient,
    args: &std::collections::HashMap<String, String, S>,
    context: PromptContext<'_>,
) -> Result<GetPromptResult, rmcp::ErrorData> {
    let breed_id_str = require_arg(args, "breed_id")?;
    let breed_id: i64 = breed_id_str.parse().map_err(|_| {
        rmcp::ErrorData::invalid_params(format!("Invalid breed_id: {breed_id_str}"), None)
    })?;
    let gender = require_arg(args, "gender")?;
    let target_trait = require_arg(args, "target_trait")?;

    let selection = if let Some(ctx) = context {
        super::elicitation::try_elicit::<super::elicitation::SelectionCriteria>(
            ctx,
            "Any selection criteria? (minimum accuracy, priority traits)",
        )
        .await
    } else {
        None
    };

    let criteria = crate::SearchCriteria::new()
        .with_breed_id(breed_id)
        .with_gender(gender)
        .with_status("CURRENT");

    let results = client
        .search_animals(
            0,
            50,
            Some(breed_id),
            Some(target_trait),
            None,
            Some(&criteria),
        )
        .await
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Search failed: {e}"), None))?;

    let candidate_data = serde_json::json!({
        "breed_id": breed_id,
        "gender": gender,
        "target_trait": target_trait,
        "total_candidates": results.total_count,
        "top_candidates": results.results,
        "user_criteria": selection.map(|c| serde_json::json!({
            "min_accuracy": c.min_accuracy,
            "priority_traits": c.priority_traits,
        })),
    });

    let data_json = serde_json::to_string_pretty(&candidate_data).unwrap_or_default();

    let system_msg = format!(
        "You are a sheep breeding advisor helping select replacement {gender}s. \
         The farmer wants to prioritize {target_trait}.\n\n\
         Review the candidates below and provide:\n\
         1. Top 5 recommended replacements with reasoning\n\
         2. Key traits to consider alongside {target_trait}\n\
         3. Any trade-offs or risks with top candidates\n\
         4. Selection criteria summary\n\n\
         ## Replacement Candidates\n\n```json\n{data_json}\n```"
    );

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        system_msg,
    )])
    .with_description(format!("Replacement {gender} selection for {target_trait}")))
}

/// Plain-language EBV interpretation.
async fn interpret_ebvs<S: BuildHasher + Sync>(
    client: &NsipClient,
    args: &std::collections::HashMap<String, String, S>,
) -> Result<GetPromptResult, rmcp::ErrorData> {
    let lpn_id = require_arg(args, "lpn_id")?;

    let details = client.animal_details(lpn_id).await.map_err(|e| {
        rmcp::ErrorData::internal_error(format!("Failed to fetch animal: {e}"), None)
    })?;

    let details_json = serde_json::to_string_pretty(&details).unwrap_or_default();

    let glossary = super::analytics::ebv_glossary();
    let mut glossary_text = String::new();
    for t in &glossary {
        let _ = writeln!(
            glossary_text,
            "- **{}** ({}): {} [{}]",
            t.abbreviation, t.name, t.description, t.selection_direction
        );
    }

    let system_msg = format!(
        "You are a sheep breeding advisor explaining EBVs to a farmer in plain language. \
         Avoid jargon. Use practical terms like 'heavier lambs at weaning' instead of \
         'higher WWT EBV'.\n\n\
         ## Trait Glossary\n\n{glossary_text}\n\
         ## Animal Data\n\n```json\n{details_json}\n```\n\n\
         Please explain each of this animal's EBV traits in farmer-friendly language. \
         Compare to breed averages where possible. \
         Summarize what this animal would contribute to a breeding program."
    );

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        system_msg,
    )])
    .with_description(format!("EBV interpretation for {lpn_id}")))
}

// ---------------------------------------------------------------------------
// Argument helpers
// ---------------------------------------------------------------------------

/// Extract a required argument by name.
fn require_arg<'a, S: BuildHasher>(
    args: &'a std::collections::HashMap<String, String, S>,
    name: &str,
) -> Result<&'a str, rmcp::ErrorData> {
    args.get(name).map(String::as_str).ok_or_else(|| {
        rmcp::ErrorData::invalid_params(format!("Missing required argument: {name}"), None)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // list_prompts
    // -----------------------------------------------------------------------

    #[test]
    fn prompts_list_has_seven_entries() {
        let result = list_prompts();
        assert_eq!(result.prompts.len(), 7);
    }

    #[test]
    fn prompts_have_names_and_descriptions() {
        let result = list_prompts();
        for prompt in &result.prompts {
            assert!(!prompt.name.is_empty());
            assert!(prompt.description.is_some());
        }
    }

    #[test]
    fn list_prompts_has_no_cursor() {
        let result = list_prompts();
        assert!(result.next_cursor.is_none());
    }

    #[test]
    fn list_prompts_has_no_meta() {
        let result = list_prompts();
        assert!(result.meta.is_none());
    }

    #[test]
    fn prompt_names_are_unique() {
        let result = list_prompts();
        let names: Vec<&str> = result.prompts.iter().map(|p| p.name.as_ref()).collect();
        let mut deduped = names.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(names.len(), deduped.len(), "Duplicate prompt names found");
    }

    #[test]
    fn all_expected_prompt_names_present() {
        let result = list_prompts();
        let names: Vec<&str> = result.prompts.iter().map(|p| p.name.as_ref()).collect();
        let expected = [
            "evaluate-ram",
            "evaluate-ewe",
            "compare-breeding-stock",
            "plan-mating",
            "flock-improvement",
            "select-replacement",
            "interpret-ebvs",
        ];
        for name in &expected {
            assert!(names.contains(name), "Missing prompt: {name}");
        }
    }

    // -----------------------------------------------------------------------
    // Individual prompt definitions
    // -----------------------------------------------------------------------

    #[test]
    fn evaluate_ram_has_required_argument() {
        let result = list_prompts();
        let ram_prompt = result.prompts.iter().find(|p| p.name == "evaluate-ram");
        assert!(ram_prompt.is_some());
        let args = ram_prompt.unwrap().arguments.as_ref().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "lpn_id");
        assert_eq!(args[0].required, Some(true));
    }

    #[test]
    fn evaluate_ewe_has_required_lpn_argument() {
        let prompt = prompt_evaluate_ewe();
        assert_eq!(prompt.name, "evaluate-ewe");
        let args = prompt.arguments.unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "lpn_id");
        assert_eq!(args[0].required, Some(true));
        assert!(args[0].description.is_some());
        assert!(args[0].title.is_some());
    }

    #[test]
    fn evaluate_ewe_description_mentions_maternal() {
        let prompt = prompt_evaluate_ewe();
        let desc = prompt.description.unwrap();
        assert!(
            desc.contains("maternal"),
            "Ewe prompt should mention maternal traits"
        );
    }

    #[test]
    fn compare_breeding_stock_has_animal_ids_argument() {
        let prompt = prompt_compare_breeding_stock();
        assert_eq!(prompt.name, "compare-breeding-stock");
        let args = prompt.arguments.unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "animal_ids");
        assert_eq!(args[0].required, Some(true));
        let desc = args[0].description.as_deref().unwrap();
        assert!(desc.contains("2-5"), "Should mention 2-5 animals");
    }

    #[test]
    fn plan_mating_has_two_arguments() {
        let result = list_prompts();
        let mating_prompt = result.prompts.iter().find(|p| p.name == "plan-mating");
        assert!(mating_prompt.is_some());
        let args = mating_prompt.unwrap().arguments.as_ref().unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, "sire_id");
        assert_eq!(args[1].name, "dam_id");
    }

    #[test]
    fn plan_mating_both_arguments_required() {
        let prompt = prompt_plan_mating();
        let args = prompt.arguments.unwrap();
        for arg in &args {
            assert_eq!(
                arg.required,
                Some(true),
                "Arg {} should be required",
                arg.name
            );
        }
    }

    #[test]
    fn flock_improvement_has_required_and_optional_args() {
        let prompt = prompt_flock_improvement();
        assert_eq!(prompt.name, "flock-improvement");
        let args = prompt.arguments.unwrap();
        assert_eq!(args.len(), 2);

        let breed_arg = args.iter().find(|a| a.name == "breed_id").unwrap();
        assert_eq!(breed_arg.required, Some(true));

        let flock_arg = args.iter().find(|a| a.name == "flock_id").unwrap();
        assert_eq!(flock_arg.required, Some(false));
    }

    #[test]
    fn select_replacement_has_three_required_args() {
        let prompt = prompt_select_replacement();
        assert_eq!(prompt.name, "select-replacement");
        let args = prompt.arguments.unwrap();
        assert_eq!(args.len(), 3);

        let expected_names = ["breed_id", "gender", "target_trait"];
        for name in &expected_names {
            let arg = args.iter().find(|a| a.name == *name);
            assert!(arg.is_some(), "Missing argument: {name}");
            assert_eq!(
                arg.unwrap().required,
                Some(true),
                "Arg {name} should be required"
            );
        }
    }

    #[test]
    fn interpret_ebvs_has_required_lpn_argument() {
        let prompt = prompt_interpret_ebvs();
        assert_eq!(prompt.name, "interpret-ebvs");
        let args = prompt.arguments.unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "lpn_id");
        assert_eq!(args[0].required, Some(true));
    }

    // -----------------------------------------------------------------------
    // require_arg helper
    // -----------------------------------------------------------------------

    #[test]
    fn require_arg_returns_error_for_missing() {
        let args = std::collections::HashMap::new();
        let result = require_arg(&args, "missing");
        assert!(result.is_err());
    }

    #[test]
    fn require_arg_error_message_contains_param_name() {
        let args: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let err = require_arg(&args, "lpn_id").unwrap_err();
        let msg = err.message;
        assert!(
            msg.contains("lpn_id"),
            "Error should mention the missing arg name"
        );
    }

    #[test]
    fn require_arg_returns_value() {
        let mut args = std::collections::HashMap::new();
        args.insert("lpn_id".to_string(), "ABC123".to_string());
        let result = require_arg(&args, "lpn_id");
        assert_eq!(result.unwrap(), "ABC123");
    }

    #[test]
    fn require_arg_returns_correct_value_among_many() {
        let mut args = std::collections::HashMap::new();
        args.insert("sire_id".to_string(), "SIRE001".to_string());
        args.insert("dam_id".to_string(), "DAM002".to_string());
        args.insert("breed_id".to_string(), "640".to_string());
        assert_eq!(require_arg(&args, "sire_id").unwrap(), "SIRE001");
        assert_eq!(require_arg(&args, "dam_id").unwrap(), "DAM002");
        assert_eq!(require_arg(&args, "breed_id").unwrap(), "640");
    }

    #[test]
    fn require_arg_empty_string_is_valid() {
        let mut args = std::collections::HashMap::new();
        args.insert("lpn_id".to_string(), String::new());
        let result = require_arg(&args, "lpn_id");
        assert_eq!(result.unwrap(), "");
    }

    // -----------------------------------------------------------------------
    // get_prompt dispatcher
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn get_prompt_unknown_name_returns_error() {
        let client = NsipClient::new();
        let args = std::collections::HashMap::new();
        let result = get_prompt(&client, "nonexistent-prompt", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("Unknown prompt"),
            "Error should mention unknown prompt"
        );
    }

    #[tokio::test]
    async fn get_prompt_missing_required_arg_returns_error() {
        let client = NsipClient::new();
        let args = std::collections::HashMap::new();
        // evaluate-ram requires lpn_id
        let result = get_prompt(&client, "evaluate-ram", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("lpn_id"),
            "Error should mention missing lpn_id"
        );
    }

    #[tokio::test]
    async fn get_prompt_compare_too_few_ids_returns_error() {
        let client = NsipClient::new();
        let mut args = std::collections::HashMap::new();
        args.insert("animal_ids".to_string(), "SINGLE_ID".to_string());
        let result = get_prompt(&client, "compare-breeding-stock", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("2-5"),
            "Error should mention 2-5 animals required"
        );
    }

    #[tokio::test]
    async fn get_prompt_compare_too_many_ids_returns_error() {
        let client = NsipClient::new();
        let mut args = std::collections::HashMap::new();
        args.insert("animal_ids".to_string(), "A,B,C,D,E,F".to_string());
        let result = get_prompt(&client, "compare-breeding-stock", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("2-5"),
            "Error should mention 2-5 animals required"
        );
    }

    #[tokio::test]
    async fn get_prompt_flock_improvement_invalid_breed_id() {
        let client = NsipClient::new();
        let mut args = std::collections::HashMap::new();
        args.insert("breed_id".to_string(), "not_a_number".to_string());
        let result = get_prompt(&client, "flock-improvement", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("Invalid breed_id"),
            "Error should mention invalid breed_id"
        );
    }

    #[tokio::test]
    async fn get_prompt_select_replacement_invalid_breed_id() {
        let client = NsipClient::new();
        let mut args = std::collections::HashMap::new();
        args.insert("breed_id".to_string(), "abc".to_string());
        args.insert("gender".to_string(), "Male".to_string());
        args.insert("target_trait".to_string(), "WWT".to_string());
        let result = get_prompt(&client, "select-replacement", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid breed_id"));
    }

    #[tokio::test]
    async fn get_prompt_select_replacement_missing_gender() {
        let client = NsipClient::new();
        let mut args = std::collections::HashMap::new();
        args.insert("breed_id".to_string(), "640".to_string());
        // missing gender and target_trait
        let result = get_prompt(&client, "select-replacement", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("gender"));
    }

    #[tokio::test]
    async fn get_prompt_evaluate_ewe_missing_lpn() {
        let client = NsipClient::new();
        let args = std::collections::HashMap::new();
        let result = get_prompt(&client, "evaluate-ewe", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("lpn_id"));
    }

    #[tokio::test]
    async fn get_prompt_interpret_ebvs_missing_lpn() {
        let client = NsipClient::new();
        let args = std::collections::HashMap::new();
        let result = get_prompt(&client, "interpret-ebvs", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("lpn_id"));
    }

    #[tokio::test]
    async fn get_prompt_plan_mating_missing_dam() {
        let client = NsipClient::new();
        let mut args = std::collections::HashMap::new();
        args.insert("sire_id".to_string(), "SIRE001".to_string());
        // missing dam_id
        let result = get_prompt(&client, "plan-mating", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("dam_id"));
    }

    #[tokio::test]
    async fn get_prompt_plan_mating_missing_sire() {
        let client = NsipClient::new();
        let args = std::collections::HashMap::new();
        // missing both
        let result = get_prompt(&client, "plan-mating", &args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("sire_id"));
    }

    // -----------------------------------------------------------------------
    // Prompt argument metadata
    // -----------------------------------------------------------------------

    #[test]
    fn all_prompt_arguments_have_titles() {
        let result = list_prompts();
        let all_args: Vec<_> = result
            .prompts
            .iter()
            .filter_map(|p| p.arguments.as_ref().map(|a| (&p.name, a)))
            .flat_map(|(name, args)| args.iter().map(move |a| (name, a)))
            .collect();
        for (prompt_name, arg) in &all_args {
            assert!(
                arg.title.is_some(),
                "Prompt {prompt_name} arg {} missing title",
                arg.name
            );
        }
    }

    #[test]
    fn all_prompt_arguments_have_descriptions() {
        let result = list_prompts();
        let all_args: Vec<_> = result
            .prompts
            .iter()
            .filter_map(|p| p.arguments.as_ref().map(|a| (&p.name, a)))
            .flat_map(|(name, args)| args.iter().map(move |a| (name, a)))
            .collect();
        for (prompt_name, arg) in &all_args {
            assert!(
                arg.description.is_some(),
                "Prompt {prompt_name} arg {} missing description",
                arg.name
            );
        }
    }

    // -----------------------------------------------------------------------
    // Wiremock-based prompt handler tests
    // -----------------------------------------------------------------------

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

        /// Standard animal-details response for a given LPN ID and gender.
        fn animal_details_response(lpn_id: &str, gender: &str) -> serde_json::Value {
            serde_json::json!({
                "success": true,
                "data": {
                    "gender": gender,
                    "dateOfBirth": "01/15/2020",
                    "progenyCount": 5,
                    "breed": { "breedName": "Katahdin", "breedId": 640 },
                    "searchResultViewModel": {
                        "lpnId": lpn_id,
                        "status": "CURRENT",
                        "bwt": 0.5,
                        "accbwt": 80,
                        "wwt": 2.0,
                        "accwwt": 70,
                        "nlb": 0.15,
                        "accnlb": 60
                    },
                    "contactInfo": {
                        "farmName": "Test Farm"
                    }
                }
            })
        }

        /// Standard lineage response for a given LPN ID.
        fn lineage_response(lpn_id: &str) -> serde_json::Value {
            serde_json::json!({
                "lpnId": lpn_id,
                "content": "<div>Test Farm</div><div>DOB: 1/1/2020</div><div>Sex: Male</div>",
                "children": [
                    {
                        "lpnId": format!("{lpn_id}_SIRE"),
                        "content": "<div>Sire Farm</div>",
                        "children": []
                    },
                    {
                        "lpnId": format!("{lpn_id}_DAM"),
                        "content": "<div>Dam Farm</div>",
                        "children": []
                    }
                ]
            })
        }

        /// Standard search results response.
        fn search_results_response() -> serde_json::Value {
            serde_json::json!({
                "TotalCount": 3,
                "Results": [
                    { "lpnId": "A1", "gender": "Male", "bwt": 0.3, "accbwt": 70, "wwt": 2.5, "accwwt": 65 },
                    { "lpnId": "A2", "gender": "Male", "bwt": -0.1, "accbwt": 80, "wwt": 3.0, "accwwt": 75 },
                    { "lpnId": "A3", "gender": "Male", "bwt": 0.5, "accbwt": 60, "wwt": 1.8, "accwwt": 55 }
                ]
            })
        }

        /// Standard trait ranges response.
        fn trait_ranges_response() -> serde_json::Value {
            serde_json::json!({
                "rangeBWT": "-0.5 - 1.0",
                "rangeWWT": "-3.0 - 6.0",
                "rangeNLB": "-0.1 - 0.3"
            })
        }

        // -- evaluate-ram ---------------------------------------------------

        #[tokio::test]
        async fn evaluate_ram_returns_assessment() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "RAM001"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(animal_details_response("RAM001", "Male")),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("lpn_id".to_string(), "RAM001".to_string());

            let result = get_prompt(&client, "evaluate-ram", &args, None)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 2);
            assert!(result.description.as_deref().unwrap().contains("ram"));
            assert!(result.description.as_deref().unwrap().contains("RAM001"));
        }

        // -- evaluate-ewe ---------------------------------------------------

        #[tokio::test]
        async fn evaluate_ewe_returns_maternal_assessment() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "EWE001"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(animal_details_response("EWE001", "Female")),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("lpn_id".to_string(), "EWE001".to_string());

            let result = get_prompt(&client, "evaluate-ewe", &args, None)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 2);
            assert!(result.description.as_deref().unwrap().contains("ewe"));
            assert!(result.description.as_deref().unwrap().contains("EWE001"));
        }

        // -- compare-breeding-stock -----------------------------------------

        #[tokio::test]
        async fn compare_breeding_stock_two_animals() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "CMP1"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(animal_details_response("CMP1", "Male")),
                )
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "CMP2"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(animal_details_response("CMP2", "Female")),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("animal_ids".to_string(), "CMP1, CMP2".to_string());

            let result = get_prompt(&client, "compare-breeding-stock", &args, None)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 1);
            assert!(result.description.as_deref().unwrap().contains('2'));
        }

        // -- plan-mating ----------------------------------------------------

        #[tokio::test]
        async fn plan_mating_fetches_sire_dam_and_lineages() {
            let server = MockServer::start().await;

            // Mount animal details for sire and dam
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "SIRE01"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(animal_details_response("SIRE01", "Male")),
                )
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "DAM01"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(animal_details_response("DAM01", "Female")),
                )
                .mount(&server)
                .await;

            // Mount lineage for sire and dam
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "SIRE01"))
                .respond_with(ResponseTemplate::new(200).set_body_json(lineage_response("SIRE01")))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .and(query_param("lpnId", "DAM01"))
                .respond_with(ResponseTemplate::new(200).set_body_json(lineage_response("DAM01")))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("sire_id".to_string(), "SIRE01".to_string());
            args.insert("dam_id".to_string(), "DAM01".to_string());

            let result = get_prompt(&client, "plan-mating", &args, None)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 1);
            let desc = result.description.as_deref().unwrap();
            assert!(desc.contains("SIRE01"));
            assert!(desc.contains("DAM01"));
        }

        // -- flock-improvement ----------------------------------------------

        #[tokio::test]
        async fn flock_improvement_breed_only() {
            let server = MockServer::start().await;

            // Mount search results
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(200).set_body_json(search_results_response()))
                .mount(&server)
                .await;

            // Mount trait ranges
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .and(query_param("breedId", "640"))
                .respond_with(ResponseTemplate::new(200).set_body_json(trait_ranges_response()))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("breed_id".to_string(), "640".to_string());

            let result = get_prompt(&client, "flock-improvement", &args, None)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 1);
            let desc = result.description.as_deref().unwrap();
            assert!(desc.contains("breed"));
        }

        #[tokio::test]
        async fn flock_improvement_with_flock_id() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(200).set_body_json(search_results_response()))
                .mount(&server)
                .await;

            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .and(query_param("breedId", "640"))
                .respond_with(ResponseTemplate::new(200).set_body_json(trait_ranges_response()))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("breed_id".to_string(), "640".to_string());
            args.insert("flock_id".to_string(), "FLOCK42".to_string());

            let result = get_prompt(&client, "flock-improvement", &args, None)
                .await
                .unwrap();

            let desc = result.description.as_deref().unwrap();
            assert!(desc.contains("flock FLOCK42"));
        }

        // -- select-replacement ---------------------------------------------

        #[tokio::test]
        async fn select_replacement_returns_candidates() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(200).set_body_json(search_results_response()))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("breed_id".to_string(), "640".to_string());
            args.insert("gender".to_string(), "Male".to_string());
            args.insert("target_trait".to_string(), "WWT".to_string());

            let result = get_prompt(&client, "select-replacement", &args, None)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 1);
            let desc = result.description.as_deref().unwrap();
            assert!(desc.contains("Male"));
            assert!(desc.contains("WWT"));
        }

        // -- interpret-ebvs -------------------------------------------------

        #[tokio::test]
        async fn interpret_ebvs_returns_glossary_and_data() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .and(query_param("searchString", "INTERP01"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(animal_details_response("INTERP01", "Female")),
                )
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("lpn_id".to_string(), "INTERP01".to_string());

            let result = get_prompt(&client, "interpret-ebvs", &args, None)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 1);
            let desc = result.description.as_deref().unwrap();
            assert!(desc.contains("INTERP01"));
        }

        // -- API error propagation ------------------------------------------

        #[tokio::test]
        async fn evaluate_ram_propagates_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("lpn_id".to_string(), "FAIL001".to_string());

            let result = get_prompt(&client, "evaluate-ram", &args, None).await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.message.contains("Failed to fetch animal"));
        }

        #[tokio::test]
        async fn plan_mating_propagates_lineage_error() {
            let server = MockServer::start().await;

            // Details succeed
            Mock::given(method("GET"))
                .and(path("/details/getAnimalDetails"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(animal_details_response("S1", "Male")),
                )
                .mount(&server)
                .await;

            // Lineage fails
            Mock::given(method("GET"))
                .and(path("/details/getLineage"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("sire_id".to_string(), "S1".to_string());
            args.insert("dam_id".to_string(), "D1".to_string());

            let result = get_prompt(&client, "plan-mating", &args, None).await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                err.message.contains("lineage") || err.message.contains("failed"),
                "Error should mention lineage failure, got: {}",
                err.message
            );
        }

        #[tokio::test]
        async fn flock_improvement_propagates_search_error() {
            let server = MockServer::start().await;

            // Search fails
            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
                .mount(&server)
                .await;

            // Trait ranges succeed
            Mock::given(method("GET"))
                .and(path("/search/getTraitRangesByBreed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(trait_ranges_response()))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("breed_id".to_string(), "640".to_string());

            let result = get_prompt(&client, "flock-improvement", &args, None).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn select_replacement_propagates_search_error() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/search/getPageOfSearchResults"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
                .mount(&server)
                .await;

            let client = mock_client(&server.uri());
            let mut args = std::collections::HashMap::new();
            args.insert("breed_id".to_string(), "640".to_string());
            args.insert("gender".to_string(), "Female".to_string());
            args.insert("target_trait".to_string(), "BWT".to_string());

            let result = get_prompt(&client, "select-replacement", &args, None).await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.message.contains("Search failed"));
        }
    }
}
