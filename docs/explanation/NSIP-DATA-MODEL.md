---
diataxis_type: explanation
---
# NSIP Data Model

> How the NSIP Search API organizes sheep genetic evaluation data, from breed groups down to individual trait values.

---

## Overview

The **National Sheep Improvement Program (NSIP)** was founded in 1986 as a non-profit organization with a volunteer board of directors. It operates through a cooperative agreement with **Sheep Genetics (Australia)** -- flock data is processed through **Pedigree Master** software and sent to **LAMBPLAN** for BLUP genetic analysis. This partnership gives U.S. sheep producers access to the same world-class genetic evaluation methodology used by Australian sheep breeders.

NSIP currently serves **23 participating breeds** organized into breed groups, with fee-based enrollment by active seedstock flock size. Data is submitted every two weeks for ongoing EBV development, and new members receive breed-specific mentors to help with data collection and interpretation.

The NSIP data model represents a hierarchy of genetic evaluation data. At the top level, animals are organized into breed groups and breeds. Each animal carries a set of EBV traits, belongs to a flock, has pedigree connections (lineage), and may have progeny records.

The `nsip` crate models this hierarchy through a set of Rust structs that map directly to the API's JSON responses. Understanding these structures is essential for writing effective queries and interpreting results.

---

## The Data Hierarchy

```
Breed Group (e.g., "USA Hair", "USA Range", "USA Terminal")
  |
  +-- Breed (e.g., "Katahdin", "Suffolk", "Targhee")
        |
        +-- Flock (identified by flock_id, associated with ContactInfo)
              |
              +-- Animal (identified by LPN ID)
                    |
                    +-- Traits (HashMap of EBV values with accuracy)
                    +-- Lineage (pedigree tree: sire, dam, grandparents...)
                    +-- Progeny (offspring list with their traits)
```

Each level in this hierarchy corresponds to API endpoints and Rust types in the `nsip` crate.

---

## Breed Groups and Breeds

Breed groups are the top-level organizational unit. Each group contains one or more related breeds that share a common evaluation framework.

```rust
pub struct BreedGroup {
    pub id: i64,        // e.g., 61
    pub name: String,   // e.g., "Range"
    pub breeds: Vec<Breed>,
}

pub struct Breed {
    pub id: i64,        // e.g., 486
    pub name: String,   // e.g., "South African Meat Merino"
}
```

Breed group IDs and breed IDs are numeric identifiers assigned by the NSIP system. They are used as parameters when searching for animals or querying trait ranges. The 23 participating breeds include: Katahdin, Suffolk, Polypay, Targhee, Dorper, White Suffolk, Dorset, Hampshire, Rambouillet, Columbia, Texel, Romney, Coopworth, Finnsheep, Border Leicester, Southdown, Cheviot, Clun Forest, Shropshire, SAMM (South African Meat Merino), Tunis, Black Welsh Mountain, and various Composite/Commercial/Terminal entries.

```bash
# List all breed groups and their breeds
nsip breed-groups
```

```rust
let groups = client.breed_groups().await?;
for group in &groups {
    println!("Group: {} (ID: {})", group.name, group.id);
    for breed in &group.breeds {
        println!("  Breed: {} (ID: {})", breed.name, breed.id);
    }
}
```

The grouping matters because traits are evaluated within breed groups. NSIP uses four primary group categories: **USA Hair** (Katahdin, Dorper, St. Croix), **USA Terminal** (Suffolk, Hampshire, Texel, Dorset, White Suffolk, Southdown), **USA Maternal** (Polypay, Finnsheep, Coopworth, Border Leicester), and **USA Range** (Targhee, Rambouillet, Columbia, SAMM). Not all traits are evaluated for all breeds -- for example, wool traits are only meaningful for wool breeds, and parasite resistance traits (WFEC, PFEC) depend on breed-specific data collection.

See [Breed Groups and Traits](BREED-GROUPS-AND-TRAITS.md) for a detailed discussion of which traits apply to which breed groups.

---

## Animal Identification

Every animal in the NSIP system is identified by a **LPN ID** (Lamb Plan Number). This is a string identifier that uniquely identifies an animal across the entire NSIP database.

LPN IDs are the primary key for looking up animal details, lineage, and progeny. They appear throughout the data model:

- `AnimalDetails.lpn_id` -- the animal itself
- `AnimalDetails.sire` and `AnimalDetails.dam` -- parent references
- `LineageAnimal.lpn_id` -- nodes in the pedigree tree
- `ProgenyAnimal.lpn_id` -- offspring records

---

## Animal Details

