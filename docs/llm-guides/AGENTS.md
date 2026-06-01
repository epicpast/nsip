# NSIP MCP Server -- Agent Instructions

The NSIP MCP server provides 13 tools for querying the National Sheep Improvement Program database, including animal search, EBV comparison, inbreeding analysis, and mating recommendations.

Full reference: [`docs/MCP.md`](../MCP.md)

## Configuration

Add to your MCP configuration (`.mcp.json` or equivalent):

```json
{
  "mcpServers": {
    "nsip": {
      "command": "nsip",
      "args": ["mcp"]
    }
  }
}
```

## Tool Quick-Reference

| Tool | Purpose | Key Parameters |
|---|---|---|
| `search` | Find animals with filters | `breed_id`, `gender`, `status`, `sort_by` |
| `details` | Full EBV data for one animal | `animal_id` |
| `lineage` | Pedigree / ancestry tree | `animal_id` |
| `progeny` | Offspring list | `animal_id`, `page`, `page_size` |
| `profile` | Combined details+lineage+progeny | `animal_id` |
| `breed_groups` | List all breeds | _(none)_ |
| `trait_ranges` | Min/max EBVs for a breed | `breed_id` |
| `compare` | Side-by-side EBV comparison | `lpn_ids` (2-5), `traits` |
| `rank` | Weighted multi-trait ranking | `breed_id`, `weights`, `gender`, `top_n` |
| `inbreeding_check` | COI for a potential mating | `sire_id`, `dam_id` |
| `mating_recommendations` | Find optimal mates | `animal_id`, `breed_id`, `target_traits` |
| `flock_summary` | Flock-level statistics | `flock_id`, `breed_id` |
| `database_status` | DB freshness and statuses | _(none)_ |

## Common Workflows

### 1. Evaluate an Animal

1. Call `details` with the animal's LPN ID to get EBV data
2. Call `trait_ranges` with the animal's `breed_id` for breed context
3. Call `lineage` to review ancestry and pedigree depth

### 2. Plan a Mating

1. Call `inbreeding_check` with `sire_id` and `dam_id`
2. If rating is `Red` (COI > 12.5%), advise against the mating and stop
3. Call `details` for both sire and dam to compare EBVs
4. Call `mating_recommendations` to see ranked alternative mates

### 3. Rank a Flock or Breed

1. Call `breed_groups` to discover the breed ID (if unknown)
2. Call `rank` with trait weights -- use negative weights for lower-is-better traits (BWT, DAG, WEC, FEC)
3. Call `compare` on the top-ranked candidates for a side-by-side view

### 4. Flock Improvement

1. Call `flock_summary` with the flock ID to see current trait averages
2. Call `trait_ranges` with the breed ID for breed-level benchmarks
3. Call `rank` to find sires that improve weak traits

## Data Conventions

- **LPN IDs**: String identifiers (e.g., `6401492025FLE029`, `430735-0032`)
- **Breed IDs**: Numeric -- use `breed_groups` to discover valid IDs
- **13 EBV traits**: BWT, WWT, PWWT, YWT, FAT, EMD, NLB, NWT, PWT, DAG, WGR, WEC, FEC
- **Lower-is-better traits**: BWT, DAG, WEC, FEC -- use negative weights in `rank`
- **Status values**: `CURRENT`, `SOLD`, `DEAD`
- **Gender values**: `Male`, `Female`, `Both`

## Error Handling

- **Not found**: Tools return an error when an animal ID doesn't exist -- verify IDs with `search` first
- **Pagination**: `search` and `progeny` are paginated (0-indexed pages, default 15 and 10 results)
- **Breed ID required**: `rank`, `trait_ranges`, and `mating_recommendations` require a valid `breed_id`
- **COI rating**: `Green` (<6.25%), `Yellow` (6.25-12.5%), `Red` (>12.5%) -- Red means avoid the mating
