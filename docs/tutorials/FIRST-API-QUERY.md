# Your First API Query

> **Learning Goal:** By the end of this tutorial, you will know how to construct targeted searches using `SearchCriteria`, paginate through large result sets, sort by genetic traits, and combine multiple filters to find exactly the animals you need.

**Time to complete:** 15 minutes
**Prerequisites:**
- Completed the [Getting Started](GETTING-STARTED.md) tutorial
- A Rust project with `nsip` and `tokio` in your dependencies

---

## What You Will Build

A Rust program that:

1. Discovers available breeds and their IDs
2. Builds targeted search queries with multiple filters
3. Pages through results and sorts by genetic traits
4. Retrieves detailed profiles for interesting animals

---

## Step 1: Discover Breeds

Before searching for animals, you need to know which breed IDs to use. Create a new file `src/main.rs`:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let breed_groups = client.breed_groups().await?;

    for group in &breed_groups {
        println!("{} (group ID: {})", group.name, group.id);
        for breed in &group.breeds {
            println!("  {} (breed ID: {})", breed.name, breed.id);
        }
    }

    Ok(())
}
```

Run it and note the breed IDs you are interested in:

```bash
cargo run
```

**What just happened?** Each breed group has an ID, and each breed within a group has its own ID. You will use breed IDs when searching for animals. For example, if "Poll Dorset" has breed ID 645, you would pass `Some(645)` to `search_animals`.

---

## Step 2: Build a Basic Search

Now search for animals in a specific breed. Replace `src/main.rs`:

```rust
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    // Search for current female animals
    let criteria = SearchCriteria::new()
        .with_status("CURRENT")
        .with_gender("Female");

    let results = client
        .search_animals(
            0,              // first page
            10,             // 10 results per page
            Some(640),      // breed_id -- replace with your breed
            None,           // no trait sorting
            None,           // default sort order
            Some(&criteria),
        )
        .await?;

    println!("Total matches: {}", results.total_count);
    println!("Page: {}, page size: {}", results.page, results.page_size);
    println!("Results on this page: {}\n", results.results.len());

    for (i, animal) in results.results.iter().enumerate() {
        println!("  [{}] {}", i + 1, animal);
    }

    Ok(())
}
```

Run it:

```bash
cargo run
```

**What just happened?**

- `SearchCriteria` uses the builder pattern. Each `.with_*` method returns a new `SearchCriteria` with the added filter, so you can chain them.
- `with_status("CURRENT")` filters to animals that are alive and actively evaluated. Other valid values: `"SOLD"`, `"DEAD"`.
- `with_gender("Female")` limits results to ewes. Valid values: `"Male"`, `"Female"`, `"Both"`.
- The first argument to `search_animals` is the page number (0-based), and the second is the page size.

---

## Step 3: Add More Filters

The `SearchCriteria` builder supports several additional filters. Try combining them:

```rust
use std::collections::HashMap;
use nsip::{NsipClient, SearchCriteria, TraitRangeFilter};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    // Find proven rams born after 2020
    let criteria = SearchCriteria::new()
        .with_status("CURRENT")
        .with_gender("Male")
        .with_proven_only(true)
        .with_born_after("2020-01-01");

    let results = client
        .search_animals(0, 10, Some(640), None, None, Some(&criteria))
        .await?;

    println!("Proven rams born after 2020: {}\n", results.total_count);

    // Now add trait range filters
    let mut trait_ranges = HashMap::new();
    trait_ranges.insert(
        "BWT".to_string(),
        TraitRangeFilter { min: -1.0, max: 2.0 },
    );

    let criteria_with_traits = SearchCriteria::new()
        .with_status("CURRENT")
        .with_gender("Male")
        .with_born_after("2020-01-01")
        .with_trait_ranges(trait_ranges);

    let filtered = client
        .search_animals(0, 10, Some(640), None, None, Some(&criteria_with_traits))
        .await?;

    println!("With BWT between -1.0 and 2.0: {}", filtered.total_count);

    Ok(())
}
```

**What just happened?**

- `with_proven_only(true)` limits results to animals with progeny records, meaning their EBV estimates are backed by offspring data.
- `with_born_after("2020-01-01")` filters by date of birth. There is also `with_born_before()` for the upper bound.
- `with_trait_ranges()` takes a `HashMap<String, TraitRangeFilter>` to filter animals by EBV values. The example filters for animals with Birth Weight (BWT) EBV between -1.0 and 2.0 lbs.
- You can check what trait ranges are valid for a breed using `client.trait_ranges(breed_id)`.

---

## Step 4: Sort Results by a Trait

To find top-performing animals, sort the search results by a specific EBV trait:

```rust
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let criteria = SearchCriteria::new()
        .with_status("CURRENT")
        .with_gender("Male");

    // Sort by Weaning Weight (WWT), highest first
    let top_weaners = client
        .search_animals(
            0,
            5,
            Some(640),
            Some("WWT"),    // sort by Weaning Weight
            Some(true),     // reverse = true means descending (highest first)
            Some(&criteria),
        )
        .await?;

    println!("Top 5 rams by Weaning Weight:\n");
    for animal in &top_weaners.results {
        println!("  {}", animal);
    }

    // Sort by Birth Weight (BWT), lowest first (lighter birth weights are preferred)
    let low_bwt = client
        .search_animals(
            0,
            5,
            Some(640),
            Some("BWT"),    // sort by Birth Weight
            Some(false),    // reverse = false means ascending (lowest first)
            Some(&criteria),
        )
        .await?;

    println!("\nTop 5 rams by lowest Birth Weight:\n");
    for animal in &low_bwt.results {
        println!("  {}", animal);
    }

    Ok(())
}
```

**What just happened?**

- The `sorted_trait` parameter accepts a trait abbreviation like `"WWT"` (Weaning Weight), `"BWT"` (Birth Weight), or any available EBV trait for the breed.
- The `reverse` parameter controls sort direction: `Some(true)` for descending (highest first), `Some(false)` for ascending (lowest first).
- Which direction is "best" depends on the trait. For growth traits (WWT, PWWT, YWT), higher is generally better. For Birth Weight, lower is often preferred to reduce lambing difficulty.

---

## Step 5: Paginate Through All Results

When a search returns more animals than one page, use pagination to walk through the full result set:

```rust
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let criteria = SearchCriteria::new().with_status("CURRENT");
    let page_size = 20;
    let mut page = 0;
    let mut total_fetched = 0;

    loop {
        let results = client
            .search_animals(page, page_size, Some(640), None, None, Some(&criteria))
            .await?;

        let count = results.results.len();
        total_fetched += count;

        println!(
            "Page {}: {} results (total so far: {}/{})",
            page, count, total_fetched, results.total_count
        );

        // Stop when we have fetched all results or received an empty page
        if count == 0 || total_fetched as i64 >= results.total_count {
            break;
        }

        page += 1;
    }

    println!("\nDone. Fetched {} animals total.", total_fetched);

    Ok(())
}
```

**What just happened?**

- Each call to `search_animals` returns one page of results. The `total_count` field tells you the overall number of matching animals.
- Increment the `page` parameter to fetch the next page. Pages are 0-indexed.
- The loop stops when either the returned page is empty or we have fetched all matching results.
- Be mindful of rate limits when paginating through large result sets. A page size of 20-50 is reasonable for most use cases.

---

## What You Learned

In this tutorial you:

- Discovered breed groups and their IDs with `breed_groups()`
- Built searches with `SearchCriteria` using status, gender, date, and trait range filters
- Sorted results by EBV traits in ascending or descending order
- Paginated through large result sets

---

## Next Steps

- [Interpreting Results](INTERPRETING-RESULTS.md) -- understand what the EBV numbers mean and how to compare them
- [Understanding EBVs](../explanation/EBV-EXPLAINED.md) -- background on Estimated Breeding Values and accuracy
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) -- side-by-side trait comparisons using the CLI or library
- [Error Handling Reference](../reference/ERROR-HANDLING.md) -- handle API errors gracefully
