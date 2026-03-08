//! Tool set filtering for the MCP server.
//!
//! Tools are organized into four configurable sets that can be independently
//! enabled or disabled via the `--tools` CLI flag.

use std::{collections::HashSet, fmt};

/// Tool set identifiers for the NSIP MCP server.
///
/// Each set groups related tools by functional area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolSet {
    /// Search & retrieval tools: `search`, `details`, `lineage`, `progeny`, `profile`.
    Search,
    /// Analytics tools: `compare`, `rank`, `inbreeding_check`, `mating_recommendations`.
    Analytics,
    /// Flock tools: `flock_summary`, `database_status`.
    Flock,
    /// Breed tools: `breed_groups`, `trait_ranges`.
    Breed,
}

impl fmt::Display for ToolSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Search => write!(f, "search"),
            Self::Analytics => write!(f, "analytics"),
            Self::Flock => write!(f, "flock"),
            Self::Breed => write!(f, "breed"),
        }
    }
}

/// All known tool set variants.
const ALL_SETS: [ToolSet; 4] = [
    ToolSet::Search,
    ToolSet::Analytics,
    ToolSet::Flock,
    ToolSet::Breed,
];

/// Returns the tool set that a given tool name belongs to.
///
/// Each NSIP tool belongs to exactly one set. Unknown tool names return `None`.
#[must_use]
pub fn tool_set_for(tool_name: &str) -> Option<ToolSet> {
    match tool_name {
        "search" | "details" | "lineage" | "progeny" | "profile" => Some(ToolSet::Search),
        "compare" | "rank" | "inbreeding_check" | "mating_recommendations" => {
            Some(ToolSet::Analytics)
        },
        "flock_summary" | "database_status" => Some(ToolSet::Flock),
        "breed_groups" | "trait_ranges" => Some(ToolSet::Breed),
        _ => None,
    }
}

/// Pre-computed set of enabled tool sets for fast lookup.
#[derive(Debug, Clone)]
pub struct EnabledToolSets {
    /// Which sets are enabled.
    enabled: HashSet<ToolSet>,
}

impl Default for EnabledToolSets {
    fn default() -> Self {
        Self::all()
    }
}

impl EnabledToolSets {
    /// All 4 tool sets enabled (the default).
    #[must_use]
    pub fn all() -> Self {
        Self {
            enabled: ALL_SETS.into_iter().collect(),
        }
    }

    /// Parse a comma-separated list of set names.
    ///
    /// Unknown tokens are silently ignored. An empty string enables nothing.
    #[must_use]
    pub fn from_csv(csv: &str) -> Self {
        let mut enabled = HashSet::new();
        for token in csv.split(',') {
            match token.trim() {
                "search" => {
                    enabled.insert(ToolSet::Search);
                },
                "analytics" => {
                    enabled.insert(ToolSet::Analytics);
                },
                "flock" => {
                    enabled.insert(ToolSet::Flock);
                },
                "breed" => {
                    enabled.insert(ToolSet::Breed);
                },
                _ => {},
            }
        }
        Self { enabled }
    }

    /// Check if a given tool set is enabled.
    #[must_use]
    pub fn is_set_enabled(&self, set: ToolSet) -> bool {
        self.enabled.contains(&set)
    }

    /// Check if a tool is enabled by name.
    ///
    /// A tool is enabled if its set is enabled. Unknown tools are always enabled.
    #[must_use]
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        tool_set_for(tool_name).is_none_or(|set| self.enabled.contains(&set))
    }

    /// Returns the names of all disabled tools.
    ///
    /// Used to remove routes from the `ToolRouter` at construction time.
    pub fn disabled_tool_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        ALL_TOOL_NAMES
            .iter()
            .copied()
            .filter(|name| !self.is_tool_enabled(name))
    }
}

