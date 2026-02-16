# Getting Started with NSIP

> **Learning Goal:** By the end of this tutorial, you'll search for sheep, retrieve detailed genetic data, and understand how to use the NSIP API client.

**Time to complete:** 15 minutes  
**Prerequisites:** Rust 1.92+ installed

---

## What You'll Build

A simple Rust program that:
1. Connects to the NSIP Search API
2. Lists available breed groups
3. Searches for animals by breed
4. Retrieves detailed genetic data for a specific animal

---

## Step 1: Create a New Project

````bash
cargo new nsip-demo
cd nsip-demo
````

Add `nsip` to your dependencies:

````bash
cargo add nsip tokio --features tokio/full
````

Your `Cargo.toml` should now include:

````toml
[dependencies]
nsip = "0.3"
tokio = { version = "1", features = ["full"] }
````

---

## Step 2: List Breed Groups

Replace the contents of `src/main.rs`:

````rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    // Create a new client
    let client = NsipClient::new();

    // List all breed groups
    let breed_groups = client.breed_groups().await?;
    
    println!("📋 Available Breed Groups:\n");
    for group in &breed_groups {
        println!("  {} (ID: {})", group.name, group.id);
        for breed in &group.breeds {
            println!("    └─ {}", breed.name);
        }
        println!();
    }

    Ok(())
}
````

Run the program:

````bash
cargo run
````

**Expected output:**
````
📋 Available Breed Groups:

  Maternal (ID: 640)
    └─ Border Leicester
    └─ Coopworth
    └─ Corriedale
    ...
````

**What just happened?**
- `NsipClient::new()` creates a client with default settings (30s timeout, 3 retries)
- `breed_groups()` fetches all available sheep breeds from the NSIP database
- The API organizes breeds into groups (Maternal, Terminal, Dual Purpose, etc.)

---

## Step 3: Search for Animals

Add search functionality below the breed groups code:

````rust
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    // Search for Maternal breed animals (breed_id: 640)
    let criteria = SearchCriteria::new()
        .with_status("CURRENT")
        .with_gender("M");  // Male animals only

    let results = client
        .search_animals(
            0,           // page number
            10,          // page size
            Some(640),   // breed_id (Maternal)
            None,        // sorted_trait
            None,        // reverse sort
            Some(&criteria)
        )
        .await?;

    println!("🔍 Search Results: {} animals found\n", results.total_count);
    
    for animal in &results.results {
        println!("  {} - {} ({}) - Flock: {}", 
            animal.lpn_id,
            animal.breed_name,
            animal.animal_sex,
            animal.flock_id.as_deref().unwrap_or("N/A")
        );
    }

    Ok(())
}
````

Run again:

````bash
cargo run
````

**What's happening?**
- `SearchCriteria::new()` creates a search filter using the builder pattern
- `with_status("CURRENT")` filters to currently active animals
- `with_gender("M")` limits results to male animals (rams)
- The search is paginated (page 0, 10 results per page)

---

## Step 4: Get Detailed Animal Data

Now fetch detailed genetic information for a specific animal:

````rust
#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    // Get the first animal from search results
    let criteria = SearchCriteria::new().with_status("CURRENT");
    let results = client.search_animals(0, 1, Some(640), None, None, Some(&criteria)).await?;
    
    if let Some(first_animal) = results.results.first() {
        let lpn_id = &first_animal.lpn_id;
        
        // Fetch detailed profile (details + lineage + progeny)
        let profile = client.search_by_lpn(lpn_id).await?;
        
        println!("🐑 Animal Profile: {}\n", lpn_id);
        println!("  Breed: {}", profile.details.breed_name);
        println!("  Gender: {}", profile.details.animal_sex);
        println!("  Status: {}", profile.details.animal_status);
        println!("  Date of Birth: {}", profile.details.date_of_birth);
        
        // Display EBV traits
        if !profile.details.traits.is_empty() {
            println!("\n  📊 EBV Traits:");
            for (trait_name, trait_data) in &profile.details.traits {
                println!("    {} = {} (accuracy: {})", 
                    trait_name, 
                    trait_data.value, 
                    trait_data.accuracy
                );
            }
        }
        
        // Display lineage
        if let Some(lineage) = &profile.lineage {
            println!("\n  🌳 Lineage:");
            if let Some(sire) = &lineage.sire {
                println!("    Sire: {}", sire.lpn_id);
            }
            if let Some(dam) = &lineage.dam {
                println!("    Dam: {}", dam.lpn_id);
            }
        }
        
        // Display progeny count
        if let Some(progeny) = &profile.progeny {
            println!("\n  👶 Progeny: {} offspring", progeny.total_count);
        }
    }

    Ok(())
}
````

**What you learned:**
- `search_by_lpn()` fetches a complete animal profile in one efficient call
- EBV (Estimated Breeding Values) traits are stored in a `HashMap<String, Trait>`
- The `AnimalProfile` combines details, lineage, and progeny data
- All API calls use async/await and return `Result<T, nsip::Error>`

---

## Step 5: Error Handling

Handle common errors gracefully:

````rust
use nsip::{NsipClient, Error};

#[tokio::main]
async fn main() {
    let client = NsipClient::new();

    match client.animal_details("INVALID_ID").await {
        Ok(animal) => {
            println!("Found: {}", animal.lpn_id);
        }
        Err(Error::NotFound(msg)) => {
            eprintln!("❌ Animal not found: {}", msg);
        }
        Err(Error::Timeout(msg)) => {
            eprintln!("⏱️  Request timed out: {}", msg);
        }
        Err(Error::Api { status, message }) => {
            eprintln!("🚨 API error (HTTP {}): {}", status, message);
        }
        Err(e) => {
            eprintln!("💥 Unexpected error: {}", e);
        }
    }
}
````

**Error types:**
- `Error::NotFound` - Animal/resource doesn't exist (HTTP 404)
- `Error::Timeout` - Request exceeded timeout duration
- `Error::Api` - Server returned an error status
- `Error::Connection` - Network connectivity issues
- `Error::Validation` - Invalid input parameters
- `Error::Parse` - Failed to deserialize API response

---

## Next Steps

**Tutorials:**
- [Working with EBV Traits](EBV-TRAITS.md) - Filter and rank animals by genetic traits
- [MCP Integration](MCP-INTEGRATION.md) - Connect AI assistants to NSIP data

**How-To Guides:**
- [How to Configure Timeout and Retries](../how-to/CONFIGURE-CLIENT.md)
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md)
- [How to Calculate Inbreeding Coefficient](../how-to/INBREEDING-CHECK.md)

**Reference:**
- [API Reference](../MCP.md) - Complete API documentation
- [Error Handling Reference](../reference/ERROR-HANDLING.md)
