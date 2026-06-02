//! Elicitation schemas for structured user input in breeding prompts.
//!
//! Defines flat JSON Schema types for MCP form-mode elicitation,
//! allowing clients to present structured input forms for breeding decisions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Elicitation schema for compare-breeding-stock prompt.
///
/// Allows the user to specify which traits matter for comparison.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ComparePreferences {
    /// Comma-separated trait abbreviations to focus on (e.g. "WWT,YWT,NLB").
    pub traits: Option<String>,
}

rmcp::elicit_safe!(ComparePreferences);

/// Elicitation schema for plan-mating prompt.
///
/// Gathers breeding constraints for mating planning.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(crate) struct MatingConstraints {
    /// Maximum acceptable coefficient of inbreeding (0.0-1.0).
    pub max_coi: Option<f64>,
    /// Breeding objective: "Growth", "Maternal", or "Dual".
    pub breeding_objective: Option<String>,
}

rmcp::elicit_safe!(MatingConstraints);

/// Elicitation schema for flock-improvement prompt.
///
/// Gathers flock context for improvement analysis.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(crate) struct FlockContext {
    /// Breeding objective: "Growth", "Maternal", or "Dual".
    pub breeding_objective: Option<String>,
    /// Approximate flock size (number of breeding ewes).
    pub flock_size: Option<i32>,
}

rmcp::elicit_safe!(FlockContext);

/// Elicitation schema for select-replacement prompt.
///
/// Gathers selection criteria for replacement candidates.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(crate) struct SelectionCriteria {
    /// Minimum accuracy percentage to require (0-100).
    pub min_accuracy: Option<i32>,
    /// Comma-separated priority traits (e.g. "WWT,NLB").
    pub priority_traits: Option<String>,
}

rmcp::elicit_safe!(SelectionCriteria);

/// Attempt to elicit structured data from an optional prompt context.
///
/// Convenience wrapper around [`try_elicit`] for use in prompt handlers where
/// `context` is `Option<&RequestContext>`. Returns `None` when the context is
/// absent, or forwards the result of `try_elicit` otherwise.
pub(crate) async fn try_elicit_opt<T>(
    context: super::prompts::PromptContext<'_>,
    message: &str,
) -> Option<T>
where
    T: rmcp::service::ElicitationSafe + for<'de> serde::Deserialize<'de>,
{
    match context {
        Some(ctx) => try_elicit(ctx, message).await,
        None => None,
    }
}

