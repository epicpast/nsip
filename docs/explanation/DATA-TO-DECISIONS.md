# From Data to Decisions

> How NSIP API data connects to real-world sheep breeding decisions, from sire selection through mating plans to flock-level genetic improvement.

---

## The Decision Pipeline

Sheep breeding decisions follow a logical pipeline: define objectives, gather data, evaluate candidates, make selections, plan matings, and track progress. The NSIP system and the `nsip` crate provide data at every stage.

```
Define breeding objective
    |
    v
Identify candidate animals  -->  nsip search (with criteria)
    |
    v
Evaluate EBVs and accuracy  -->  nsip details / nsip profile
    |
    v
Compare candidates           -->  nsip compare
    |
    v
Check pedigree / inbreeding  -->  nsip lineage / MCP inbreeding_check
    |
    v
Plan matings                 -->  MCP mating_recommendations
    |
    v
Monitor genetic trend        -->  nsip search (across years)
```

Each step maps to specific API calls and data types in the `nsip` crate.

---

## Step 1: Define Your Breeding Objective

Before querying the API, you need clarity on what you are selecting for. The breeding objective depends on your production system, market, and constraints.

### Questions to Answer

- **What is the end product?** Market lambs, breeding stock, wool, or a combination?
- **What are the major costs?** Feed, labor, animal health, replacements?
- **What traits limit profitability?** Low weaning rates, poor growth, parasite problems?
- **What are the market specifications?** Weight targets, fat coverage, timing?

The answers determine which EBV traits to prioritize and what selection index to use. See [Breed Groups and Traits](BREED-GROUPS-AND-TRAITS.md) for guidance on matching traits to production systems.

---

## Step 2: Identify Candidate Animals

Use the `SearchCriteria` builder to narrow the candidate pool. Effective searches combine multiple filters:

```rust
let criteria = SearchCriteria::new()
    .with_breed_id(640)           // Katahdin
    .with_status("CURRENT")       // Active animals only
    .with_gender("Male")          // Rams
    .with_proven_only(true);      // Progeny-tested

let results = client.search_animals(0, 50, 640, Some("WWT"), true, &criteria).await?;
```

Or via the CLI:

```bash
nsip search --breed-id 640 --status CURRENT --gender Male --proven-only --sort WWT --reverse
```

### Filtering by Trait Ranges

If you have specific EBV thresholds, use trait range filters to exclude animals that do not meet minimum standards:

```bash
nsip trait-ranges 640
```

This returns the observed range for each trait within the breed. Use these ranges to set realistic filters -- filtering outside the observed range will return zero results.

```rust
use std::collections::HashMap;
use nsip::{SearchCriteria, models::TraitRangeFilter};

let mut ranges = HashMap::new();
ranges.insert("BWT".to_string(), TraitRangeFilter { min: -2.0, max: 0.5 });
ranges.insert("WWT".to_string(), TraitRangeFilter { min: 2.0, max: 10.0 });

let criteria = SearchCriteria::new()
    .with_breed_id(640)
    .with_status("CURRENT")
    .with_gender("Male")
    .with_trait_ranges(ranges);
```

---

## Step 3: Evaluate Individual Animals

Once you have a shortlist, fetch full details for each candidate:

```rust
let profile = client.search_by_lpn("6400012006BWR107").await?;

// Access EBV traits
for (abbrev, trait_data) in &profile.details.traits {
    println!("{}: {:.2} (accuracy: {}%)",
        abbrev,
        trait_data.value,
        trait_data.accuracy.unwrap_or(0),
    );
}
```

### What to Look For

**EBV values relative to breed average.** An EBV of +3.0 for WWT means 3 lbs above the current breed average. But "above average" is only meaningful if you know the breed's range -- use `trait_ranges` for context.

**Accuracy.** Prioritize animals with accuracy above 60% for traits critical to your breeding objective. Low-accuracy animals may look exceptional but carry higher risk. See [Understanding EBVs](EBV-EXPLAINED.md) for accuracy interpretation.

**Trait balance.** An animal with outstanding WWT but very high BWT may cause lambing problems. Check all relevant traits, not just the one you are selecting for.

**Genotyped status.** Animals with genomic data ("genotyped": "Yes") have more precise EBVs, particularly for young animals with limited progeny data.

---

## Step 4: Compare Candidates

Side-by-side comparison reveals relative strengths and weaknesses:

