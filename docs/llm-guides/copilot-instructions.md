# NSIP MCP Server -- Copilot Instructions

The NSIP MCP server provides sheep genetic evaluation tools via Model Context Protocol (stdio transport). Configure in `.vscode/mcp.json`:

```json
{
  "servers": {
    "nsip": {
      "command": "nsip",
      "args": ["mcp"]
    }
  }
}
```

Full reference: `docs/MCP.md`

## Tools

- `search` -- Find animals (params: `breed_id`, `gender`, `status`, `sort_by`, `page`, `page_size`)
- `details` -- Full EBV data (params: `animal_id`)
- `lineage` -- Pedigree tree (params: `animal_id`)
- `progeny` -- Offspring list (params: `animal_id`, `page`, `page_size`)
- `profile` -- Details+lineage+progeny combined (params: `animal_id`)
- `breed_groups` -- List all breeds (no params)
- `trait_ranges` -- Min/max EBVs for a breed (params: `breed_id`)
- `compare` -- Side-by-side comparison (params: `animal_ids` 2-5, `traits`)
- `rank` -- Weighted multi-trait ranking (params: `breed_id`, `weights`, `gender`, `top_n`)
- `inbreeding_check` -- COI calculation (params: `sire_id`, `dam_id`)
- `mating_recommendations` -- Optimal mates (params: `animal_id`, `breed_id`, `target_traits`)
- `flock_summary` -- Flock statistics (params: `flock_id`, `breed_id`)
- `database_status` -- DB freshness (no params)

## Workflows

- **Evaluate animal**: `details` -> `trait_ranges` (breed context) -> `lineage`
- **Plan mating**: `inbreeding_check` (abort if Red) -> `details` both -> `mating_recommendations`
- **Rank flock**: `breed_groups` (if breed unknown) -> `rank` -> `compare` top candidates
- **Flock improvement**: `flock_summary` -> `trait_ranges` -> `rank` for improvement sires

## Data Conventions

- LPN IDs: strings (e.g., `430735-0032`)
- Breed IDs: numeric (discover via `breed_groups`)
- 13 EBV traits: BWT, WWT, PWWT, YWT, FAT, EMD, NLB, NWT, PWT, DAG, WGR, WEC, FEC
- Lower-is-better: BWT, DAG, WEC, FEC (use negative weights in `rank`)
- Status: `CURRENT`, `SOLD`, `DEAD`
- Gender: `Male`, `Female`, `Both`
- COI thresholds: Green (<6.25%), Yellow (6.25-12.5%), Red (>12.5%)
- Pagination: 0-indexed pages
