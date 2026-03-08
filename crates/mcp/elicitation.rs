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
#[allow(dead_code)]
pub(crate) struct ComparePreferences {
    /// Comma-separated trait abbreviations to focus on (e.g. "WWT,YWT,NLB").
    pub traits: Option<String>,
}

rmcp::elicit_safe!(ComparePreferences);

/// Elicitation schema for plan-mating prompt.
///
/// Gathers breeding constraints for mating planning.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub(crate) struct SelectionCriteria {
    /// Minimum accuracy percentage to require (0-100).
    pub min_accuracy: Option<i32>,
    /// Comma-separated priority traits (e.g. "WWT,NLB").
    pub priority_traits: Option<String>,
}

rmcp::elicit_safe!(SelectionCriteria);

/// Attempt to elicit structured data from the user.
///
/// Returns `Some(data)` if the user accepted, `None` if elicitation is
/// unavailable, the user declined, or the user cancelled. Errors from
/// the service layer are logged and treated as unavailable.
#[allow(dead_code)]
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
}
