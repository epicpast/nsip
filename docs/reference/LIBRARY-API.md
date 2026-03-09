# Library API Reference

Complete reference for the `nsip` Rust library crate.

---

## Crate Exports

```rust
pub use client::NsipClient;
pub use models::{
    AnimalDetails, AnimalProfile, Breed, BreedGroup, ContactInfo,
    DateLastUpdated, Lineage, LineageAnimal, Progeny, ProgenyAnimal,
    SearchCriteria, SearchResults, Trait, TraitRange, TraitRangeFilter,
};
pub mod mcp;
pub enum Error { /* ... */ }
pub type Result<T> = std::result::Result<T, Error>;
```

---

## NsipClient

HTTP client for the NSIP Search API at `nsipsearch.nsip.org/api`. All data-fetching methods are `async` and require a Tokio runtime.

### Construction

#### `NsipClient::new() -> Self`

Create a client with default settings (base URL `http://nsipsearch.nsip.org/api`, 30-second timeout, 3 retries).

```rust
use nsip::NsipClient;

let client = NsipClient::new();
```

#### `NsipClient::with_base_url(base_url: impl Into<String>) -> Self`

Create a client with a custom base URL. Uses default timeout and retry settings.

```rust
let client = NsipClient::with_base_url("http://localhost:8080/api");
```

#### `NsipClient::builder() -> NsipClientBuilder`

Create a builder for full control over client configuration.

```rust
let client = NsipClient::builder()
    .base_url("http://localhost:8080/api")
    .timeout_secs(60)
    .max_retries(5)
    .build()?;
```

#### `NsipClient::base_url(&self) -> &str`

Return the configured base URL.

```rust
let client = NsipClient::new();
assert_eq!(client.base_url(), "http://nsipsearch.nsip.org/api");
```

---

### NsipClientBuilder

Builder for constructing an `NsipClient` with custom settings.

#### `base_url(self, url: impl Into<String>) -> Self`

Set the API base URL. Default: `http://nsipsearch.nsip.org/api`.

#### `timeout_secs(self, secs: u64) -> Self`

Set the request timeout in seconds. Default: 30. This is a `const fn`.

#### `max_retries(self, retries: u32) -> Self`

Set the maximum number of retries for server errors (HTTP 500, 502, 503, 504). Default: 3. This is a `const fn`.

#### `build(self) -> Result<NsipClient>`

Build the client. Returns `Error::Connection` if the underlying HTTP client cannot be constructed.

```rust
let client = NsipClient::builder()
    .timeout_secs(60)
    .max_retries(5)
    .build()?;
```

---

### Methods

All methods are `async` and return `Result<T>`.

#### `date_last_updated(&self) -> Result<DateLastUpdated>`

Get the date when the NSIP database was last updated.

**API endpoint:** `GET search/getDateLastUpdated`

**Errors:** `Connection`, `Timeout`, `Api`, `Parse`

```rust
let updated = client.date_last_updated().await?;
println!("{}", serde_json::to_string_pretty(&updated.data)?);
```

---

#### `breed_groups(&self) -> Result<Vec<BreedGroup>>`

List all available breed groups and the individual breeds within each group.

**API endpoint:** `GET search/getAvailableBreedGroups`

**Errors:** `Connection`, `Timeout`, `Api`, `Parse`

```rust
let groups = client.breed_groups().await?;
for group in &groups {
    println!("{}: {} breeds", group.name, group.breeds.len());
}
```

---

#### `statuses(&self) -> Result<Vec<String>>`

List all available animal statuses.

**API endpoint:** `GET search/getStatusesByBreedGroup`

**Errors:** `Connection`, `Timeout`, `Api`, `Parse`

```rust
let statuses = client.statuses().await?;
// e.g., ["CURRENT", "SOLD", "DEAD"]
```

---

#### `trait_ranges(&self, breed_id: i64) -> Result<serde_json::Value>`

Get the minimum and maximum EBV trait values for a specific breed.

**API endpoint:** `GET search/getTraitRangesByBreed?breedId={breed_id}`

**Parameters:**

| Parameter | Type | Validation |
|-----------|------|------------|
| `breed_id` | `i64` | Must be > 0 |

**Errors:** `Validation` (if `breed_id <= 0`), `Connection`, `Timeout`, `Api`, `Parse`

```rust
let ranges = client.trait_ranges(486).await?;
```