```bash
nsip compare ANIMAL_A ANIMAL_B --traits BWT,WWT,PWWT,NLB,NLW
```

```rust
let profile_a = client.search_by_lpn("ANIMAL_A").await?;
let profile_b = client.search_by_lpn("ANIMAL_B").await?;

let traits_to_compare = ["BWT", "WWT", "PWWT", "NLB", "NLW"];
for trait_name in &traits_to_compare {
    let val_a = profile_a.details.traits.get(*trait_name);
    let val_b = profile_b.details.traits.get(*trait_name);
    println!("{}: A={:.2} (acc {}%), B={:.2} (acc {}%)",
        trait_name,
        val_a.map(|t| t.value).unwrap_or(0.0),
        val_a.and_then(|t| t.accuracy).unwrap_or(0),
        val_b.map(|t| t.value).unwrap_or(0.0),
        val_b.and_then(|t| t.accuracy).unwrap_or(0),
    );
}
```

### Decision Factors Beyond EBVs

EBVs are the primary selection tool, but real-world decisions also consider:

- **Physical soundness** -- structural correctness, feet, teeth, reproductive anatomy
- **Temperament** -- docile animals are safer to handle and less stressed
- **Price** -- the economic return must justify the purchase cost
- **Logistics** -- distance, quarantine requirements, health testing
- **Genetic diversity** -- avoid over-reliance on a single sire line

---

## Step 5: Check Pedigree and Inbreeding

Before finalizing mating decisions, examine the pedigree to assess genetic relationships:

```bash
nsip lineage ANIMAL_ID
```

```rust
let lineage = client.lineage("ANIMAL_ID").await?;

if let Some(sire) = &lineage.sire {
    println!("Sire: {}", sire.lpn_id);
}
if let Some(dam) = &lineage.dam {
    println!("Dam: {}", dam.lpn_id);
}

// Check deeper ancestry
for (gen_idx, generation) in lineage.generations.iter().enumerate() {
    println!("Generation {}: {} ancestors", gen_idx + 1, generation.len());
}
```

### Inbreeding Concerns

Mating related animals concentrates genes -- both desirable and undesirable ones. Inbreeding depression in sheep typically manifests as:

- Reduced lamb survival
- Lower fertility
- Decreased immune function
- Slower growth

The MCP server provides an `inbreeding_check` tool that calculates the Coefficient of Inbreeding (COI) for a proposed mating using Wright's path formula on the available pedigree data. Use it before committing to a mating plan.

### Using Pedigree Data for Diversity

The lineage data reveals how related animals in your shortlist are. If your top two ram candidates share a grandsire, using both in the same flock would concentrate that line. Consider selecting genetically diverse candidates to maintain long-term genetic variation.

---

## Step 6: Plan Matings

Mating allocation -- deciding which ram mates which ewes -- is where all the previous analysis comes together. The objectives are:

1. **Maximize expected genetic progress** by matching superior sires with the most ewes
2. **Correct specific weaknesses** by pairing animals whose strengths compensate for each other's weaknesses
3. **Manage inbreeding** by avoiding matings between closely related animals
4. **Maintain genetic diversity** by not over-using any single sire

### Corrective Mating

If a ewe has a high BWT EBV (undesirable), mate her to a ram with low BWT to pull the offspring toward the desired range. The expected offspring EBV is approximately the average of the parents' EBVs:

```
Expected offspring EBV = (Sire EBV + Dam EBV) / 2
```

