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

Both `id` fields are typed `i64`, matching the signed integers the NSIP API emits for these identifiers; the crate does not narrow them, so any value the API returns round-trips faithfully. The names are owned `String`s rather than borrowed slices because a `BreedGroup` is a self-contained value handed back from a parse, with no source buffer to borrow from. Breed group IDs and breed IDs are the numeric identifiers assigned by the NSIP system and used as parameters when searching for animals or querying trait ranges. The 23 participating breeds include: Katahdin, Suffolk, Polypay, Targhee, Dorper, White Suffolk, Dorset, Hampshire, Rambouillet, Columbia, Texel, Romney, Coopworth, Finnsheep, Border Leicester, Southdown, Cheviot, Clun Forest, Shropshire, SAMM (South African Meat Merino), Tunis, Black Welsh Mountain, and various Composite/Commercial/Terminal entries.

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

The `status` field is itself `Option<String>` rather than a closed enum, a deliberate choice: the NSIP API may add or rename statuses over time, and an open string keeps the model tolerant of values the crate has not seen rather than failing to parse them. The set of statuses an evaluation currently uses can be queried dynamically from the API rather than assumed from this table.

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

The design choice that shapes this type is the split between named parents and an open-ended `generations` vector. The `subject`, `sire`, and `dam` fields are each `Option<LineageAnimal>` because the immediate family is what callers reach for most often, and naming them makes the common case direct and self-documenting. Beyond parents, ancestry is modelled as a vector of vectors rather than a fixed set of named grandparent fields: index 0 holds grandparents, index 1 great-grandparents, and so on, growing only as deep as the pedigree the API actually returns. Within each generation, animals appear in pedigree order — sire's sire, sire's dam, dam's sire, dam's dam for the grandparent generation — so position carries meaning. This open-ended shape avoids hardcoding a maximum pedigree depth while keeping the structure traversable by simple indexing.

### Index Values in Lineage

`LineageAnimal` carries selection index values (`us_index`, `src_index`) as `Option<f64>` that are not present in the main `AnimalDetails` struct. They are optional because not every node in a pedigree has a published index, and they live on the lineage node rather than on `AnimalDetails` because the API delivers them embedded in the lineage tree's HTML content — the crate parses them out where they actually arrive. These indexes combine multiple EBVs into a single ranking score and are useful for quick pedigree-level comparisons.

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

The fields encode several deliberate design decisions. Progeny trait values are a bare `HashMap<String, f64>` rather than the full `Trait` struct used on `AnimalDetails`, because the progeny endpoint returns EBV values only — accuracy is not part of a progeny record, so modelling it as `Option<i32>` would imply data that never arrives. The `sex` and `date_of_birth` fields are `Option<String>` to tolerate the API omitting them for individual offspring, while `lpn_id` is a required `String` because an offspring with no identifier could not be looked up or distinguished.

### A Note on the Pagination Types

The pagination fields expose a small but deliberate type asymmetry: `page` and `page_size` are `u32`, while `total_count` is `i64`. The page and page size are caller-supplied request parameters that can never sensibly be negative, so an unsigned type encodes that invariant directly and matches the validation the client performs (page size must be 1–100). The total count, by contrast, originates from the API rather than the caller, and the crate types it as `i64` to round-trip whatever signed integer the upstream JSON provides without narrowing or risking a conversion failure on an unexpected value. The asymmetry is intentional: request-side fields adopt the type that expresses their constraints, response-side fields adopt the type that faithfully mirrors the source. The same pattern appears on `SearchResults`, which shares the `total_count: i64` / `page: u32` / `page_size: u32` layout for the same reasons.

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

Both bounds are plain `f64` and inclusive; there is no separate "unset" state because a `TraitRangeFilter` only exists once a caller has decided to constrain a trait. The companion `TraitRange` type (returned when querying a breed's observed ranges) reports the actual min and max values present for each trait within a breed, which is what lets a caller set a filter inside the achievable range rather than one that would match no animals.

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

The `AnimalProfile` struct aggregates details, lineage, and progeny into a single response.

```rust
pub struct AnimalProfile {
    pub details: AnimalDetails,
    pub lineage: Lineage,
    pub progeny: Progeny,
}
```

All three fields are owned (non-`Option`) composite values rather than optionals, which reflects the construction contract: a profile is built only after its three constituent requests have each succeeded, so the type can guarantee all three parts are present rather than forcing callers to unwrap them. The design rationale for the aggregate is efficiency — the three underlying endpoints (details, lineage, progeny) are independent, so they are fetched concurrently and assembled into one value, giving a complete picture of an animal without serial round-trips.

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