---

#### `search_animals(&self, page: u32, page_size: u32, breed_id: Option<i64>, sorted_trait: Option<&str>, reverse: Option<bool>, criteria: Option<&SearchCriteria>) -> Result<SearchResults>`

Search for animals with filters and pagination.

**API endpoint:** `POST search/getPageOfSearchResults`

**Parameters:**

| Parameter | Type | Validation | Description |
|-----------|------|------------|-------------|
| `page` | `u32` | -- | Page number (0-indexed) |
| `page_size` | `u32` | 1-100 | Results per page |
| `breed_id` | `Option<i64>` | -- | Breed filter |
| `sorted_trait` | `Option<&str>` | -- | Trait abbreviation to sort by |
| `reverse` | `Option<bool>` | -- | Reverse sort order |
| `criteria` | `Option<&SearchCriteria>` | -- | Additional search criteria |

**Errors:** `Validation` (if `page_size == 0` or `page_size > 100`), `Connection`, `Timeout`, `Api`, `Parse`

```rust
use nsip::{NsipClient, SearchCriteria};

let client = NsipClient::new();
let criteria = SearchCriteria::new()
    .with_breed_id(486)
    .with_status("CURRENT")
    .with_gender("Male");

let results = client.search_animals(
    0,           // page
    15,          // page_size
    Some(486),   // breed_id
    Some("WWT"), // sort by weaning weight
    None,        // default sort order
    Some(&criteria),
).await?;

println!("Found {} animals", results.total_count);
```

---

#### `animal_details(&self, search_string: &str) -> Result<AnimalDetails>`

Get detailed EBV data, breed, contact info, and status for a specific animal.

**API endpoint:** `GET details/getAnimalDetails?searchString={search_string}`

**Parameters:**

| Parameter | Type | Validation |
|-----------|------|------------|
| `search_string` | `&str` | Must not be empty or whitespace-only |

**Errors:** `Validation` (if empty/whitespace), `NotFound`, `Connection`, `Timeout`, `Api`, `Parse`

```rust
let animal = client.animal_details("430735-0032").await?;
println!("LPN: {}, Breed: {:?}", animal.lpn_id, animal.breed);
```

---

#### `lineage(&self, lpn_id: &str) -> Result<Lineage>`

Get pedigree / ancestry tree for a specific animal including parents and grandparents.

**API endpoint:** `GET details/getLineage?lpnId={lpn_id}`

**Parameters:**

| Parameter | Type | Validation |
|-----------|------|------------|
| `lpn_id` | `&str` | Must not be empty or whitespace-only |

**Errors:** `Validation` (if empty/whitespace), `NotFound`, `Connection`, `Timeout`, `Api`, `Parse`

```rust
let lineage = client.lineage("430735-0032").await?;
if let Some(sire) = &lineage.sire {
    println!("Sire: {}", sire.lpn_id);
}
if let Some(dam) = &lineage.dam {
    println!("Dam: {}", dam.lpn_id);
}
```

---

#### `progeny(&self, lpn_id: &str, page: u32, page_size: u32) -> Result<Progeny>`

Get a paginated list of offspring for a specific animal.

**API endpoint:** `GET details/getPageOfProgeny`

**Parameters:**

| Parameter | Type | Validation |
|-----------|------|------------|
| `lpn_id` | `&str` | Must not be empty |
| `page` | `u32` | -- |
| `page_size` | `u32` | Must be > 0 |

**Errors:** `Validation` (if `lpn_id` empty or `page_size == 0`), `NotFound`, `Connection`, `Timeout`, `Api`, `Parse`

```rust
let progeny = client.progeny("430735-0032", 0, 10).await?;
println!("{} total offspring", progeny.total_count);
for animal in &progeny.animals {
    println!("  {}", animal.lpn_id);
}
```

---

#### `search_by_lpn(&self, lpn_id: &str) -> Result<AnimalProfile>`

Get a complete profile for an animal: details, lineage, and progeny fetched concurrently via `tokio::join!`.

**Parameters:**

| Parameter | Type | Validation |
|-----------|------|------------|
| `lpn_id` | `&str` | Must not be empty or whitespace-only |

**Errors:** `Validation` (if empty/whitespace), `NotFound`, `Connection`, `Timeout`, `Api`, `Parse`

