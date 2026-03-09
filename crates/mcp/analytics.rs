//! Livestock breeding analytics — COI, ranking, and trait complementarity.
//!
//! Pure computation functions with no MCP dependencies. All functions are
//! designed to work with the data types from [`crate::models`].

use std::collections::HashMap;

use serde::Serialize;

use crate::{AnimalDetails, Lineage};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Traffic-light rating for inbreeding coefficient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CoiRating {
    /// COI < 6.25% — acceptable inbreeding level.
    Green,
    /// 6.25% <= COI < 12.5% — elevated inbreeding, proceed with caution.
    Yellow,
    /// COI >= 12.5% — high inbreeding, generally avoid this mating.
    Red,
}

/// A common ancestor found in both sire and dam pedigrees.
#[derive(Debug, Clone, Serialize)]
pub struct SharedAncestor {
    /// LPN ID of the common ancestor.
    pub lpn_id: String,
    /// Generations from sire to this ancestor.
    pub sire_depth: usize,
    /// Generations from dam to this ancestor.
    pub dam_depth: usize,
}

/// Result of a coefficient of inbreeding calculation.
#[derive(Debug, Clone, Serialize)]
pub struct CoiResult {
    /// Wright's coefficient of inbreeding (0.0 – 1.0).
    pub coefficient: f64,
    /// Traffic-light rating based on threshold.
    pub rating: CoiRating,
    /// Common ancestors contributing to inbreeding.
    pub shared_ancestors: Vec<SharedAncestor>,
}

/// A ranked animal with composite score.
#[derive(Debug, Clone, Serialize)]
pub struct RankedAnimal {
    /// LPN identifier.
    pub lpn_id: String,
    /// Weighted composite score.
    pub score: f64,
    /// Per-trait weighted scores.
    pub trait_scores: HashMap<String, f64>,
}

// ---------------------------------------------------------------------------
// COI calculation
// ---------------------------------------------------------------------------

/// Threshold for "Green" COI rating.
const COI_GREEN_THRESHOLD: f64 = 0.0625;
/// Threshold for "Yellow" COI rating (above this is "Red").
const COI_YELLOW_THRESHOLD: f64 = 0.125;

/// Classify a COI value into a traffic-light rating.
fn coi_rating(coefficient: f64) -> CoiRating {
    if coefficient < COI_GREEN_THRESHOLD {
        CoiRating::Green
    } else if coefficient < COI_YELLOW_THRESHOLD {
        CoiRating::Yellow
    } else {
        CoiRating::Red
    }
}

/// Collect all ancestors from a lineage tree into a map of `lpn_id -> Vec<depth>`.
///
/// Depth 0 = the animal's parents (sire/dam), depth 1 = grandparents, etc.
fn collect_ancestor_depths(lineage: &Lineage) -> HashMap<String, Vec<usize>> {
    let mut depths: HashMap<String, Vec<usize>> = HashMap::new();

    // Sire and dam are at depth 0 (parents)
    if let Some(ref sire) = lineage.sire {
        depths.entry(sire.lpn_id.clone()).or_default().push(0);
    }
    if let Some(ref dam) = lineage.dam {
        depths.entry(dam.lpn_id.clone()).or_default().push(0);
    }

    // Generations: index 0 = grandparents (depth 1), index 1 = great-grandparents (depth 2), etc.
    for (gp_idx, generation) in lineage.generations.iter().enumerate() {
        let depth = gp_idx + 1;
        for animal in generation {
            depths.entry(animal.lpn_id.clone()).or_default().push(depth);
        }
    }

    depths
}

/// Build the cartesian product of sire/dam depth paths for a single ancestor.
fn ancestor_path_combinations(
    lpn_id: &str,
    sire_depths: &[usize],
    dam_depths: &[usize],
) -> Vec<SharedAncestor> {
    sire_depths
        .iter()
        .flat_map(|&sd| {
            dam_depths.iter().map(move |&dd| SharedAncestor {
                lpn_id: lpn_id.to_string(),
                sire_depth: sd,
                dam_depth: dd,
            })
        })
        .collect()
}

