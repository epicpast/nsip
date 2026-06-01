# NSIP MCP Server -- Claude Code / Claude Desktop

The NSIP MCP server exposes the full [NSIP Search](https://nsipsearch.nsip.org/) API -- plus analytics-powered breeding intelligence -- to Claude via the Model Context Protocol. It provides 13 tools, 5 resources, 4 resource templates, and 7 guided prompts for sheep genetic evaluation.

Full reference: [`docs/MCP.md`](../MCP.md)

## Configuration

Place in `.mcp.json` at your project root (Claude Code) or `claude_desktop_config.json` (Claude Desktop):

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

## Resources

Read these URIs directly for reference data:

| URI | Description |
|---|---|
| `nsip://glossary` | EBV trait definitions, units, selection direction |
| `nsip://breeds` | Live breed directory (JSON) |
| `nsip://guide/selection` | How to use EBVs for breeding decisions |
| `nsip://guide/inbreeding` | COI thresholds and avoidance strategies |
| `nsip://status` | Database last-updated date |

Resource templates for dynamic data:

- `nsip://animal/{lpn_id}` -- Full animal profile
- `nsip://animal/{lpn_id}/pedigree` -- Pedigree tree
- `nsip://animal/{lpn_id}/progeny` -- Offspring list
- `nsip://breed/{breed_id}/ranges` -- Breed trait ranges

## Guided Prompts

Use these MCP prompts for structured breeding workflows:

| Prompt | Purpose | Arguments |
|---|---|---|
| `evaluate-ram` | Assess a ram's breeding value | `lpn_id` |
| `evaluate-ewe` | Assess a ewe's breeding value | `lpn_id` |
| `compare-breeding-stock` | Side-by-side animal comparison | `lpn_ids` (comma-separated) |
| `plan-mating` | Mating assessment with COI check | `sire_id`, `dam_id` |
| `flock-improvement` | Trait gap analysis | `breed_id`, `flock_id` (optional) |
| `select-replacement` | Find top replacement candidates | `breed_id`, `gender`, `target_trait` |
| `interpret-ebvs` | Farmer-friendly EBV explanation | `lpn_id` |

## Common Workflows

### 1. Evaluate an Animal

1. Call `details` with the animal's LPN ID
2. Call `trait_ranges` with the animal's `breed_id` to get breed context
3. Call `lineage` to review pedigree
4. Or use the `evaluate-ram` / `evaluate-ewe` prompt for a guided assessment

### 2. Plan a Mating

1. Call `inbreeding_check` with `sire_id` and `dam_id` -- abort if rating is Red
2. Call `details` for both sire and dam to review EBVs
3. Call `mating_recommendations` for the animal to see ranked alternatives
4. Or use the `plan-mating` prompt which does all of this in one step

### 3. Rank a Flock or Breed

1. Call `breed_groups` if the breed ID is unknown
2. Call `rank` with trait weights (negative for lower-is-better traits)
3. Call `compare` on the top candidates for detailed side-by-side review

### 4. Flock Improvement

1. Call `flock_summary` to see current averages
2. Call `trait_ranges` for breed-level benchmarks
3. Call `rank` to find improvement sires with desired trait profile
4. Or use the `flock-improvement` prompt for a guided analysis

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