/// Attempt to elicit structured data from the user.
///
/// Returns `Some(data)` if the user accepted, `None` if elicitation is
/// unavailable, the user declined, or the user cancelled. Errors from
/// the service layer are logged and treated as unavailable.
pub(crate) async fn try_elicit<T>(
    context: &rmcp::service::RequestContext<rmcp::service::RoleServer>,
    message: &str,
) -> Option<T>
where
    T: rmcp::service::ElicitationSafe + for<'de> serde::Deserialize<'de>,
{
    match context.peer.elicit::<T>(message).await {
        Ok(data) => data,
        Err(
            rmcp::service::ElicitationError::CapabilityNotSupported
            | rmcp::service::ElicitationError::UserDeclined
            | rmcp::service::ElicitationError::UserCancelled
            | rmcp::service::ElicitationError::NoContent,
        ) => None,
        Err(e) => {
            tracing::warn!("Elicitation failed: {e}");
            None
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn compare_preferences_schema_is_object() {
        let schema = schemars::schema_for!(ComparePreferences);
        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["type"], "object");
    }

    #[test]
    fn mating_constraints_schema_is_object() {
        let schema = schemars::schema_for!(MatingConstraints);
        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["type"], "object");
    }

    #[test]
    fn flock_context_schema_is_object() {
        let schema = schemars::schema_for!(FlockContext);
        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["type"], "object");
    }

    #[test]
    fn selection_criteria_schema_is_object() {
        let schema = schemars::schema_for!(SelectionCriteria);
        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["type"], "object");
    }

    #[test]
    fn all_fields_are_optional() {
        let cp: ComparePreferences = serde_json::from_str("{}").unwrap();
        assert!(cp.traits.is_none());

        let mc: MatingConstraints = serde_json::from_str("{}").unwrap();
        assert!(mc.max_coi.is_none());
        assert!(mc.breeding_objective.is_none());

        let fc: FlockContext = serde_json::from_str("{}").unwrap();
        assert!(fc.breeding_objective.is_none());
        assert!(fc.flock_size.is_none());

        let sc: SelectionCriteria = serde_json::from_str("{}").unwrap();
        assert!(sc.min_accuracy.is_none());
        assert!(sc.priority_traits.is_none());
    }

    #[test]
    fn round_trip_serialization() {
        let mc = MatingConstraints {
            max_coi: Some(0.0625),
            breeding_objective: Some("Dual".to_string()),
        };
        let json = serde_json::to_string(&mc).unwrap();
        let mc2: MatingConstraints = serde_json::from_str(&json).unwrap();
        assert_eq!(mc2.max_coi, Some(0.0625));
        assert_eq!(mc2.breeding_objective.as_deref(), Some("Dual"));
    }

    #[test]
    fn compare_preferences_with_traits() {
        let cp = ComparePreferences {
            traits: Some("WWT,YWT".to_string()),
        };
        let json = serde_json::to_value(&cp).unwrap();
        assert_eq!(json["traits"], "WWT,YWT");
    }

    #[test]
    fn mating_constraints_with_all_fields() {
        let mc = MatingConstraints {
            max_coi: Some(0.125),
            breeding_objective: Some("Growth".to_string()),
        };
        let json = serde_json::to_value(&mc).unwrap();
        assert_eq!(json["max_coi"], 0.125);
        assert_eq!(json["breeding_objective"], "Growth");
    }

    #[test]
    fn flock_context_with_all_fields() {
        let fc = FlockContext {
            breeding_objective: Some("Maternal".to_string()),
            flock_size: Some(200),
        };
        let json = serde_json::to_value(&fc).unwrap();
        assert_eq!(json["breeding_objective"], "Maternal");
        assert_eq!(json["flock_size"], 200);
    }

    #[test]
    fn selection_criteria_with_all_fields() {
        let sc = SelectionCriteria {
            min_accuracy: Some(60),
            priority_traits: Some("BWT,NLB".to_string()),
        };
        let json = serde_json::to_value(&sc).unwrap();
        assert_eq!(json["min_accuracy"], 60);
        assert_eq!(json["priority_traits"], "BWT,NLB");
    }

    #[test]
    fn compare_preferences_debug_format() {
        let cp = ComparePreferences {
            traits: Some("WWT".into()),
        };
        let debug = format!("{cp:?}");
        assert!(debug.contains("WWT"));
    }

    #[test]
    fn mating_constraints_debug_format() {
        let mc = MatingConstraints {
            max_coi: Some(0.0625),
            breeding_objective: None,
        };
        let debug = format!("{mc:?}");
        assert!(debug.contains("0.0625"));
    }

    #[test]
    fn flock_context_debug_format() {
        let fc = FlockContext {
            breeding_objective: None,
            flock_size: Some(100),
        };
        let debug = format!("{fc:?}");
        assert!(debug.contains("100"));
    }

    #[test]
    fn selection_criteria_debug_format() {
        let sc = SelectionCriteria {
            min_accuracy: Some(50),
            priority_traits: None,
        };
        let debug = format!("{sc:?}");
        assert!(debug.contains("50"));
    }

    #[test]
    fn mating_constraints_schema_has_max_coi_field() {
        let schema = schemars::schema_for!(MatingConstraints);
        let json = serde_json::to_value(&schema).unwrap();
        let props = json["properties"].as_object().unwrap();
        assert!(props.contains_key("max_coi"));
        assert!(props.contains_key("breeding_objective"));
    }

    #[test]
    fn selection_criteria_schema_has_fields() {
        let schema = schemars::schema_for!(SelectionCriteria);
        let json = serde_json::to_value(&schema).unwrap();
        let props = json["properties"].as_object().unwrap();
        assert!(props.contains_key("min_accuracy"));
        assert!(props.contains_key("priority_traits"));
    }

    #[test]
    fn flock_context_schema_has_fields() {
        let schema = schemars::schema_for!(FlockContext);
        let json = serde_json::to_value(&schema).unwrap();
        let props = json["properties"].as_object().unwrap();
        assert!(props.contains_key("breeding_objective"));
        assert!(props.contains_key("flock_size"));
    }

    #[test]
    fn compare_preferences_schema_has_traits_field() {
        let schema = schemars::schema_for!(ComparePreferences);
        let json = serde_json::to_value(&schema).unwrap();
        let props = json["properties"].as_object().unwrap();
        assert!(props.contains_key("traits"));
    }

    #[test]
    fn compare_preferences_deserialize_with_traits() {
        let json = r#"{"traits":"PEMD,PFAT"}"#;
        let cp: ComparePreferences = serde_json::from_str(json).unwrap();
        assert_eq!(cp.traits.as_deref(), Some("PEMD,PFAT"));
    }

    #[test]
    fn flock_context_round_trip() {
        let fc = FlockContext {
            breeding_objective: Some("Dual".to_string()),
            flock_size: Some(350),
        };
        let json = serde_json::to_string(&fc).unwrap();
        let fc2: FlockContext = serde_json::from_str(&json).unwrap();
        assert_eq!(fc2.breeding_objective.as_deref(), Some("Dual"));
        assert_eq!(fc2.flock_size, Some(350));
    }

    #[test]
    fn selection_criteria_round_trip() {
        let sc = SelectionCriteria {
            min_accuracy: Some(70),
            priority_traits: Some("WWT,PEMD".to_string()),
        };
        let json = serde_json::to_string(&sc).unwrap();
        let sc2: SelectionCriteria = serde_json::from_str(&json).unwrap();
        assert_eq!(sc2.min_accuracy, Some(70));
        assert_eq!(sc2.priority_traits.as_deref(), Some("WWT,PEMD"));
    }

    #[tokio::test]
    async fn try_elicit_opt_returns_none_when_context_is_none() {
        let result: Option<ComparePreferences> = try_elicit_opt(None, "test message").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn try_elicit_opt_returns_none_for_mating_constraints_without_context() {
        let result: Option<MatingConstraints> = try_elicit_opt(None, "provide constraints").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn try_elicit_opt_returns_none_for_flock_context_without_context() {
        let result: Option<FlockContext> = try_elicit_opt(None, "provide flock info").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn try_elicit_opt_returns_none_for_selection_criteria_without_context() {
        let result: Option<SelectionCriteria> =
            try_elicit_opt(None, "provide selection criteria").await;
        assert!(result.is_none());
    }
}
