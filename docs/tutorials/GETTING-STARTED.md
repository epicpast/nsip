# Getting Started with NSIP

> **Learning Goal:** By the end of this tutorial, you will have a working Rust program that connects to the NSIP Search API, lists sheep breed groups, searches for animals, and retrieves detailed genetic data.

**Time to complete:** 15 minutes
**Prerequisites:** Rust 1.92+ installed ([rustup.rs](https://rustup.rs/))

---

## What You Will Build

A command-line Rust program that:

1. Connects to the NSIP Search API
2. Lists available breed groups
3. Searches for animals by breed
4. Retrieves detailed genetic data for a specific animal

---

## Step 1: Create a New Project

Open a terminal and create a new Rust project:

```bash
cargo new nsip-demo
cd nsip-demo
```

Add the required dependencies:

```bash
cargo add nsip tokio --features tokio/full
```

Your `Cargo.toml` should now include:

```toml
[dependencies]
nsip = "0.3"
tokio = { version = "1", features = ["full"] }
```

**What just happened?** You created a new Rust binary project and added the `nsip` crate (the NSIP API client) and `tokio` (an async runtime required for making HTTP requests).

---

## Step 2: List Breed Groups

Replace the contents of `src/main.rs` with:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    // Create a new client with default settings
    let client = NsipClient::new();

    // Fetch all breed groups from the NSIP database
    let breed_groups = client.breed_groups().await?;

    println!("Available Breed Groups:\n");
    for group in &breed_groups {
        println!("  {} (ID: {})", group.name, group.id);
        for breed in &group.breeds {
            println!("    - {} (ID: {})", breed.name, breed.id);
        }
        println!();
    }

    Ok(())
}
```

Run the program:

```bash
cargo run
```

You should see output similar to:

```
Available Breed Groups:

  USA Hair (ID: 61)
    - Katahdin (ID: 640)
    - Dorper (ID: 644)
    - St. Croix (ID: 648)
    ...
```

**What just happened?**

- `NsipClient::new()` creates a client with default settings (30-second timeout, 3 retries on server errors).
- `breed_groups()` is an async method that fetches all available sheep breeds from the NSIP database.
- The API organizes breeds into groups (USA Hair, USA Terminal, USA Maternal, USA Range, etc.). Each group contains one or more breeds.
- The `?` operator propagates any errors up to `main`, which returns `Result`.

---

## Step 3: Search for Animals

Now replace `src/main.rs` with a program that searches for animals:

```rust
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    // Build search criteria using the builder pattern
    let criteria = SearchCriteria::new()
        .with_status("CURRENT")
        .with_gender("Male");

    // Search for animals in breed 640 (first page, 5 results)
    let results = client
        .search_animals(
            0,              // page number (0-based)
            5,              // results per page
            Some(640),      // breed_id
            None,           // sorted_trait (no sorting)
            None,           // reverse sort
            Some(&criteria),
        )
        .await?;

    println!("Found {} animals total\n", results.total_count);
    println!("Showing page {} ({} results):\n", results.page, results.results.len());

    for animal in &results.results {
        println!("  {}", animal);
    }

    Ok(())
}
```

Run it:

```bash
cargo run
```

**What just happened?**

- `SearchCriteria::new()` creates an empty filter. The builder methods (`with_status`, `with_gender`) add constraints.
- `with_status("CURRENT")` limits results to active, living animals.
- `with_gender("Male")` filters to rams only. Valid values are `"Male"`, `"Female"`, and `"Both"`.
- `search_animals()` takes pagination parameters (page number and page size), an optional breed ID, optional sorting, and optional search criteria.
- The `results.results` field contains the matching animals as JSON values. The `total_count` tells you how many animals matched overall.

---

## Step 4: Get Animal Details

To fetch detailed genetic information for a specific animal, use the `animal_details` method. Replace `src/main.rs`:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    // Look up a specific animal by search string
    let details = client.animal_details("400001").await?;

    println!("Animal: {}", details.lpn_id);

    if let Some(breed) = &details.breed {
        println!("Breed: {}", breed);
    }
    if let Some(gender) = &details.gender {
        println!("Gender: {}", gender);
    }
    if let Some(status) = &details.status {
        println!("Status: {}", status);
    }
    if let Some(dob) = &details.date_of_birth {
        println!("Date of Birth: {}", dob);
    }

    // Display EBV traits
    if !details.traits.is_empty() {
        println!("\nEBV Traits:");
        for (abbreviation, trait_data) in &details.traits {
            print!("  {} ({}) = {:.2}",
                abbreviation,
                trait_data.name,
                trait_data.value,
            );
            if let Some(acc) = trait_data.accuracy {
                print!("  (accuracy: {}%)", acc);
            }
            if let Some(units) = &trait_data.units {
                print!("  {}", units);
            }
            println!();
        }
    }

    Ok(())
}
```