The `AnimalDetails` struct is the core data type representing a single animal's record. It contains identification, demographics, and EBV trait values.

```rust
pub struct AnimalDetails {
    pub lpn_id: String,
    pub breed: Option<String>,
    pub breed_group: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,           // "Male" or "Female"
    pub status: Option<String>,           // "CURRENT", "SOLD", "DEAD", etc.
    pub sire: Option<String>,             // Sire's LPN ID
    pub dam: Option<String>,              // Dam's LPN ID
    pub registration_number: Option<String>,
    pub total_progeny: Option<i64>,
    pub flock_count: Option<i64>,
    pub genotyped: Option<String>,
    pub traits: HashMap<String, Trait>,   // Keyed by trait abbreviation
    pub contact_info: Option<ContactInfo>,
}
```

### Why Most Fields Are Optional

The NSIP API returns different subsets of fields depending on the endpoint and the data available for a particular animal. A search result row contains fewer fields than a full detail response. Some animals lack registration numbers, genotyping data, or contact information. The `Option` wrapper handles this gracefully -- callers must explicitly handle the absence of data rather than encountering unexpected nulls.

### The Traits Map

Traits are stored as a `HashMap<String, Trait>` keyed by the standard trait abbreviation (BWT, WWT, PWWT, etc.). This design allows the data model to accommodate any set of traits without hardcoding specific fields for each one.

```rust
pub struct Trait {
    pub name: String,          // Trait abbreviation, e.g., "BWT"
    pub value: f64,            // The EBV value
    pub accuracy: Option<i32>, // Integer percentage 0-100
    pub units: Option<String>, // e.g., "lbs", "mm"
}
```

Note that `accuracy` is `Option<i32>`, not `f64`. The API sometimes returns accuracy as a decimal fraction (0.0--1.0) and sometimes as a percentage (0--100). The `nsip` crate normalizes these to integer percentages internally via the `convert_accuracy` function.

### Animal Status

The `status` field indicates an animal's current standing in the flock:

| Status | Meaning |
|---|---|
| CURRENT | Active in the flock, available for breeding |
| SOLD | Transferred to another flock |
| DEAD | Deceased |

Query the available statuses dynamically:

```bash
nsip statuses
```

---

## Lineage (Pedigree)

The lineage system represents an animal's ancestry as a tree structure. The NSIP API returns pedigree data as a recursive tree where each node has an `lpnId`, HTML `content` (containing farm name, index values, and demographics), and a `children` array (where index 0 is the sire and index 1 is the dam).

The `nsip` crate parses this into a structured `Lineage` type:

```rust
pub struct Lineage {
    pub subject: Option<LineageAnimal>,       // The animal itself
    pub sire: Option<LineageAnimal>,          // Father
    pub dam: Option<LineageAnimal>,           // Mother
    pub generations: Vec<Vec<LineageAnimal>>, // Deeper ancestors
}

pub struct LineageAnimal {
    pub lpn_id: String,
    pub farm_name: Option<String>,
    pub us_index: Option<f64>,     // US (Hair) Index
    pub src_index: Option<f64>,    // SRC$ Index
    pub date_of_birth: Option<String>,
    pub sex: Option<String>,
    pub status: Option<String>,
}
```

### Generations Structure

The `generations` field is a vector of vectors. Index 0 contains grandparents, index 1 contains great-grandparents, and so on. Within each generation, animals appear in pedigree order (sire's sire, sire's dam, dam's sire, dam's dam for the grandparent generation).

```rust
let lineage = client.lineage("6400012006BWR107").await?;

if let Some(sire) = &lineage.sire {
    println!("Sire: {} ({})", sire.lpn_id, sire.farm_name.as_deref().unwrap_or("unknown"));
}

// Grandparents
if let Some(grandparents) = lineage.generations.first() {
    for gp in grandparents {
        println!("Grandparent: {}", gp.lpn_id);
    }
}
```

### Index Values in Lineage

Lineage nodes include selection index values (`us_index`, `src_index`) that are not present in the main `AnimalDetails` struct. These indexes combine multiple EBVs into a single ranking score and are particularly useful for quick pedigree-level comparisons.

---

## Progeny

The progeny endpoint returns a paginated list of an animal's offspring, each with their own trait values.

```rust
pub struct Progeny {
    pub total_count: i64,
    pub animals: Vec<ProgenyAnimal>,
    pub page: u32,
    pub page_size: u32,
}

pub struct ProgenyAnimal {
    pub lpn_id: String,
    pub sex: Option<String>,
    pub date_of_birth: Option<String>,
    pub traits: HashMap<String, f64>,  // Trait values only, no accuracy
}
```