```rust
let profile = client.search_by_lpn("430735-0032").await?;
println!("Details: {}", profile.details.lpn_id);
println!("Sire: {:?}", profile.lineage.sire);
println!("Offspring: {}", profile.progeny.total_count);
```

---

## SearchCriteria

Builder for constructing search filters. All builder methods consume and return `self`, allowing method chaining.

### Construction

#### `SearchCriteria::new() -> Self`

Create an empty criteria with all fields set to `None`. This is a `const fn`.

```rust
let criteria = SearchCriteria::new();
```

### Builder Methods

All builder methods consume `self` and return a new `SearchCriteria` with the field set.

| Method | Parameter type | Description |
|--------|---------------|-------------|
| `with_breed_group_id(self, id: i64)` | `i64` | Set breed group ID filter (`const fn`) |
| `with_breed_id(self, id: i64)` | `i64` | Set breed ID filter (`const fn`) |
| `with_born_after(self, date: impl Into<String>)` | `String` | Only animals born after this date (`YYYY-MM-DD`) |
| `with_born_before(self, date: impl Into<String>)` | `String` | Only animals born before this date (`YYYY-MM-DD`) |
| `with_gender(self, gender: impl Into<String>)` | `String` | Gender filter: `"Male"`, `"Female"`, `"Both"` |
| `with_proven_only(self, proven: bool)` | `bool` | Only proven animals (`const fn`) |
| `with_status(self, status: impl Into<String>)` | `String` | Status filter: `"CURRENT"`, `"SOLD"`, `"DEAD"` |
| `with_flock_id(self, flock_id: impl Into<String>)` | `String` | Flock ID filter |
| `with_trait_ranges(self, ranges: HashMap<String, TraitRangeFilter>)` | `HashMap` | Per-trait min/max filters |

### Fields

All fields are `pub` and `Option`-wrapped. The struct derives `Debug`, `Clone`, `Default`, `Serialize`, and `Deserialize`. JSON serialization uses `camelCase` field names and skips `None` values.

| Field | Type | JSON key |
|-------|------|----------|
| `breed_group_id` | `Option<i64>` | `breedGroupId` |
| `breed_id` | `Option<i64>` | `breedId` |
| `born_after` | `Option<String>` | `bornAfter` |
| `born_before` | `Option<String>` | `bornBefore` |
| `gender` | `Option<String>` | `gender` |
| `proven_only` | `Option<bool>` | `provenOnly` |
| `status` | `Option<String>` | `status` |
| `flock_id` | `Option<String>` | `flockId` |
| `trait_ranges` | `Option<HashMap<String, TraitRangeFilter>>` | `traitRanges` |

### Example

```rust
use std::collections::HashMap;
use nsip::{SearchCriteria, TraitRangeFilter};

let criteria = SearchCriteria::new()
    .with_breed_id(486)
    .with_status("CURRENT")
    .with_gender("Female")
    .with_born_after("2020-01-01")
    .with_proven_only(true)
    .with_trait_ranges(HashMap::from([
        ("BWT".to_string(), TraitRangeFilter { min: -1.0, max: 1.0 }),
        ("WWT".to_string(), TraitRangeFilter { min: 5.0, max: 20.0 }),
    ]));
```

---

## Model Types

### AnimalDetails

Detailed information about a single animal including EBV traits, breed, and contact info.

| Field | Type | Description |
|-------|------|-------------|
| `lpn_id` | `String` | Unique LPN identifier |
| `breed` | `Option<String>` | Breed name |
| `breed_group` | `Option<String>` | Breed group name |
| `date_of_birth` | `Option<String>` | Date of birth |
| `gender` | `Option<String>` | `"Male"` or `"Female"` |
| `status` | `Option<String>` | `"CURRENT"`, `"SOLD"`, `"DEAD"`, etc. |
| `sire` | `Option<String>` | Sire LPN identifier |
| `dam` | `Option<String>` | Dam LPN identifier |
| `registration_number` | `Option<String>` | Registration number |
| `total_progeny` | `Option<i64>` | Total number of progeny |
| `flock_count` | `Option<i64>` | Number of flocks |
| `genotyped` | `Option<String>` | Genotyped status |
| `traits` | `HashMap<String, Trait>` | EBV traits keyed by abbreviation (e.g. `"BWT"`, `"WWT"`) |
| `contact_info` | `Option<ContactInfo>` | Owner/flock contact information |

---

### AnimalProfile

