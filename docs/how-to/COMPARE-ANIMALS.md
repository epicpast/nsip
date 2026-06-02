---
diataxis_type: how-to
---
# How to Compare Animals

> **Problem:** You need to compare EBV traits across multiple animals to inform breeding or selection decisions.

**Prerequisites:**
- `nsip` CLI installed, or `nsip` crate added to your `Cargo.toml`
- LPN IDs of the animals you want to compare

---

## CLI Method

### Step 1: Run the Compare Command

Compare two or more animals (up to 5) by their LPN IDs:

```bash
nsip compare 430735-0032 430735-0041 430735-0058
```

This outputs a side-by-side ASCII table with all EBV traits aligned for comparison.

### Step 2: Filter to Specific Traits

Use `--traits` to focus on the traits that matter for your breeding goal:

```bash
nsip compare 430735-0032 430735-0041 --traits BWT,WWT,YWT,EMD
```

### Step 3: Get JSON Output

Add `-J` for machine-readable output:

```bash
nsip compare 430735-0032 430735-0041 -J
```

---

## Library Method

### Step 1: Fetch Animal Details

Use `animal_details()` to retrieve EBV data for each animal:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let animal_a = client.animal_details("430735-0032").await?;
    let animal_b = client.animal_details("430735-0041").await?;

    Ok(())
}
```

### Step 2: Fetch Multiple Animals Concurrently

Use `tokio::join!` to fetch details in parallel:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let (a, b, c) = tokio::join!(
        client.animal_details("430735-0032"),
        client.animal_details("430735-0041"),
        client.animal_details("430735-0058"),
    );

    let animals = vec![a?, b?, c?];

    Ok(())
}
```

### Step 3: Compare Specific Traits

Access the `traits` field on `AnimalDetails` to compare EBVs. Trait keys use standard abbreviations (BWT, WWT, YWT, EMD, etc.):

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let (a, b) = tokio::join!(
        client.animal_details("430735-0032"),
        client.animal_details("430735-0041"),
    );
    let animals = vec![a?, b?];

    let traits_of_interest = ["BWT", "WWT", "YWT", "EMD"];

    for animal in &animals {
        println!("Animal: {}", animal.lpn_id);
        for trait_name in &traits_of_interest {
            if let Some(t) = animal.traits.get(*trait_name) {
                println!(
                    "  {}: {:.2} (accuracy: {}%)",
                    t.name,
                    t.value,
                    t.accuracy.unwrap_or(0),
                );
            }
        }
    }

    Ok(())
}
```

### Step 4: Calculate a Weighted Score

Build a simple composite score to rank animals against a breeding objective:

```rust
use std::collections::HashMap;
use nsip::{AnimalDetails, NsipClient};

fn weighted_score(animal: &AnimalDetails, weights: &HashMap<&str, f64>) -> f64 {
    weights
        .iter()
        .filter_map(|(trait_name, weight)| {
            animal.traits.get(*trait_name).map(|t| {
                let accuracy = f64::from(t.accuracy.unwrap_or(50)) / 100.0;
                t.value * weight * accuracy
            })
        })
        .sum()
}

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let (a, b) = tokio::join!(
        client.animal_details("430735-0032"),
        client.animal_details("430735-0041"),
    );
    let animals = vec![a?, b?];

    // Terminal sire objective: penalize birth weight (lbs), reward growth (lbs) and muscle (mm)
    let weights: HashMap<&str, f64> = HashMap::from([
        ("BWT", -1.0),
        ("WWT", 2.0),
        ("YWT", 1.5),
        ("EMD", 1.0),
    ]);

    let mut scored: Vec<_> = animals
        .iter()
        .map(|a| (&a.lpn_id, weighted_score(a, &weights)))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (lpn_id, score) in &scored {
        println!("{lpn_id}: {score:.2}");
    }

    Ok(())
}
```

---

## MCP Method

If you are using the NSIP MCP server through an AI assistant:

```json
{
  "tool": "compare",
  "arguments": {
    "lpn_ids": ["430735-0032", "430735-0041", "430735-0058"],
    "traits": "BWT,WWT,YWT,EMD"
  }
}
```

The `compare` tool returns a structured side-by-side comparison with all requested traits. Missing traits are clearly marked.

For weighted ranking across a breed, use the `rank` tool instead:

```json
{
  "tool": "rank",
  "arguments": {
    "breed_id": 486,
    "weights": { "BWT": -1.0, "WWT": 2.0, "YWT": 1.5, "EMD": 1.0 },
    "gender": "Male",
    "status": "CURRENT",
    "top_n": 5
  }
}
```

---

## Verify Results

After comparing, confirm that:

1. All requested animals were found (check for `NotFound` errors).
2. The traits you care about are present for each animal. Not all animals have all 13 EBV traits.
3. Accuracy values are reasonable -- low-accuracy EBVs (below 40%) should be treated with caution.

---

## See Also

- [How to Filter Search Results](FILTER-SEARCH-RESULTS.md) -- find candidates before comparing
- [How to Export JSON](EXPORT-JSON.md) -- export comparison data for further analysis
- [Understanding EBVs](../explanation/EBV-EXPLAINED.md)
- [MCP Compare Tool Reference](../MCP.md#compare)