This is an approximation -- actual offspring will vary around this expected value due to Mendelian sampling (the random 50% of each parent's genes that gets passed on).

### Using the MCP Server for Mating Recommendations

The MCP server's `mating_recommendations` tool automates the mating allocation process, considering EBV complementarity and inbreeding avoidance:

```bash
nsip mcp
# Use the mating_recommendations tool with sire and dam LPN IDs
```

---

## Step 7: Monitor Progress

Genetic improvement is a long-term process. Track progress by monitoring genetic trends across lamb crops:

### Year-Over-Year Comparison

Search for animals born in different years and compare average EBVs:

```rust
let criteria_2024 = SearchCriteria::new()
    .with_breed_id(640)
    .with_born_after("2024-01-01")
    .with_born_before("2024-12-31");

let criteria_2023 = SearchCriteria::new()
    .with_breed_id(640)
    .with_born_after("2023-01-01")
    .with_born_before("2023-12-31");

let results_2024 = client.search_animals(0, 100, 640, None, false, &criteria_2024).await?;
let results_2023 = client.search_animals(0, 100, 640, None, false, &criteria_2023).await?;

// Compare average EBVs between years
```

### Progeny Analysis

For a sire in active use, monitor his progeny performance:

```rust
let progeny = client.progeny("SIRE_LPN_ID", 0, 100).await?;
println!("Total progeny: {}", progeny.total_count);

// Calculate average trait values across progeny
let wwt_values: Vec<f64> = progeny.animals.iter()
    .filter_map(|a| a.traits.get("WWT").copied())
    .collect();

if !wwt_values.is_empty() {
    let avg: f64 = wwt_values.iter().sum::<f64>() / wwt_values.len() as f64;
    println!("Average WWT of progeny: {:.2}", avg);
}
```

---

## Putting It All Together: A Worked Example

Consider a commercial Katahdin flock aiming to improve weaning weight while keeping birth weight under control. Katahdin is the most-represented breed in the NSIP system, accounting for approximately 35% of all records, which means there is a large pool of evaluated animals to choose from and generally higher accuracy EBVs.

**Objective:** Increase WWT without increasing BWT.

**Search criteria:**
- Breed: Katahdin (breed_id 640)
- Gender: Male (rams)
- Status: CURRENT
- Proven only: true (high accuracy)
- BWT range: -2.0 to 0.5 (keep birth weight low)
- WWT range: 3.0+ (above average growth)

```bash
nsip search --breed-id 640 --status CURRENT --gender Male --proven-only --sort WWT --reverse
```

**Evaluate top candidates:** Fetch profiles for the top 5 rams. Check WWT, BWT, MWWT (maternal trait for daughters), NLW, accuracy, and genotyped status. Also check the **USA MAT-HAIR Index** value, which combines DWWT, MWWT, NLB, and NLW into a single selection score with NLW most heavily weighted.

**Compare finalists:** Use `nsip compare` on the top 2-3 candidates. Look for the best balance of high WWT, low BWT, acceptable NLW, strong MWWT, and high accuracy. If parasite resistance matters in your environment, also check WFEC -- a ram with WFEC of -90% can reduce worm burden in his lambs by approximately 45%.

**Check pedigree:** Verify the chosen ram is not related to existing flock sires. Use the lineage endpoint to inspect ancestry 3-4 generations deep.

**Allocate matings:** Use the ram across the ewe flock. Consider splitting ewes between two genetically diverse rams to maintain variation. Use corrective mating for ewes with high BWT by pairing them with the ram having the lowest BWT. For Katahdin flocks, the USA MAT-HAIR Index provides a convenient single-number ranking that balances growth and maternal traits.

**Track results:** In the next evaluation cycle, compare the new lamb crop's average EBVs to the previous year. A successful sire choice will show improvement in WWT without regression in BWT. Since NSIP data is submitted every two weeks and processed through LAMBPLAN, updated EBVs reflecting new progeny data become available on a regular cycle.

---

## Common Decision Pitfalls

**Chasing the highest number.** The animal with the highest EBV for one trait is rarely the best overall choice. Consider the full trait profile and accuracy.

**Ignoring accuracy.** A young ram with EBV +8 at 25% accuracy is riskier than a proven ram with EBV +5 at 85% accuracy. The proven ram's true value is more certain.

**Forgetting inbreeding.** Short-term gains from mating the best to the best can create long-term inbreeding problems. Maintain genetic diversity.

**Neglecting maternal traits.** Terminal sire selection is straightforward, but self-replacing flocks must select for both growth and maternal performance. NLW and MWWT matter -- the USA MAT-HAIR Index combines these for hair sheep breeds.

**Using outdated data.** Always check `nsip date-updated` to confirm you are working with the latest evaluation. EBVs change as new data arrives.

---

## Further Reading

- [Understanding EBVs](EBV-EXPLAINED.md) -- the foundation of genetic selection
- [Genetic Evaluation](GENETIC-EVALUATION.md) -- how EBVs are calculated
- [Breed Groups and Traits](BREED-GROUPS-AND-TRAITS.md) -- choosing the right traits for your system
- [NSIP Data Model](NSIP-DATA-MODEL.md) -- navigating the API data structures
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) -- step-by-step comparison guide
- [How to Configure Client](../how-to/CONFIGURE-CLIENT.md) -- customizing timeout and retry settings
- [MCP Server Reference](../MCP.md) -- using the analytics tools for ranking and mating