Combined profile returned by `search_by_lpn()`.

| Field | Type | Description |
|-------|------|-------------|
| `details` | `AnimalDetails` | Animal details and EBVs |
| `lineage` | `Lineage` | Pedigree / ancestry data |
| `progeny` | `Progeny` | Offspring list |

---

### Breed

A single breed within a breed group.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Breed identifier |
| `name` | `String` | Breed name |

---

### BreedGroup

A group of related breeds.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Group identifier |
| `name` | `String` | Group name |
| `breeds` | `Vec<Breed>` | Breeds in this group |

---

### ContactInfo

Breeder contact information associated with an animal.

| Field | Type | Description |
|-------|------|-------------|
| `farm_name` | `Option<String>` | Farm name |
| `contact_name` | `Option<String>` | Contact person |
| `phone` | `Option<String>` | Phone number |
| `email` | `Option<String>` | Email address |
| `address` | `Option<String>` | Street address |
| `city` | `Option<String>` | City |
| `state` | `Option<String>` | State |
| `zip_code` | `Option<String>` | ZIP code |

---

### DateLastUpdated

Response from the database last-updated endpoint.

| Field | Type | Description |
|-------|------|-------------|
| `data` | `serde_json::Value` | Raw JSON response containing the date |

---

### Lineage

Pedigree / ancestry tree for an animal.

| Field | Type | Description |
|-------|------|-------------|
| `subject` | `Option<LineageAnimal>` | The animal itself |
| `sire` | `Option<LineageAnimal>` | Father |
| `dam` | `Option<LineageAnimal>` | Mother |
| `generations` | `Vec<Vec<LineageAnimal>>` | Extended pedigree by generation depth |

---

### LineageAnimal

A single animal within a pedigree tree.

| Field | Type | Description |
|-------|------|-------------|
| `lpn_id` | `String` | LPN identifier |
| `farm_name` | `Option<String>` | Farm name |
| `us_index` | `Option<f64>` | US selection index |
| `src_index` | `Option<f64>` | Source index |
| `date_of_birth` | `Option<String>` | Date of birth |
| `sex` | `Option<String>` | Sex |
| `status` | `Option<String>` | Status |

---

### Progeny

Paginated list of offspring.

| Field | Type | Description |
|-------|------|-------------|
| `total_count` | `i64` | Total number of offspring |
| `animals` | `Vec<ProgenyAnimal>` | Offspring on this page |
| `page` | `u32` | Current page number |
| `page_size` | `u32` | Page size |

---

### ProgenyAnimal

A single offspring animal.

| Field | Type | Description |
|-------|------|-------------|
| `lpn_id` | `String` | LPN identifier |
| `sex` | `Option<String>` | Sex |
| `date_of_birth` | `Option<String>` | Date of birth |
| `traits` | `HashMap<String, f64>` | Trait abbreviation to EBV value |

---

### SearchResults

Paginated search results.

| Field | Type | Description |
|-------|------|-------------|
| `total_count` | `i64` | Total matching animals |
| `results` | `Vec<serde_json::Value>` | Raw result objects for this page |
| `page` | `u32` | Current page number |
| `page_size` | `u32` | Page size |

---

### Trait

A single EBV trait value for an animal.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Trait abbreviation (e.g., `BWT`, `WWT`) |
| `value` | `f64` | EBV value |
| `accuracy` | `Option<i32>` | Accuracy percentage (0-100) |
| `units` | `Option<String>` | Unit of measurement |

---

### TraitRange

Breed-level minimum and maximum for a trait.

| Field | Type | Description |
|-------|------|-------------|
| `trait_name` | `String` | Trait abbreviation |
| `min_value` | `f64` | Minimum value in the breed |
| `max_value` | `f64` | Maximum value in the breed |
| `unit` | `Option<String>` | Unit of measurement |

---

### TraitRangeFilter

Min/max bounds for filtering animals by trait value in search criteria.

| Field | Type | Description |
|-------|------|-------------|
| `min` | `f64` | Minimum value (inclusive) |
| `max` | `f64` | Maximum value (inclusive) |

---

## mcp Module

The `mcp` module provides a complete Model Context Protocol server implementation with tools, resources, prompts, and analytics for livestock breeding intelligence.

### NsipServer

MCP server implementation that exposes 13 tools, 5 static resources, 4 resource templates, and 7 guided breeding prompts.