/// Find ancestors that appear in both the sire's and dam's pedigrees.
///
/// Returns pairs of `(lpn_id, sire_depth, dam_depth)` for each shared ancestor.
#[must_use]
pub fn find_shared_ancestors(sire_lineage: &Lineage, dam_lineage: &Lineage) -> Vec<SharedAncestor> {
    let sire_ancestors = collect_ancestor_depths(sire_lineage);
    let dam_ancestors = collect_ancestor_depths(dam_lineage);

    let mut shared = Vec::new();

    for (lpn_id, sire_depths) in &sire_ancestors {
        if let Some(dam_depths) = dam_ancestors.get(lpn_id) {
            shared.extend(ancestor_path_combinations(lpn_id, sire_depths, dam_depths));
        }
    }

    shared
}

/// Calculate Wright's coefficient of inbreeding from sire and dam lineage trees.
///
/// Formula: `COI = Σ [(0.5)^(n₁ + n₂ + 1)]`
/// where n₁ = path length from sire to common ancestor,
///       n₂ = path length from dam to common ancestor.
///
/// # Arguments
///
/// * `sire_lineage` — Pedigree tree of the sire.
/// * `dam_lineage` — Pedigree tree of the dam.
///
/// # Returns
///
/// A [`CoiResult`] with the coefficient, rating, and list of shared ancestors.
#[must_use]
pub fn calculate_coi(sire_lineage: &Lineage, dam_lineage: &Lineage) -> CoiResult {
    let shared = find_shared_ancestors(sire_lineage, dam_lineage);

    let coefficient: f64 = shared
        .iter()
        .map(|a| 0.5_f64.powi(i32::try_from(a.sire_depth + a.dam_depth + 1).unwrap_or(i32::MAX)))
        .sum();

    let rating = coi_rating(coefficient);

    CoiResult {
        coefficient,
        rating,
        shared_ancestors: shared,
    }
}

// ---------------------------------------------------------------------------
// Ranking
// ---------------------------------------------------------------------------

