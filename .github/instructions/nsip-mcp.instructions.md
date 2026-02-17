---
applyTo: "**/*-record.yml,**/BREEDING-EVENT-LOG*,reports/**"
---

# NSIP MCP Agent Instructions

When working with breeding record issues, event logs, or livestock data queries, use
the NSIP MCP tools described below. This document is the authoritative reference for
agent-driven interactions with the NSIP Search API.

---

## What is the NSIP MCP Server?

The NSIP MCP server wraps the [National Sheep Improvement Program](https://nsip.org/)
Search API as a Model Context Protocol service. It gives AI agents direct access to:

- **400,000+ sheep** with estimated breeding values (EBVs)
- **Pedigree trees** spanning multiple generations
- **Breed-level benchmarks** (trait ranges, averages)
- **Breeding analytics** — inbreeding coefficients, trait ranking, mating recommendations

**Transport**: stdio only — started with `nsip mcp`

---

## Tool Reference

### search

Find animals matching filter criteria. Returns paginated results.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `breed_group_id` | integer | no | | Breed group ID |
| `breed_id` | integer | no | | Breed ID |
| `status` | string | no | | `"CURRENT"`, `"SOLD"`, or `"DEAD"` |
| `gender` | string | no | | `"Male"`, `"Female"`, or `"Both"` |
| `born_after` | string | no | | Animals born after this date (`YYYY-MM-DD`) |
| `born_before` | string | no | | Animals born before this date (`YYYY-MM-DD`) |
| `proven_only` | boolean | no | false | Only return proven animals |
| `flock_id` | string | no | | Flock ID |
| `sort_by` | string | no | | Trait abbreviation to sort by (e.g. `"WWT"`) |
| `reverse` | boolean | no | false | Reverse the sort order |
| `page` | integer | no | 0 | Page number (0-indexed) |
| `page_size` | integer | no | 15 | Results per page (1-100) |

---

### details

Fetch full EBV data, breed, contact info, and status for one animal.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `animal_id` | string | yes | LPN ID or registration number |

---

### lineage

Retrieve the pedigree (ancestry) tree — parents, grandparents, and deeper ancestors.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `animal_id` | string | yes | LPN ID |

---

### progeny

List offspring for an animal with pagination.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `animal_id` | string | yes | | LPN ID |
| `page` | integer | no | 0 | Page number (0-indexed) |
| `page_size` | integer | no | 10 | Results per page |

---

### profile

All-in-one call that combines `details`, `lineage`, and `progeny` in a single request.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `animal_id` | string | yes | LPN ID |

---

### breed_groups

List all breed groups and individual breeds in the NSIP database. No parameters.

---

### trait_ranges

Get the min/max EBV values across all animals within a breed.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `breed_id` | integer | yes | Breed ID |

---

### compare

Side-by-side EBV comparison of 2-5 animals. Optionally filter to specific traits.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `animal_ids` | array of strings | yes | LPN IDs (2-5 items) |
| `traits` | string | no | Comma-separated trait filter (e.g. `"BWT,WWT,YWT"`) |

---

### rank

Rank animals within a breed by a weighted composite score.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `breed_id` | integer | yes | | Breed ID to search |
| `weights` | object | yes | | Trait-to-weight map, e.g. `{"BWT": -1.0, "WWT": 2.0}` |
| `gender` | string | no | | `"Male"`, `"Female"`, or `"Both"` |
| `status` | string | no | | `"CURRENT"`, `"SOLD"`, `"DEAD"` |
| `top_n` | integer | no | 10 | Number of top results to return |

---

### inbreeding_check

Calculate Wright's coefficient of inbreeding (COI) for a potential sire x dam mating.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `sire_id` | string | yes | LPN ID of the sire (father) |
| `dam_id` | string | yes | LPN ID of the dam (mother) |

**COI traffic-light thresholds:**

| Rating | COI | Recommendation |
|---|---|---|
| Green | < 6.25% | Safe — proceed |
| Yellow | 6.25-12.5% | Caution — consider alternatives |
| Red | > 12.5% | Avoid — high inbreeding depression risk |

---

### mating_recommendations

Find optimal mates for an animal.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `animal_id` | string | yes | | LPN ID of the animal to find mates for |
| `breed_id` | integer | yes | | Breed ID to search for candidates |
| `target_traits` | string | no | WWT, BWT, NLB | Comma-separated traits to optimize |
| `max_results` | integer | no | 5 | Number of recommendations |

---

### flock_summary

Summarize a flock: animal count, gender breakdown, and average EBV traits.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `flock_id` | string | yes | Flock ID |
| `breed_id` | integer | no | Filter to a specific breed within the flock |

---

### database_status

Check when the NSIP database was last updated. No parameters.

---

## EBV Trait Glossary

| Abbreviation | Full Name | Unit | Selection |
|---|---|---|---|
| BWT | Birth Weight | lbs | Lower preferred |
| WWT | Weaning Weight | lbs | Higher preferred |
| PWWT | Post-Weaning Weight | lbs | Higher preferred |
| YWT | Yearling Weight | lbs | Higher preferred |
| FAT | Fat Depth | mm | Moderate preferred |
| EMD | Eye Muscle Depth | mm | Higher preferred |
| NLB | Number of Lambs Born | lambs | Higher (with caution) |
| NWT | Number of Lambs Weaned | lambs | Higher preferred |
| PWT | Pounds Weaned | lbs | Higher preferred |
| DAG | Dag Score | score | Lower preferred |
| WEC | Worm Egg Count | eggs/g | Lower preferred |
| FEC | Faecal Egg Count | eggs/g | Lower preferred |

## Formatting Rules

1. **Always use tables** for EBV data — never dump raw JSON.
2. **Show accuracy** alongside every EBV value: `+9.6 (68%)`.
3. **Use the traffic light** for COI results (Green/Yellow/Red).
4. **Include breed context** whenever possible via `trait_ranges`.
5. **Caveat low accuracy**: Note if a key trait has accuracy below 40%.
6. **Use the trait's natural units**: lbs for weights, mm for depth/fat, lambs for NLB/NWT.
