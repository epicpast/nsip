---
diataxis_type: reference
---
# MCP Tools Reference

Complete reference for the 13 tools exposed by the `nsip mcp` server.

For installation, configuration, resources, and prompts, see [MCP Server Reference](../MCP.md).

---

## Overview

The MCP server exposes 13 tools over the Model Context Protocol (stdio transport, protocol version `2025-11-25`). All tools return JSON results as text content. Errors use standard MCP error codes.

| Tool | Description |
|------|-------------|
| [search](#search) | Search for animals with filters and sorting |
| [details](#details) | Get detailed EBV data for an animal |
| [lineage](#lineage) | Get pedigree / ancestry tree |
| [progeny](#progeny) | Get offspring list (paginated) |
| [profile](#profile) | Get complete animal profile in one call |
| [breed_groups](#breed_groups) | List all breed groups and breeds |
| [trait_ranges](#trait_ranges) | Get min/max EBV ranges for a breed |
| [compare](#compare) | Compare 2-5 animals side-by-side |
| [rank](#rank) | Rank animals by weighted EBV traits |
| [inbreeding_check](#inbreeding_check) | Calculate Wright's COI for a mating pair |
| [mating_recommendations](#mating_recommendations) | Find optimal mates for an animal |
| [flock_summary](#flock_summary) | Summarize a flock's animals and trait averages |
| [database_status](#database_status) | Get database last-updated date and statuses |

---

## search

Search for animals in the NSIP database with filters for breed, gender, status, date range, flock, and sorting.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `breed_group_id` | integer | no | -- | Breed group ID to filter by |
| `breed_id` | integer | no | -- | Breed ID to filter by |
| `status` | string | no | -- | `"CURRENT"`, `"SOLD"`, or `"DEAD"` |
| `gender` | string | no | -- | `"Male"`, `"Female"`, or `"Both"` |
| `born_after` | string | no | -- | Only animals born after this date (`YYYY-MM-DD`) |
| `born_before` | string | no | -- | Only animals born before this date (`YYYY-MM-DD`) |
| `proven_only` | boolean | no | false | Only return proven animals |
| `flock_id` | string | no | -- | Flock ID to filter by |
| `sort_by` | string | no | -- | Trait abbreviation to sort by (e.g., `"WWT"`) |
| `reverse` | boolean | no | -- | Reverse the sort order |
| `page` | integer | no | 0 | Page number (0-indexed) |
| `page_size` | integer | no | 15 | Results per page (1-100) |

**Returns:** `SearchResults` -- total count, result objects, page, and page size.

**Example:**

```json
{
  "tool": "search",
  "arguments": {
    "breed_id": 486,
    "gender": "Male",
    "status": "CURRENT",
    "sort_by": "WWT",
    "page_size": 10
  }
}
```

---

## details

Get detailed EBV data, breed, contact info, and status for a specific animal by LPN ID.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `lpn_id` | string | yes | LPN ID or registration number |

**Returns:** `AnimalDetails` -- LPN ID, breed, sex, date of birth, status, EBV traits, and contact info.

**Example:**

```json
{
  "tool": "details",
  "arguments": {
    "lpn_id": "430735-0032"
  }
}
```

---

## lineage

Get pedigree / ancestry tree for a specific animal including parents and grandparents.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `lpn_id` | string | yes | LPN ID of the animal |

**Returns:** `Lineage` -- subject, sire, dam, and extended generations.

**Example:**

```json
{
  "tool": "lineage",
  "arguments": {
    "lpn_id": "430735-0032"
  }
}
```

---

## progeny

Get a paginated list of offspring for a specific animal.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `lpn_id` | string | yes | -- | LPN ID of the animal |
| `page` | integer | no | 0 | Page number (0-indexed) |
| `page_size` | integer | no | 10 | Results per page |

**Returns:** `Progeny` -- total count, offspring animals with their traits, page, and page size.

**Example:**

```json
{
  "tool": "progeny",
  "arguments": {
    "lpn_id": "430735-0032",
    "page": 0,
    "page_size": 20
  }
}
```

---

## profile

Get a complete profile for an animal: details, pedigree, and offspring in one call.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `lpn_id` | string | yes | LPN ID of the animal |

**Returns:** `AnimalProfile` -- combined details, lineage, and progeny.

**Example:**

```json
{
  "tool": "profile",
  "arguments": {
    "lpn_id": "430735-0032"
  }
}
```

---

## breed_groups

List all breed groups and individual breeds in the NSIP database.

**Parameters:** None.

**Returns:** Array of `BreedGroup` objects, each containing `id`, `name`, and a `breeds` array.

**Example:**

```json
{
  "tool": "breed_groups",
  "arguments": {}
}
```

---

## trait_ranges

Get the minimum and maximum EBV trait ranges for a specific breed. Useful for understanding breed norms and setting trait filters.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `breed_id` | integer | yes | Breed ID to query |

**Returns:** JSON object with per-trait min/max values.

**Example:**

```json
{
  "tool": "trait_ranges",
  "arguments": {
    "breed_id": 486
  }
}
```

---

## compare

Compare 2-5 animals side-by-side on their EBV traits. Optionally filter to specific traits.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `lpn_ids` | array of strings | yes | LPN IDs to compare (2-5 items) |
| `traits` | string | no | Comma-separated trait filter (e.g., `"BWT,WWT,YWT"`) |

**Returns:** Array of `AnimalDetails` objects for the requested animals.

**Example:**

```json
{
  "tool": "compare",
  "arguments": {
    "lpn_ids": ["430735-0032", "430735-0041", "430735-0058"],
    "traits": "BWT,WWT,YWT,EMD"
  }
}
```

---

## rank

Rank animals within a breed by weighted EBV traits. Specify trait weights to prioritize breeding goals.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `breed_id` | integer | yes | -- | Breed ID to search within |
| `weights` | object | yes | -- | Trait weights as `{"TRAIT": weight}` (e.g., `{"BWT": -1.0, "WWT": 2.0}`) |
| `gender` | string | no | -- | `"Male"`, `"Female"`, or `"Both"` |
| `status` | string | no | -- | Animal status filter (e.g., `"CURRENT"`) |
| `top_n` | integer | no | 10 | Number of top-ranked results to return |

**Ranking formula:** `Score = sum(trait_value * weight * accuracy / 100)` for each trait where both a weight and value exist.

**Returns:** Ranked list of animals with their composite scores and individual trait values.

**Example -- terminal sire selection:**

```json
{
  "tool": "rank",
  "arguments": {
    "breed_id": 486,
    "weights": {
      "BWT": -1.0,
      "WWT": 2.0,
      "YWT": 1.5,
      "EMD": 1.0
    },
    "gender": "Male",
    "status": "CURRENT",
    "top_n": 5
  }
}
```

**Example -- maternal sire selection:**

```json
{
  "tool": "rank",
  "arguments": {
    "breed_id": 486,
    "weights": {
      "NLB": 2.0,
      "NWT": 2.0,
      "PWT": 1.5,
      "BWT": -0.5
    },
    "gender": "Male",
    "top_n": 10
  }
}
```

---

## inbreeding_check

Calculate Wright's coefficient of inbreeding (COI) for a potential sire-dam mating. Returns the COI value, a traffic-light rating, and shared ancestors.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sire_id` | string | yes | LPN ID of the sire |
| `dam_id` | string | yes | LPN ID of the dam |

**COI formula:** `COI = sum[(0.5)^(n1 + n2 + 1)]` where `n1` and `n2` are path lengths from sire and dam to each common ancestor.

**Traffic-light thresholds:**

| Rating | COI range | Interpretation |
|--------|-----------|----------------|
| Green | < 6.25% | Acceptable -- proceed with mating |
| Yellow | 6.25% to < 12.5% | Elevated -- consider alternatives |
| Red | >= 12.5% | High -- generally avoid |

**Returns:** COI coefficient, rating, and list of shared ancestors with path depths.

**Example:**

```json
{
  "tool": "inbreeding_check",
  "arguments": {
    "sire_id": "430735-0032",
    "dam_id": "430735-0089"
  }
}
```

**Example response:**

```json
{
  "coefficient": 0.03125,
  "rating": "Green",
  "shared_ancestors": [
    {
      "lpn_id": "410220-0015",
      "sire_depth": 2,
      "dam_depth": 2
    }
  ]
}
```

---

## mating_recommendations

Find optimal mates for an animal. Searches the breed for candidates, checks inbreeding, and ranks by trait complementarity.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `lpn_id` | string | yes | -- | LPN ID of the animal to find mates for |
| `breed_id` | integer | yes | -- | Breed ID to search for potential mates |
| `target_traits` | string | no | `WWT,BWT,NLB` | Traits to optimize (comma-separated) |
| `max_results` | integer | no | 5 | Maximum number of recommendations |

**Default trait weights (when `target_traits` is omitted):**
- WWT: 1.0
- BWT: -0.5
- NLB: 0.5

Traits where lower values are preferred (`BWT`, `DAG`, `WEC`, `FEC`) automatically receive negative weights.

**Offspring EBV prediction:** `predicted_offspring_EBV = (sire_EBV + dam_EBV) / 2`

**Returns:** Ranked list of recommended mates, each with a score, COI check, and predicted offspring EBVs.

Each `coi` object includes a `reliable` boolean. It is `true` when the mate's lineage was fetched successfully and the COI reflects real pedigree overlap. It is `false` when the mate's lineage could not be retrieved; in that case the COI is computed against an empty pedigree (typically `0.0` / `Green`) and should not be trusted.

**Example:**

```json
{
  "tool": "mating_recommendations",
  "arguments": {
    "lpn_id": "430735-0032",
    "breed_id": 486,
    "target_traits": "WWT,EMD,NLB",
    "max_results": 3
  }
}
```

**Example response:**

```json
[
  {
    "mate_lpn_id": "430735-0089",
    "rank_score": 18.42,
    "coi": {
      "coefficient": 0.015,
      "rating": "Green",
      "reliable": true
    },
    "predicted_offspring_ebvs": {
      "BWT": 0.15,
      "WWT": 11.3,
      "EMD": 1.8,
      "NLB": 0.12
    }
  }
]
```

---

## flock_summary

Summarize a flock's animals: count, gender breakdown, and average EBV traits.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `flock_id` | string | yes | Flock ID to summarize |
| `breed_id` | integer | no | Breed ID to filter within the flock |

**Returns:** Flock summary with total count, sample size, male/female counts, and trait averages.

**Example:**

```json
{
  "tool": "flock_summary",
  "arguments": {
    "flock_id": "430735",
    "breed_id": 486
  }
}
```

**Example response:**

```json
{
  "flock_id": "430735",
  "total_count": 87,
  "sample_size": 87,
  "males": 12,
  "females": 75,
  "trait_averages": {
    "BWT": 0.32,
    "WWT": 8.45,
    "YWT": 12.10,
    "NLB": 0.08,
    "EMD": 0.95
  }
}
```

---

## database_status

Get the NSIP database last-updated date and available animal statuses.

**Parameters:** None.

**Returns:** Database status object with last-updated date and available statuses.

**Example:**

```json
{
  "tool": "database_status",
  "arguments": {}
}
```

---

## EBV Trait Abbreviations

These abbreviations are used in `sort_by`, `traits`, `weights`, and `target_traits` parameters:

| Abbreviation | Name | Unit | Selection Direction |
|--------------|------|------|---------------------|
| BWT | Birth Weight | lbs | Lower preferred |
| WWT | Weaning Weight | lbs | Higher preferred |
| PWWT | Post-Weaning Weight | lbs | Higher preferred |
| YWT | Yearling Weight | lbs | Higher preferred |
| FAT | Fat Depth | mm | Moderate preferred |
| EMD | Eye Muscle Depth | mm | Higher preferred |
| NLB | Number of Lambs Born | lambs | Higher preferred |
| NWT | Number of Lambs Weaned | lambs | Higher preferred |
| PWT | Pounds Weaned | lbs | Higher preferred |
| DAG | Dag Score | score | Lower preferred |
| WGR | Wool Growth Rate | g/day | Higher preferred |
| WEC | Worm Egg Count | eggs/g | Lower preferred |
| FEC | Fecal Egg Count | eggs/g | Lower preferred |

---

## See Also

- [MCP Server Reference](../MCP.md) -- installation, configuration, resources, and prompts
- [CLI Reference](CLI.md) -- command-line interface
- [Library API Reference](LIBRARY-API.md) -- Rust library API
- [Configuration Reference](CONFIGURATION.md) -- environment and client settings