```rust
use nsip::mcp::NsipServer;

let server = NsipServer::new();
```

#### `NsipServer::new() -> Self`

Create a new MCP server instance with default NSIP API client.

```rust
use nsip::mcp::NsipServer;

let server = NsipServer::new();
```

#### `serve_stdio() -> Result<()>`

Start the MCP server on stdio transport (used by Claude Desktop, Claude Code, Cursor, etc.).

```rust
use nsip::mcp::serve_stdio;

#[tokio::main]
async fn main() -> nsip::Result<()> {
    serve_stdio().await
}
```

---

### analytics Submodule

Pure computation functions for breeding analytics with no external dependencies.

#### Types

**`CoiRating`** — Traffic-light rating for inbreeding coefficient:
- `Green` — COI < 6.25% (acceptable)
- `Yellow` — 6.25% ≤ COI < 12.5% (elevated, proceed with caution)
- `Red` — COI ≥ 12.5% (high inbreeding, generally avoid)

**`SharedAncestor`** — Common ancestor found in both sire and dam pedigrees.

| Field | Type | Description |
|-------|------|-------------|
| `lpn_id` | `String` | LPN ID of the common ancestor |
| `sire_depth` | `usize` | Generations from sire to this ancestor |
| `dam_depth` | `usize` | Generations from dam to this ancestor |

**`CoiResult`** — Result of coefficient of inbreeding calculation.

| Field | Type | Description |
|-------|------|-------------|
| `coefficient` | `f64` | Wright's coefficient of inbreeding (0.0–1.0) |
| `rating` | `CoiRating` | Traffic-light rating |
| `shared_ancestors` | `Vec<SharedAncestor>` | Common ancestors contributing to inbreeding |

**`RankedAnimal`** — Animal with weighted composite score for trait-based ranking.

| Field | Type | Description |
|-------|------|-------------|
| `lpn_id` | `String` | LPN identifier |
| `score` | `f64` | Weighted composite score |
| `trait_scores` | `HashMap<String, f64>` | Per-trait weighted scores |

**`TraitDefinition`** — EBV trait metadata with interpretation and selection direction.

| Field | Type | Description |
|-------|------|-------------|
| `abbreviation` | `&'static str` | Trait abbreviation (e.g., `"BWT"`) |
| `name` | `&'static str` | Full trait name (e.g., `"Birth Weight"`) |
| `unit` | `&'static str` | Unit of measurement (e.g., `"lbs"`) |
| `description` | `&'static str` | What the trait measures |
| `selection_direction` | `&'static str` | Whether higher or lower is better |

#### Functions

**`calculate_coi(sire_lineage: &Lineage, dam_lineage: &Lineage) -> CoiResult`**

Calculate Wright's coefficient of inbreeding from sire and dam pedigrees.

Formula: `COI = Σ [(0.5)^(n₁ + n₂ + 1)]` where:
- `n₁` = path length from sire to common ancestor
- `n₂` = path length from dam to common ancestor

```rust
use nsip::{NsipClient, mcp::analytics::calculate_coi};

# async fn example() -> nsip::Result<()> {
let client = NsipClient::new();
let sire_lineage = client.lineage("430735-0032").await?;
let dam_lineage = client.lineage("430735-0089").await?;

let coi_result = calculate_coi(&sire_lineage, &dam_lineage);
println!("COI: {:.2}% ({:?})", coi_result.coefficient * 100.0, coi_result.rating);
# Ok(())
# }
```

**`rank_animals(animals: &[AnimalDetails], weights: &HashMap<String, f64>) -> Vec<RankedAnimal>`**

Rank animals by weighted composite of EBV traits.

Score formula: `Σ (trait_value × weight × accuracy/100)` for each trait.

```rust
use std::collections::HashMap;
use nsip::{AnimalDetails, NsipClient, SearchCriteria, mcp::analytics::rank_animals};

# async fn example() -> nsip::Result<()> {
let client = NsipClient::new();
let criteria = SearchCriteria::new()
    .with_breed_id(486)
    .with_gender("Male")
    .with_status("CURRENT");
let search = client.search_animals(0, 50, Some(486), None, None, Some(&criteria)).await?;

let animals: Vec<AnimalDetails> = search.results
    .iter()
    .filter_map(|r| AnimalDetails::from_api_response(r).ok())
    .collect();

let weights = HashMap::from([
    ("BWT".to_string(), -1.0),
    ("WWT".to_string(), 2.0),
    ("YWT".to_string(), 1.5),
]);

let ranked = rank_animals(&animals, &weights);
for animal in ranked.iter().take(5) {
    println!("{}: {:.2}", animal.lpn_id, animal.score);
}
# Ok(())
# }
```