/// All 13 registered tool names.
const ALL_TOOL_NAMES: &[&str] = &[
    "search",
    "details",
    "lineage",
    "progeny",
    "profile",
    "compare",
    "rank",
    "inbreeding_check",
    "mating_recommendations",
    "flock_summary",
    "database_status",
    "breed_groups",
    "trait_ranges",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_enables_every_set() {
        let sets = EnabledToolSets::all();
        assert!(sets.is_set_enabled(ToolSet::Search));
        assert!(sets.is_set_enabled(ToolSet::Analytics));
        assert!(sets.is_set_enabled(ToolSet::Flock));
        assert!(sets.is_set_enabled(ToolSet::Breed));
    }

    #[test]
    fn default_enables_all() {
        let sets = EnabledToolSets::default();
        for name in ALL_TOOL_NAMES {
            assert!(sets.is_tool_enabled(name), "expected {name} enabled");
        }
    }

    #[test]
    fn from_csv_selective() {
        let sets = EnabledToolSets::from_csv("search,breed");
        assert!(sets.is_tool_enabled("search"));
        assert!(sets.is_tool_enabled("details"));
        assert!(sets.is_tool_enabled("breed_groups"));
        assert!(sets.is_tool_enabled("trait_ranges"));
        assert!(!sets.is_tool_enabled("compare"));
        assert!(!sets.is_tool_enabled("flock_summary"));
    }

    #[test]
    fn from_csv_whitespace_trimming() {
        let sets = EnabledToolSets::from_csv(" search , analytics ");
        assert!(sets.is_tool_enabled("search"));
        assert!(sets.is_tool_enabled("compare"));
        assert!(!sets.is_tool_enabled("breed_groups"));
    }

    #[test]
    fn from_csv_empty_enables_nothing() {
        let sets = EnabledToolSets::from_csv("");
        assert!(!sets.is_tool_enabled("search"));
        assert!(!sets.is_tool_enabled("compare"));
        assert!(!sets.is_tool_enabled("flock_summary"));
        assert!(!sets.is_tool_enabled("breed_groups"));
    }

    #[test]
    fn from_csv_unknown_tokens_ignored() {
        let sets = EnabledToolSets::from_csv("search,foobar,breed");
        assert!(sets.is_tool_enabled("search"));
        assert!(sets.is_tool_enabled("breed_groups"));
        assert!(!sets.is_tool_enabled("compare"));
    }

    #[test]
    fn unknown_tools_always_enabled() {
        let sets = EnabledToolSets::from_csv("");
        assert!(sets.is_tool_enabled("some_future_tool"));
    }

    #[test]
    fn disabled_tool_names_correct() {
        let sets = EnabledToolSets::from_csv("search");
        let disabled: Vec<&str> = sets.disabled_tool_names().collect();
        assert!(disabled.contains(&"compare"));
        assert!(disabled.contains(&"rank"));
        assert!(disabled.contains(&"flock_summary"));
        assert!(disabled.contains(&"breed_groups"));
        assert!(!disabled.contains(&"search"));
        assert!(!disabled.contains(&"details"));
    }

    #[test]
    fn all_tool_names_have_set_mapping() {
        for name in ALL_TOOL_NAMES {
            assert!(
                tool_set_for(name).is_some(),
                "tool '{name}' has no set mapping"
            );
        }
    }

    #[test]
    fn tool_set_display() {
        assert_eq!(ToolSet::Search.to_string(), "search");
        assert_eq!(ToolSet::Analytics.to_string(), "analytics");
        assert_eq!(ToolSet::Flock.to_string(), "flock");
        assert_eq!(ToolSet::Breed.to_string(), "breed");
    }

    #[test]
    fn search_set_has_five_tools() {
        let count = ALL_TOOL_NAMES
            .iter()
            .filter(|n| tool_set_for(n) == Some(ToolSet::Search))
            .count();
        assert_eq!(count, 5);
    }

    #[test]
    fn analytics_set_has_four_tools() {
        let count = ALL_TOOL_NAMES
            .iter()
            .filter(|n| tool_set_for(n) == Some(ToolSet::Analytics))
            .count();
        assert_eq!(count, 4);
    }

    #[test]
    fn flock_set_has_two_tools() {
        let count = ALL_TOOL_NAMES
            .iter()
            .filter(|n| tool_set_for(n) == Some(ToolSet::Flock))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn breed_set_has_two_tools() {
        let count = ALL_TOOL_NAMES
            .iter()
            .filter(|n| tool_set_for(n) == Some(ToolSet::Breed))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn tool_set_clone_and_eq() {
        let a = ToolSet::Search;
        #[allow(clippy::clone_on_copy)]
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn enabled_tool_sets_clone() {
        let sets = EnabledToolSets::from_csv("search,breed");
        #[allow(clippy::redundant_clone)]
        let cloned = sets.clone();
        assert!(cloned.is_tool_enabled("search"));
        assert!(cloned.is_tool_enabled("breed_groups"));
        assert!(!cloned.is_tool_enabled("compare"));
    }
}
