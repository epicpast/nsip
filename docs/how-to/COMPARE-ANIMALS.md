# How to Compare Animals

> **Problem:** You need to compare genetic traits (EBVs) across multiple animals to make breeding decisions.

**Time:** 10 minutes

---

## CLI Method

The fastest way to compare animals is via the CLI:

````bash
nsip compare <lpn-id-1> <lpn-id-2> [<lpn-id-3> ...]
````

**Example:**

````bash
nsip compare ABC123 DEF456 GHI789
````

**Output:** Side-by-side ASCII table with aligned EBV traits.

---

## Programmatic Comparison

Use the library for custom comparison logic:

````rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let lpn_ids = vec!["ABC123", "DEF456"];
    
    // Fetch all profiles concurrently
    let futures = lpn_ids.iter()
        .map(|id| client.search_by_lpn(id));
    
    let profiles = futures::future::try_join_all(futures).await?;

    // Compare specific trait
    for profile in &profiles {
        let weight_trait = profile.details.traits.get("Weight")
            .map(|t| t.value)
            .unwrap_or(0.0);
        
        println!("{}: Weight EBV = {}", profile.details.lpn_id, weight_trait);
    }

    Ok(())
}
````

---

## Using the MCP Tool

If you're using an AI assistant with the MCP server:

````json
{
  "tool": "compare",
  "arguments": {
    "lpn_ids": ["ABC123", "DEF456", "GHI789"]
  }
}
````

The tool returns a structured comparison with:
- Common traits across all animals
- Trait values aligned for easy comparison
- Missing traits clearly marked

---

## Filtering by Trait Ranges

Compare only animals within specific trait ranges:

````rust
use nsip::{NsipClient, SearchCriteria, TraitRangeFilter};

let criteria = SearchCriteria::new()
    .with_trait_range("Weight", TraitRangeFilter {
        min: Some(10.0),
        max: Some(50.0),
    });

let results = client
    .search_animals(0, 10, Some(640), None, None, Some(&criteria))
    .await?;

// Now compare the filtered results
for animal in &results.results {
    println!("{}: Weight in range", animal.lpn_id);
}
````

---

## Ranking by Multiple Traits

Weighted trait comparison:

````rust
use std::collections::HashMap;

// Define trait weights (sum to 1.0)
let weights: HashMap<&str, f64> = [
    ("Weight", 0.4),
    ("Muscle", 0.3),
    ("Fat", 0.3),
].iter().cloned().collect();

// Calculate weighted score for an animal
fn calculate_score(
    animal: &nsip::AnimalDetails,
    weights: &HashMap<&str, f64>
) -> f64 {
    weights.iter()
        .map(|(trait_name, weight)| {
            animal.traits.get(*trait_name)
                .map(|t| t.value * weight)
                .unwrap_or(0.0)
        })
        .sum()
}

// Score all animals
let mut scored: Vec<_> = profiles.iter()
    .map(|p| (p.details.lpn_id.clone(), calculate_score(&p.details, &weights)))
    .collect();

// Sort by score descending
scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

for (lpn_id, score) in scored {
    println!("{}: {:.2}", lpn_id, score);
}
````

---

## See Also

- [How to Calculate Inbreeding Coefficient](INBREEDING-CHECK.md)
- [How to Rank Animals](RANK-ANIMALS.md)
- [Understanding EBVs](../explanation/EBV-EXPLAINED.md)
- [MCP Compare Tool Reference](../MCP.md#compare)
