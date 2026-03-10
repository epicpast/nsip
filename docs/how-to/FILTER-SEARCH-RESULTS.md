---
diataxis_type: how-to
---
# How to Filter Search Results

> **Problem:** You need to narrow down animals in the NSIP database by breed, gender, status, date range, flock, or trait values.

**Prerequisites:**
- `nsip` CLI installed, or `nsip` crate added to your `Cargo.toml`
- Knowledge of the breed ID you want to search (use `nsip breed-groups` to find it)

---

## CLI Method

### Step 1: Search with Basic Filters

Use flags to narrow the search:

```bash
nsip search --breed-id 486 --gender Male --status CURRENT
```

Available filters:

| Flag                | Description                                | Example                    |
|---------------------|--------------------------------------------|----------------------------|
| `--breed-id N`      | Filter by breed ID                         | `--breed-id 486`           |
| `--breed-group-id N`| Filter by breed group ID                   | `--breed-group-id 61`      |
| `--gender G`        | `Male`, `Female`, or `Both`                | `--gender Female`          |
| `--status S`        | `CURRENT`, `SOLD`, or `DEAD`               | `--status CURRENT`         |
| `--born-after D`    | Born after date (YYYY-MM-DD)               | `--born-after 2020-01-01`  |
| `--born-before D`   | Born before date (YYYY-MM-DD)              | `--born-before 2023-12-31` |
| `--proven-only`     | Only animals with proven (high-accuracy) EBVs | `--proven-only`         |
| `--flock-id F`      | Filter by flock ID                         | `--flock-id 430735`        |

### Step 2: Sort by a Trait

Sort results by any EBV trait abbreviation:

```bash
nsip search --breed-id 486 --status CURRENT --sort-by WWT
```

Add `--reverse` to sort in descending order (highest first):

```bash
nsip search --breed-id 486 --status CURRENT --sort-by WWT --reverse
```

### Step 3: Paginate Results

Control pagination with `-p` (page number, 0-indexed) and `--page-size`:

```bash
nsip search --breed-id 486 --sort-by WWT --reverse -p 0 --page-size 25
```

---

## Library Method

### Step 1: Build Search Criteria

Use the `SearchCriteria` builder to construct filters:

```rust
use nsip::{NsipClient, SearchCriteria};

let criteria = SearchCriteria::new()
    .with_breed_id(486)
    .with_gender("Male")
    .with_status("CURRENT")
    .with_born_after("2020-01-01")
    .with_born_before("2023-12-31");
```

### Step 2: Execute the Search

Pass the criteria to `search_animals()`:

```rust
let client = NsipClient::new();

let results = client
    .search_animals(
        0,              // page (0-indexed)
        25,             // page_size (1-100)
        Some(486),      // breed_id
        Some("WWT"),    // sort by trait
        Some(true),     // reverse (descending)
        Some(&criteria),
    )
    .await?;

println!("Total matches: {}", results.total_count);
println!("Page {}, showing {} results", results.page, results.results.len());
```

### Step 3: Filter by Trait Ranges

Use `with_trait_ranges()` to constrain results to animals within specific EBV bounds:

```rust
use std::collections::HashMap;
use nsip::{NsipClient, SearchCriteria, TraitRangeFilter};

let mut ranges = HashMap::new();
// BWT in lbs: filter for moderate birth weight EBVs
ranges.insert(
    "BWT".to_string(),
    TraitRangeFilter { min: -0.5, max: 0.5 },
);
// WWT in lbs: filter for above-average weaning weight EBVs
ranges.insert(
    "WWT".to_string(),
    TraitRangeFilter { min: 2.0, max: 10.0 },
);

let criteria = SearchCriteria::new()
    .with_breed_id(486)
    .with_status("CURRENT")
    .with_trait_ranges(ranges);

let client = NsipClient::new();
let results = client
    .search_animals(0, 25, Some(486), None, None, Some(&criteria))
    .await?;
```

### Step 4: Filter by Flock and Proven Status

Narrow to a specific flock and only proven animals:

```rust
let criteria = SearchCriteria::new()
    .with_breed_id(486)
    .with_flock_id("430735")
    .with_proven_only(true);
```

### Step 5: Paginate Through All Results

Iterate through pages to collect all matching animals:

```rust
use nsip::{NsipClient, SearchCriteria};

async fn fetch_all_results(
    client: &NsipClient,
    criteria: &SearchCriteria,
    breed_id: i64,
) -> Result<Vec<serde_json::Value>, nsip::Error> {
    let page_size = 100;
    let mut all_results = Vec::new();
    let mut page = 0;

    loop {
        let results = client
            .search_animals(page, page_size, Some(breed_id), None, None, Some(criteria))
            .await?;

        all_results.extend(results.results);

        if all_results.len() as i64 >= results.total_count {
            break;
        }
        page += 1;
    }

    Ok(all_results)
}
```

---

## MCP Method

Use the `search` tool with filter parameters:

```json
{
  "tool": "search",
  "arguments": {
    "breed_id": 486,
    "gender": "Male",
    "status": "CURRENT",
    "born_after": "2020-01-01",
    "sort_by": "WWT",
    "reverse": true,
    "page_size": 10
  }
}
```

---

## Find Breed and Group IDs

Before filtering by breed, look up the breed ID:

```bash
nsip breed-groups
```

Or programmatically:

```rust
let groups = client.breed_groups().await?;
for group in &groups {
    for breed in &group.breeds {
        println!("{}: {} (group: {})", breed.id, breed.name, group.name);
    }
}
```

---

## Verify Results

1. Check `total_count` on the result to confirm matches exist.
2. If results are empty, relax your filters (broader date range, remove trait ranges).
3. Use `nsip statuses` to verify valid status values for the breed group.

---

## See Also

- [How to Compare Animals](COMPARE-ANIMALS.md) -- compare filtered candidates
- [How to Export JSON](EXPORT-JSON.md) -- export filtered results
- [SearchCriteria Reference](../reference/LIBRARY-API.md#searchcriteria)
- [EBV Trait Glossary](../MCP.md#ebv-trait-glossary)