Note that progeny trait values are `f64` (not the full `Trait` struct). Accuracy is not included in progeny records -- only the EBV values themselves.

```bash
nsip progeny 6400012006BWR107
```

```rust
let progeny = client.progeny("6400012006BWR107", 0, 25).await?;
println!("Total offspring: {}", progeny.total_count);
for animal in &progeny.animals {
    let wwt = animal.traits.get("WWT").copied().unwrap_or(0.0);
    println!("  {} - WWT: {:.2}", animal.lpn_id, wwt);
}
```

---

## Search and Filtering

The `SearchCriteria` struct provides a builder-pattern API for constructing search queries:

```rust
pub struct SearchCriteria {
    pub breed_group_id: Option<i64>,
    pub breed_id: Option<i64>,
    pub born_after: Option<String>,
    pub born_before: Option<String>,
    pub gender: Option<String>,
    pub proven_only: Option<bool>,
    pub status: Option<String>,
    pub flock_id: Option<String>,
    pub trait_ranges: Option<HashMap<String, TraitRangeFilter>>,
}
```

Each field corresponds to a filter dimension. Only non-`None` fields are included in the API request body -- omitted fields apply no filter.

### Trait Range Filtering

The `trait_ranges` field allows filtering animals by EBV bounds:

```rust
pub struct TraitRangeFilter {
    pub min: f64,
    pub max: f64,
}
```

Before setting trait range filters, query the valid ranges for a breed:

```bash
nsip trait-ranges 640
```

```rust
let ranges = client.trait_ranges(640).await?;
```

This returns `TraitRange` values showing the observed min/max for each trait within that breed, preventing you from constructing filters that would return zero results.

### Search Results

Search results are paginated and contain raw JSON values:

```rust
pub struct SearchResults {
    pub total_count: i64,
    pub results: Vec<serde_json::Value>,
    pub page: u32,
    pub page_size: u32,
}
```

The `results` are `serde_json::Value` because search result rows use a different field layout than full detail responses. You can parse individual results into `AnimalDetails` using `AnimalDetails::from_api_response()`.

---

## Animal Profile (Combined View)

The `AnimalProfile` struct aggregates details, lineage, and progeny into a single response. The `search_by_lpn` method fetches all three concurrently:

```rust
pub struct AnimalProfile {
    pub details: AnimalDetails,
    pub lineage: Lineage,
    pub progeny: Progeny,
}
```

```rust
let profile = client.search_by_lpn("6400012006BWR107").await?;
// profile.details, profile.lineage, and profile.progeny are all populated
```

This is the most efficient way to get a complete picture of an animal, as it issues the three API calls concurrently rather than sequentially.

---

## Contact Information

Each animal may have associated flock/owner contact information:

```rust
pub struct ContactInfo {
    pub farm_name: Option<String>,
    pub contact_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
}
```

Contact information is typically only present in full detail responses (not search result rows).

---

## API Response Formats

The NSIP API uses inconsistent casing across endpoints. The `nsip` crate handles this transparently:

| Endpoint | Field Convention | Example |
|---|---|---|
| Animal details (nested) | camelCase | `lpnId`, `dateOfBirth` |
| Search results | camelCase | `lpnId`, `bwt`, `accbwt` |
| Legacy endpoints | PascalCase | `LpnId`, `DateOfBirth` |
| Breed groups | Mixed | `breedGroupId` / `Id` |

The `AnimalDetails::from_api_response()` method auto-detects the format and parses accordingly, so callers do not need to handle format differences.

---

## Date Last Updated

The `DateLastUpdated` type wraps the raw response from the database timestamp endpoint:

```rust
pub struct DateLastUpdated {
    pub data: serde_json::Value,
}
```

This is intentionally kept as raw JSON because the API response format may vary. Always check this date before making breeding decisions to ensure you are working with current evaluation data.

---

## Further Reading

- [Understanding EBVs](EBV-EXPLAINED.md) -- what EBV values mean and how to interpret them
- [Genetic Evaluation](GENETIC-EVALUATION.md) -- how BLUP produces the EBV estimates
- [Breed Groups and Traits](BREED-GROUPS-AND-TRAITS.md) -- which traits apply to which breeds
- [Data to Decisions](DATA-TO-DECISIONS.md) -- practical application of NSIP data
- [Getting Started Tutorial](../tutorials/GETTING-STARTED.md) -- hands-on introduction
- [Error Handling Reference](../reference/ERROR-HANDLING.md) -- handling API errors