/// Rank animals by a weighted composite of their EBV traits.
///
/// Score = `Σ (trait_value × weight × accuracy/100)` for each trait where both
/// a weight and a value exist. Animals are sorted descending by score.
///
/// # Arguments
///
/// * `animals` — Slice of animal details to rank.
/// * `weights` — Trait name to weight mapping (e.g. `{"BWT": -1.0, "WWT": 2.0}`).
///
/// # Returns
///
/// A sorted `Vec<RankedAnimal>` (highest score first).
#[must_use]
pub fn rank_animals<S: ::std::hash::BuildHasher>(
    animals: &[AnimalDetails],
    weights: &HashMap<String, f64, S>,
) -> Vec<RankedAnimal> {
    let mut ranked: Vec<RankedAnimal> = animals
        .iter()
        .map(|animal| {
            let mut trait_scores = HashMap::new();
            let mut total = 0.0;

            for (trait_name, weight) in weights {
                if let Some(t) = animal.traits.get(trait_name) {
                    let accuracy_factor =
                        f64::from(t.accuracy.unwrap_or(50)).clamp(0.0, 100.0) / 100.0;
                    let score = t.value * weight * accuracy_factor;
                    trait_scores.insert(trait_name.clone(), score);
                    total += score;
                }
            }

            RankedAnimal {
                lpn_id: animal.lpn_id.clone(),
                score: total,
                trait_scores,
            }
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked
}

// ---------------------------------------------------------------------------
// Trait complementarity
// ---------------------------------------------------------------------------

/// Predict midparent EBV values for potential offspring.
///
/// For each trait present in **both** sire and dam, the midparent value is
/// `(sire_value + dam_value) / 2.0`.
///
/// # Arguments
///
/// * `sire` — Details of the sire.
/// * `dam` — Details of the dam.
///
/// # Returns
///
/// A map of trait name to predicted midparent value.
#[must_use]
pub fn trait_complementarity(sire: &AnimalDetails, dam: &AnimalDetails) -> HashMap<String, f64> {
    let mut predictions = HashMap::new();

    for (name, sire_trait) in &sire.traits {
        if let Some(dam_trait) = dam.traits.get(name) {
            predictions.insert(
                name.clone(),
                f64::midpoint(sire_trait.value, dam_trait.value),
            );
        }
    }

    predictions
}

// ---------------------------------------------------------------------------
// EBV Glossary (static data for resources)
// ---------------------------------------------------------------------------

/// EBV trait definitions with units, interpretation, and selection direction.
pub struct TraitDefinition {
    /// Trait abbreviation (e.g. "BWT").
    pub abbreviation: &'static str,
    /// Full name.
    pub name: &'static str,
    /// Unit of measurement.
    pub unit: &'static str,
    /// What the trait measures.
    pub description: &'static str,
    /// Whether higher or lower is better for most breeding goals.
    pub selection_direction: &'static str,
}

/// Complete glossary of NSIP EBV traits.
#[must_use]
pub fn ebv_glossary() -> Vec<TraitDefinition> {
    vec![
        TraitDefinition {
            abbreviation: "BWT",
            name: "Birth Weight",
            unit: "lbs",
            description: "Predicted difference in birth weight of lambs. Lower values reduce dystocia risk.",
            selection_direction: "Lower is generally preferred",
        },
        TraitDefinition {
            abbreviation: "WWT",
            name: "Weaning Weight",
            unit: "lbs",
            description: "Predicted difference in weight at weaning (60 days). Higher values indicate faster early growth.",
            selection_direction: "Higher is preferred",
        },
        TraitDefinition {
            abbreviation: "PWWT",
            name: "Post-Weaning Weight",
            unit: "lbs",
            description: "Predicted difference in post-weaning weight. Reflects growth potential after weaning.",
            selection_direction: "Higher is preferred",
        },
        TraitDefinition {
            abbreviation: "YWT",
            name: "Yearling Weight",
            unit: "lbs",
            description: "Predicted difference in yearling weight (365 days). Important for market lamb production.",
            selection_direction: "Higher is preferred",
        },
        TraitDefinition {
            abbreviation: "FAT",
            name: "Fat Depth",
            unit: "mm",
            description: "Predicted difference in subcutaneous fat depth at the 12th-13th rib. Affects carcass quality.",
            selection_direction: "Moderate preferred (breed-dependent)",
        },
        TraitDefinition {
            abbreviation: "EMD",
            name: "Eye Muscle Depth",
            unit: "mm",
            description: "Predicted difference in loin eye muscle depth. Higher values indicate more muscling.",
            selection_direction: "Higher is preferred",
        },
        TraitDefinition {
            abbreviation: "NLB",
            name: "Number of Lambs Born",
            unit: "lambs",
            description: "Predicted difference in number of lambs born per lambing. Key maternal trait for prolificacy.",
            selection_direction: "Higher is preferred (with caution)",
        },
        TraitDefinition {
            abbreviation: "NWT",
            name: "Number of Lambs Weaned",
            unit: "lambs",
            description: "Predicted difference in number of lambs weaned per lambing. Reflects maternal ability and lamb survival.",
            selection_direction: "Higher is preferred",
        },
        TraitDefinition {
            abbreviation: "PWT",
            name: "Pounds Weaned",
            unit: "lbs",
            description: "Total weight of lambs weaned per ewe lambing. Combines prolificacy and growth.",
            selection_direction: "Higher is preferred",
        },
        TraitDefinition {
            abbreviation: "DAG",
            name: "Dag Score",
            unit: "score",
            description: "Predicted difference in dags (fecal soiling of the breech). Lower values mean cleaner sheep.",
            selection_direction: "Lower is preferred",
        },
        TraitDefinition {
            abbreviation: "WGR",
            name: "Wool Growth Rate",
            unit: "g/day",
            description: "Predicted difference in daily wool growth. Important for wool breeds.",
            selection_direction: "Higher is preferred (wool breeds)",
        },
        TraitDefinition {
            abbreviation: "WEC",
            name: "Worm Egg Count",
            unit: "eggs/g",
            description: "Predicted difference in fecal worm egg count. Lower values indicate parasite resistance.",
            selection_direction: "Lower is preferred",
        },
        TraitDefinition {
            abbreviation: "FEC",
            name: "Fecal Egg Count",
            unit: "eggs/g",
            description: "Predicted difference in fecal egg count (alternate measure). Lower values indicate parasite resistance.",
            selection_direction: "Lower is preferred",
        },
    ]
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineageAnimal, Trait};

    fn make_lineage_animal(lpn_id: &str) -> LineageAnimal {
        LineageAnimal {
            lpn_id: lpn_id.to_string(),
            farm_name: None,
            us_index: None,
            src_index: None,
            date_of_birth: None,
            sex: None,
            status: None,
        }
    }

    fn make_lineage(
        subject_id: &str,
        sire_id: Option<&str>,
        dam_id: Option<&str>,
        generations: Vec<Vec<&str>>,
    ) -> Lineage {
        Lineage {
            subject: Some(make_lineage_animal(subject_id)),
            sire: sire_id.map(make_lineage_animal),
            dam: dam_id.map(make_lineage_animal),
            generations: generations
                .into_iter()
                .map(|gp| gp.into_iter().map(make_lineage_animal).collect())
                .collect(),
        }
    }

    fn make_animal(lpn_id: &str, trait_data: &[(&str, f64, i32)]) -> AnimalDetails {
        let mut traits = HashMap::new();
        for &(name, value, accuracy) in trait_data {
            traits.insert(
                name.to_string(),
                Trait {
                    name: name.to_string(),
                    value,
                    accuracy: Some(accuracy),
                    units: None,
                },
            );
        }

        AnimalDetails {
            lpn_id: lpn_id.to_string(),
            breed: None,
            breed_group: None,
            date_of_birth: None,
            gender: None,
            status: None,
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
    fn coi_no_shared_ancestors() {
        // Sire and dam share no ancestors
        let sire_lineage = make_lineage("SIRE", Some("GS1"), Some("GD1"), vec![]);
        let dam_lineage = make_lineage("DAM", Some("GS2"), Some("GD2"), vec![]);

        let result = calculate_coi(&sire_lineage, &dam_lineage);
        assert!((result.coefficient - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.rating, CoiRating::Green);
        assert!(result.shared_ancestors.is_empty());
    }

    #[test]
    fn coi_half_siblings() {
        // Half siblings: share one grandparent (common sire's sire)
        // Sire's lineage: sire has parents GS_COMMON, GD1
        let sire_lineage = make_lineage(
            "SIRE",
            Some("SS"),
            Some("SD"),
            vec![vec!["GS_COMMON", "GD_A", "GS_B", "GD_B"]],
        );
        // Dam's lineage: dam has parents GS_COMMON (same!), GD2
        let dam_lineage = make_lineage(
            "DAM",
            Some("DS"),
            Some("DD"),
            vec![vec!["GS_COMMON", "GD_C", "GS_D", "GD_D"]],
        );

        let result = calculate_coi(&sire_lineage, &dam_lineage);
        // GS_COMMON appears at depth 1 in both → 0.5^(1+1+1) = 0.125
        assert!((result.coefficient - 0.125).abs() < f64::EPSILON);
        assert_eq!(result.rating, CoiRating::Red);
        assert_eq!(result.shared_ancestors.len(), 1);
        assert_eq!(result.shared_ancestors[0].lpn_id, "GS_COMMON");
    }

    #[test]
    fn coi_rating_thresholds() {
        assert_eq!(coi_rating(0.0), CoiRating::Green);
        assert_eq!(coi_rating(0.05), CoiRating::Green);
        assert_eq!(coi_rating(0.0625), CoiRating::Yellow);
        assert_eq!(coi_rating(0.10), CoiRating::Yellow);
        assert_eq!(coi_rating(0.125), CoiRating::Red);
        assert_eq!(coi_rating(0.25), CoiRating::Red);
    }

    #[test]
    fn rank_animals_basic() {
        let animals = vec![
            make_animal("A1", &[("BWT", -0.5, 80), ("WWT", 10.0, 90)]),
            make_animal("A2", &[("BWT", 0.2, 70), ("WWT", 15.0, 85)]),
            make_animal("A3", &[("BWT", -1.0, 60), ("WWT", 8.0, 75)]),
        ];

        let mut weights = HashMap::new();
        weights.insert("BWT".to_string(), -1.0); // Negative weight: lower BWT is better
        weights.insert("WWT".to_string(), 2.0); // Positive weight: higher WWT is better

        let ranked = rank_animals(&animals, &weights);
        assert_eq!(ranked.len(), 3);
        // A2: BWT=0.2*-1*0.7=-0.14, WWT=15*2*0.85=25.5, total=25.36
        // A1: BWT=-0.5*-1*0.8=0.4, WWT=10*2*0.9=18.0, total=18.4
        // A3: BWT=-1.0*-1*0.6=0.6, WWT=8*2*0.75=12.0, total=12.6
        assert_eq!(ranked[0].lpn_id, "A2");
        assert_eq!(ranked[1].lpn_id, "A1");
        assert_eq!(ranked[2].lpn_id, "A3");
    }

    #[test]
    fn rank_animals_empty_weights() {
        let animals = vec![make_animal("A1", &[("BWT", 1.0, 80)])];
        let weights = HashMap::new();
        let ranked = rank_animals(&animals, &weights);
        assert_eq!(ranked.len(), 1);
        assert!((ranked[0].score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn trait_complementarity_basic() {
        let sire = make_animal("SIRE", &[("BWT", 0.5, 80), ("WWT", 12.0, 90)]);
        let dam = make_animal(
            "DAM",
            &[("BWT", -0.3, 75), ("WWT", 10.0, 85), ("NLB", 0.2, 70)],
        );

        let comp = trait_complementarity(&sire, &dam);
        // Only traits present in both should appear
        assert_eq!(comp.len(), 2);
        assert!((comp["BWT"] - 0.1).abs() < f64::EPSILON);
        assert!((comp["WWT"] - 11.0).abs() < f64::EPSILON);
        assert!(!comp.contains_key("NLB")); // Only in dam
    }

    #[test]
    fn ebv_glossary_has_all_traits() {
        let glossary = ebv_glossary();
        assert_eq!(glossary.len(), 13);
        let abbreviations: Vec<&str> = glossary.iter().map(|t| t.abbreviation).collect();
        assert!(abbreviations.contains(&"BWT"));
        assert!(abbreviations.contains(&"WWT"));
        assert!(abbreviations.contains(&"NLB"));
        assert!(abbreviations.contains(&"FEC"));
    }

    // ── ancestor_path_combinations ────────────────────────────────────

    #[test]
    fn ancestor_path_combinations_single_depth_each() {
        let combos = ancestor_path_combinations("A1", &[2], &[3]);
        assert_eq!(combos.len(), 1);
        assert_eq!(combos[0].lpn_id, "A1");
        assert_eq!(combos[0].sire_depth, 2);
        assert_eq!(combos[0].dam_depth, 3);
    }

    #[test]
    fn ancestor_path_combinations_multiple_depths() {
        // 2 sire depths × 2 dam depths = 4 combinations
        let combos = ancestor_path_combinations("B1", &[0, 1], &[1, 2]);
        assert_eq!(combos.len(), 4);
        // All should carry the correct lpn_id
        assert!(combos.iter().all(|c| c.lpn_id == "B1"));
        // Verify one specific pairing exists
        assert!(combos.iter().any(|c| c.sire_depth == 0 && c.dam_depth == 2));
        assert!(combos.iter().any(|c| c.sire_depth == 1 && c.dam_depth == 1));
    }

    #[test]
    fn ancestor_path_combinations_empty_sire_depths() {
        let combos = ancestor_path_combinations("C1", &[], &[1, 2]);
        assert!(combos.is_empty());
    }

    #[test]
    fn ancestor_path_combinations_empty_dam_depths() {
        let combos = ancestor_path_combinations("D1", &[0], &[]);
        assert!(combos.is_empty());
    }

    // ── collect_ancestor_depths ───────────────────────────────────────

    #[test]
    fn collect_ancestor_depths_no_parents() {
        let lineage = make_lineage("SUBJ", None, None, vec![]);
        let depths = collect_ancestor_depths(&lineage);
        assert!(depths.is_empty());
    }

    #[test]
    fn collect_ancestor_depths_parents_only() {
        let lineage = make_lineage("SUBJ", Some("SIRE"), Some("DAM"), vec![]);
        let depths = collect_ancestor_depths(&lineage);
        assert_eq!(depths.len(), 2);
        assert_eq!(depths["SIRE"], vec![0]);
        assert_eq!(depths["DAM"], vec![0]);
    }

    #[test]
    fn collect_ancestor_depths_with_generations() {
        let lineage = make_lineage(
            "SUBJ",
            Some("SIRE"),
            Some("DAM"),
            vec![
                vec!["GP1", "GP2"],   // depth 1 (grandparents)
                vec!["GGP1", "GGP2"], // depth 2 (great-grandparents)
            ],
        );
        let depths = collect_ancestor_depths(&lineage);
        assert_eq!(depths["SIRE"], vec![0]);
        assert_eq!(depths["DAM"], vec![0]);
        assert_eq!(depths["GP1"], vec![1]);
        assert_eq!(depths["GP2"], vec![1]);
        assert_eq!(depths["GGP1"], vec![2]);
        assert_eq!(depths["GGP2"], vec![2]);
    }

    #[test]
    fn collect_ancestor_depths_same_animal_at_multiple_depths() {
        // Same ancestor appears in generation 0 and generation 1 (inbred pedigree)
        let lineage = make_lineage(
            "SUBJ",
            Some("SIRE"),
            None,
            vec![
                vec!["COMMON"], // depth 1
                vec!["COMMON"], // depth 2 — same animal again
            ],
        );
        let depths = collect_ancestor_depths(&lineage);
        let common_depths = &depths["COMMON"];
        assert!(common_depths.contains(&1));
        assert!(common_depths.contains(&2));
        assert_eq!(common_depths.len(), 2);
    }

    // ── find_shared_ancestors ─────────────────────────────────────────

    #[test]
    fn find_shared_ancestors_no_overlap() {
        let sire_lin = make_lineage("S", Some("A"), Some("B"), vec![]);
        let dam_lin = make_lineage("D", Some("C"), Some("E"), vec![]);
        let shared = find_shared_ancestors(&sire_lin, &dam_lin);
        assert!(shared.is_empty());
    }

    #[test]
    fn find_shared_ancestors_parent_level() {
        // Both have the same sire parent
        let sire_lin = make_lineage("S", Some("COMMON"), Some("X"), vec![]);
        let dam_lin = make_lineage("D", Some("COMMON"), Some("Y"), vec![]);
        let shared = find_shared_ancestors(&sire_lin, &dam_lin);
        assert_eq!(shared.len(), 1);
        assert_eq!(shared[0].lpn_id, "COMMON");
        assert_eq!(shared[0].sire_depth, 0);
        assert_eq!(shared[0].dam_depth, 0);
    }

    #[test]
    fn find_shared_ancestors_multiple_at_different_depths() {
        // COMMON_A appears at depth 0 in sire and depth 1 in dam
        // COMMON_B appears at depth 1 in sire and depth 0 in dam
        let sire_lin = make_lineage(
            "S",
            Some("COMMON_A"),
            Some("X"),
            vec![vec!["COMMON_B", "GP2"]],
        );
        let dam_lin = make_lineage(
            "D",
            Some("COMMON_B"),
            Some("Y"),
            vec![vec!["COMMON_A", "GP3"]],
        );
        let shared = find_shared_ancestors(&sire_lin, &dam_lin);
        assert_eq!(shared.len(), 2);
        let ids: Vec<&str> = shared.iter().map(|s| s.lpn_id.as_str()).collect();
        assert!(ids.contains(&"COMMON_A"));
        assert!(ids.contains(&"COMMON_B"));
    }

    // ── calculate_coi ────────────────────────────────────────────────

    #[test]
    fn calculate_coi_empty_pedigrees() {
        let sire_lin = make_lineage("S", None, None, vec![]);
        let dam_lin = make_lineage("D", None, None, vec![]);
        let result = calculate_coi(&sire_lin, &dam_lin);
        assert!((result.coefficient - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.rating, CoiRating::Green);
        assert!(result.shared_ancestors.is_empty());
    }

    #[test]
    fn calculate_coi_parent_in_common_depth0() {
        // Shared parent at depth 0 on both sides: 0.5^(0+0+1) = 0.5
        let sire_lin = make_lineage("S", Some("COMMON"), None, vec![]);
        let dam_lin = make_lineage("D", Some("COMMON"), None, vec![]);
        let result = calculate_coi(&sire_lin, &dam_lin);
        assert!((result.coefficient - 0.5).abs() < f64::EPSILON);
        assert_eq!(result.rating, CoiRating::Red);
    }

    #[test]
    fn calculate_coi_three_generation_overlap() {
        // Great-grandparent in common at depth 2 in both sire and dam trees:
        // COI contribution = 0.5^(2+2+1) = 0.5^5 = 0.03125
        let sire_lin = make_lineage(
            "S",
            Some("SS"),
            Some("SD"),
            vec![
                vec!["SSS", "SSD", "SDS", "SDD"],                  // depth 1
                vec!["COMMON", "X", "Y", "Z", "A", "B", "C", "D"], // depth 2
            ],
        );
        let dam_lin = make_lineage(
            "D",
            Some("DS"),
            Some("DD"),
            vec![
                vec!["DSS", "DSD", "DDS", "DDD"],                  // depth 1
                vec!["COMMON", "P", "Q", "R", "E", "F", "G", "H"], // depth 2
            ],
        );
        let result = calculate_coi(&sire_lin, &dam_lin);
        assert!(
            (result.coefficient - 0.03125).abs() < 1e-9,
            "Expected 0.03125, got {}",
            result.coefficient
        );
        assert_eq!(result.rating, CoiRating::Green);
    }

    // ── rank_animals edge cases ───────────────────────────────────────

    #[test]
    fn rank_animals_negative_weights() {
        let animals = vec![
            make_animal("LOW_BWT", &[("BWT", -2.0, 100)]),
            make_animal("HIGH_BWT", &[("BWT", 2.0, 100)]),
        ];
        let mut weights = HashMap::new();
        weights.insert("BWT".to_string(), -1.0); // Lower BWT is better → negative weight

        let ranked = rank_animals(&animals, &weights);
        // LOW_BWT score = -2.0 * -1.0 * 1.0 = 2.0
        // HIGH_BWT score = 2.0 * -1.0 * 1.0 = -2.0
        assert_eq!(ranked[0].lpn_id, "LOW_BWT");
        assert!((ranked[0].score - 2.0).abs() < f64::EPSILON);
        assert_eq!(ranked[1].lpn_id, "HIGH_BWT");
    }

    #[test]
    fn rank_animals_missing_trait_ignored() {
        let animals = vec![
            make_animal("A", &[("WWT", 10.0, 80)]),
            make_animal("B", &[("WWT", 5.0, 80), ("BWT", 1.0, 90)]),
        ];
        let mut weights = HashMap::new();
        weights.insert("WWT".to_string(), 1.0);
        weights.insert("BWT".to_string(), -1.0);

        let ranked = rank_animals(&animals, &weights);
        // A: only WWT → 10.0 * 1.0 * 0.8 = 8.0
        // B: WWT=5*1*0.8=4.0, BWT=1*-1*0.9=-0.9, total=3.1
        assert_eq!(ranked[0].lpn_id, "A");
        assert!((ranked[0].score - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rank_animals_tie_breaking_stable() {
        // Two animals with identical scores should both appear in output
        let animals = vec![
            make_animal("T1", &[("WWT", 10.0, 100)]),
            make_animal("T2", &[("WWT", 10.0, 100)]),
        ];
        let mut weights = HashMap::new();
        weights.insert("WWT".to_string(), 1.0);

        let ranked = rank_animals(&animals, &weights);
        assert_eq!(ranked.len(), 2);
        assert!((ranked[0].score - ranked[1].score).abs() < f64::EPSILON);
    }

    #[test]
    fn rank_animals_empty_input() {
        let animals: Vec<AnimalDetails> = vec![];
        let mut weights = HashMap::new();
        weights.insert("WWT".to_string(), 1.0);
        let ranked = rank_animals(&animals, &weights);
        assert!(ranked.is_empty());
    }

    #[test]
    fn rank_animals_zero_accuracy() {
        // Accuracy 0 → accuracy_factor = 0.0 → score = 0
        let animals = vec![make_animal("Z", &[("WWT", 100.0, 0)])];
        let mut weights = HashMap::new();
        weights.insert("WWT".to_string(), 1.0);
        let ranked = rank_animals(&animals, &weights);
        assert!((ranked[0].score - 0.0).abs() < f64::EPSILON);
    }

    // ── trait_complementarity edge cases ─────────────────────────────

    #[test]
    fn trait_complementarity_no_shared_traits() {
        let sire = make_animal("S", &[("BWT", 1.0, 80)]);
        let dam = make_animal("D", &[("NLB", 0.5, 75)]);
        let comp = trait_complementarity(&sire, &dam);
        assert!(comp.is_empty());
    }

    #[test]
    fn trait_complementarity_both_empty() {
        let sire = make_animal("S", &[]);
        let dam = make_animal("D", &[]);
        let comp = trait_complementarity(&sire, &dam);
        assert!(comp.is_empty());
    }

    #[test]
    fn trait_complementarity_dam_has_extra_traits() {
        // Traits only in dam should not appear
        let sire = make_animal("S", &[("WWT", 8.0, 80)]);
        let dam = make_animal(
            "D",
            &[("WWT", 12.0, 85), ("YWT", 20.0, 75), ("EMD", 5.0, 60)],
        );
        let comp = trait_complementarity(&sire, &dam);
        assert_eq!(comp.len(), 1);
        assert!(comp.contains_key("WWT"));
        assert!(!comp.contains_key("YWT"));
        assert!(!comp.contains_key("EMD"));
        // Midparent = (8.0 + 12.0) / 2.0 = 10.0
        assert!((comp["WWT"] - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn trait_complementarity_sire_has_extra_traits() {
        // Traits only in sire should not appear
        let sire = make_animal("S", &[("WWT", 8.0, 80), ("FAT", 2.0, 70)]);
        let dam = make_animal("D", &[("WWT", 12.0, 85)]);
        let comp = trait_complementarity(&sire, &dam);
        assert_eq!(comp.len(), 1);
        assert!(!comp.contains_key("FAT"));
    }

    // ── ebv_glossary ─────────────────────────────────────────────────

    #[test]
    fn ebv_glossary_all_expected_abbreviations() {
        let glossary = ebv_glossary();
        let abbrevs: std::collections::HashSet<&str> =
            glossary.iter().map(|t| t.abbreviation).collect();
        for expected in &[
            "BWT", "WWT", "PWWT", "YWT", "FAT", "EMD", "NLB", "NWT", "PWT", "DAG", "WGR", "WEC",
            "FEC",
        ] {
            assert!(
                abbrevs.contains(expected),
                "Missing abbreviation: {expected}"
            );
        }
    }

    #[test]
    fn ebv_glossary_fields_nonempty() {
        let glossary = ebv_glossary();
        for entry in &glossary {
            assert!(
                !entry.abbreviation.is_empty(),
                "abbreviation empty for {}",
                entry.name
            );
            assert!(
                !entry.name.is_empty(),
                "name empty for {}",
                entry.abbreviation
            );
            assert!(
                !entry.unit.is_empty(),
                "unit empty for {}",
                entry.abbreviation
            );
            assert!(
                !entry.description.is_empty(),
                "description empty for {}",
                entry.abbreviation
            );
            assert!(
                !entry.selection_direction.is_empty(),
                "selection_direction empty for {}",
                entry.abbreviation
            );
        }
    }

    #[test]
    fn ebv_glossary_unique_abbreviations() {
        let glossary = ebv_glossary();
        let abbrevs: std::collections::HashSet<&str> =
            glossary.iter().map(|t| t.abbreviation).collect();
        assert_eq!(
            abbrevs.len(),
            glossary.len(),
            "Duplicate abbreviations found"
        );
    }
}