**`trait_complementarity(sire: &AnimalDetails, dam: &AnimalDetails) -> HashMap<String, f64>`**

Predict midparent EBV values for potential offspring.

Formula: `predicted_EBV = (sire_EBV + dam_EBV) / 2.0`

```rust
use nsip::{NsipClient, mcp::analytics::trait_complementarity};

# async fn example() -> nsip::Result<()> {
let client = NsipClient::new();
let sire = client.animal_details("430735-0032").await?;
let dam = client.animal_details("430735-0089").await?;

let midparent_ebvs = trait_complementarity(&sire, &dam);
for (trait_name, value) in midparent_ebvs {
    println!("{}: {:.2}", trait_name, value);
}
# Ok(())
# }
```

**`find_shared_ancestors(sire_lineage: &Lineage, dam_lineage: &Lineage) -> Vec<SharedAncestor>`**

Find ancestors that appear in both sire and dam pedigrees.

```rust
use nsip::{NsipClient, mcp::analytics::find_shared_ancestors};

# async fn example() -> nsip::Result<()> {
let client = NsipClient::new();
let sire_lineage = client.lineage("430735-0032").await?;
let dam_lineage = client.lineage("430735-0089").await?;

let shared = find_shared_ancestors(&sire_lineage, &dam_lineage);
for ancestor in shared {
    println!("{} (sire depth: {}, dam depth: {})",
        ancestor.lpn_id, ancestor.sire_depth, ancestor.dam_depth);
}
# Ok(())
# }
```

**`ebv_glossary() -> Vec<TraitDefinition>`**

Returns complete glossary of all 13 NSIP EBV traits with metadata for each trait.

```rust
use nsip::mcp::analytics::ebv_glossary;

for trait_def in ebv_glossary() {
    println!("{} ({}): {} [{}]",
        trait_def.abbreviation,
        trait_def.name,
        trait_def.description,
        trait_def.selection_direction
    );
}
```

See [EBV Trait Glossary](../MCP.md#ebv-trait-glossary) for the complete trait reference table.

---

### prompts Submodule

Guided breeding workflow prompts that fetch live data and construct structured context for LLM-based breeding advice.

7 prompts available:
- `evaluate-ram` — Evaluate a ram's breeding value (emphasizes growth and carcass traits)
- `evaluate-ewe` — Evaluate a ewe's breeding value (emphasizes maternal traits)
- `compare-breeding-stock` — Compare 2-5 animals side-by-side
- `plan-mating` — Plan a specific mating with COI check and trait complementarity
- `flock-improvement` — Analyze a breed/flock for trait gaps
- `select-replacement` — Find top replacement candidates by gender and target trait
- `interpret-ebvs` — Plain-language EBV explanation for farmers

See [MCP Server Reference](../MCP.md#prompt-reference) for full prompt documentation.

---

### resources Submodule

Static and dynamic resources for MCP clients.

**Static resources:**
- `nsip://glossary` — EBV trait glossary (markdown)
- `nsip://breeds` — Live breed directory (JSON)
- `nsip://guide/selection` — Selection guide (markdown)
- `nsip://guide/inbreeding` — Inbreeding guide (markdown)
- `nsip://status` — Database status (JSON)

**Resource templates:**
- `nsip://animal/{lpn_id}` — Full animal profile
- `nsip://animal/{lpn_id}/pedigree` — Pedigree tree
- `nsip://animal/{lpn_id}/progeny` — Offspring list
- `nsip://breed/{breed_id}/ranges` — Breed trait ranges

See [MCP Server Reference](../MCP.md#resource-reference) for full resource documentation.

---

## See Also

- [MCP Server Reference](../MCP.md) -- complete MCP tools, resources, and prompts documentation
- [Error Handling Reference](ERROR-HANDLING.md) -- error types and recovery strategies
- [Configuration Reference](CONFIGURATION.md) -- client configuration options
- [CLI Reference](CLI.md) -- command-line interface for the same functionality
- [Getting Started](../tutorials/GETTING-STARTED.md)