Run it:

```bash
cargo run
```

**What just happened?**

- `animal_details()` fetches comprehensive data for a single animal, including breed information, status, and all EBV (Estimated Breeding Value) traits.
- The `traits` field is a `HashMap<String, Trait>` keyed by trait abbreviation (e.g., `"BWT"` for Birth Weight, `"WWT"` for Weaning Weight).
- Each `Trait` contains the full name, numeric value, optional accuracy percentage, and optional units.
- Most fields on `AnimalDetails` are `Option` types because not all data is available for every animal.

---

## Step 5: Fetch a Complete Profile

The `search_by_lpn` method combines details, lineage, and progeny into a single `AnimalProfile`:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let profile = client.search_by_lpn("400001").await?;

    // Details
    println!("Animal: {}", profile.details.lpn_id);
    if let Some(breed) = &profile.details.breed {
        println!("Breed: {}", breed);
    }

    // Lineage
    if let Some(sire) = &profile.lineage.sire {
        println!("Sire: {}", sire.lpn_id);
    }
    if let Some(dam) = &profile.lineage.dam {
        println!("Dam: {}", dam.lpn_id);
    }

    // Progeny summary
    println!("Total progeny: {}", profile.progeny.total_count);
    for offspring in &profile.progeny.animals {
        print!("  {} ", offspring.lpn_id);
        if let Some(sex) = &offspring.sex {
            print!("({})", sex);
        }
        println!();
    }

    Ok(())
}
```

**What just happened?**

- `search_by_lpn()` returns an `AnimalProfile` that bundles three pieces of data: `details` (an `AnimalDetails`), `lineage` (a `Lineage` with sire, dam, and multi-generational pedigree), and `progeny` (a `Progeny` with offspring list).
- Unlike `animal_details()`, which returns only the animal's own data, `search_by_lpn()` gives you the full picture in one call.
- The `lineage.generations` field contains a nested vector of ancestors organized by generation depth.

---

## Step 6: Handle Errors

NSIP API calls can fail for various reasons. Here is how to handle errors gracefully:

```rust
use nsip::{Error, NsipClient};

#[tokio::main]
async fn main() {
    let client = NsipClient::new();

    match client.animal_details("INVALID_ID").await {
        Ok(details) => {
            println!("Found: {}", details.lpn_id);
        }
        Err(Error::NotFound(msg)) => {
            eprintln!("Animal not found: {}", msg);
        }
        Err(Error::Timeout(msg)) => {
            eprintln!("Request timed out: {}", msg);
        }
        Err(Error::Api { status, message }) => {
            eprintln!("API error (HTTP {}): {}", status, message);
        }
        Err(Error::Connection(msg)) => {
            eprintln!("Connection failed: {}", msg);
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
}
```

**What just happened?**

- The `nsip::Error` enum has six variants covering all failure modes: `Validation`, `Api`, `NotFound`, `Timeout`, `Connection`, and `Parse`.
- Pattern matching lets you provide specific user-facing messages for each error type.
- See the [Error Handling Reference](../reference/ERROR-HANDLING.md) for full details on each variant.

---

## What You Learned

In this tutorial you:

- Created an `NsipClient` to connect to the NSIP Search API
- Used `breed_groups()` to discover available sheep breeds
- Built search filters with `SearchCriteria` and the builder pattern
- Retrieved individual animal details and EBV traits
- Fetched complete profiles including lineage and progeny
- Handled API errors with pattern matching

---

## Next Steps

Now that you have a working setup, continue with these tutorials:

- [Your First API Query](FIRST-API-QUERY.md) -- a deeper dive into searching and filtering animals
- [Interpreting Results](INTERPRETING-RESULTS.md) -- understand what the genetic data means
- [MCP Server Setup](MCP-SERVER-SETUP.md) -- connect AI assistants to NSIP data

For task-oriented instructions, see the How-To Guides:

- [How to Configure Timeout and Retries](../how-to/CONFIGURE-CLIENT.md)
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md)

For background reading:

- [Understanding EBVs](../explanation/EBV-EXPLAINED.md) -- what Estimated Breeding Values mean
- [Error Handling Reference](../reference/ERROR-HANDLING.md) -- complete error type documentation
