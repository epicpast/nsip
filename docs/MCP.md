# NSIP MCP Server Reference

The `nsip` binary ships a built-in [Model Context Protocol](https://modelcontextprotocol.io/) (MCP) server that exposes the full NSIP Search API surface -- plus analytics-powered breeding intelligence -- to any MCP-compatible client (Claude Desktop, Claude Code, Cursor, etc.).

**Capabilities:** 13 tools, 5 static resources, 4 resource templates, 7 guided prompts, elicitation, cursor-based pagination
**Protocol version:** `2025-06-18` (MCP LATEST)
**Transports:** stdio, streamable HTTP (SSE)

---

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Tool Reference](#tool-reference)
- [Resource Reference](#resource-reference)
- [Prompt Reference](#prompt-reference)
- [Analytics Reference](#analytics-reference)
- [EBV Trait Glossary](#ebv-trait-glossary)

---

## Installation

### From crates.io

```bash
cargo install nsip
```

### From source

```bash
git clone https://github.com/zircote/nsip.git
cd nsip
cargo install --path .
```

### Binary download

Pre-built binaries are available from [GitHub Releases](https://github.com/zircote/nsip/releases):

| Platform       | Binary                    |
|----------------|---------------------------|
| Linux x86_64   | `nsip-linux-amd64`        |
| Linux ARM64    | `nsip-linux-arm64`        |
| macOS x86_64   | `nsip-macos-amd64`        |
| macOS ARM64    | `nsip-macos-arm64`        |
| Windows x86_64 | `nsip-windows-amd64.exe`  |

### MCP Bundle (Claude Desktop / Claude Code)

Download the universal `nsip.mcpb` bundle from [GitHub Releases](https://github.com/zircote/nsip/releases). It contains all platforms in a single file.

**Claude Desktop:** Drag `nsip.mcpb` into Settings > Extensions, or double-click it.

**Claude Code:** Install via the extensions UI.

The bundle auto-selects the correct binary for your platform (macOS ARM64, Linux x86_64, Windows x86_64).

### Docker

```bash
docker pull ghcr.io/zircote/nsip
docker run --rm -i ghcr.io/zircote/nsip mcp
```

The Docker image uses a distroless base for minimal attack surface.

---

## Configuration

### Claude Code (`.mcp.json`)

Place at your project root or `~/.mcp.json`:

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

### Claude Desktop (`claude_desktop_config.json`)

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

On macOS the config file is at `~/Library/Application Support/Claude/claude_desktop_config.json`.

### HTTP Transport

For network-accessible deployments, use the streamable HTTP transport:

```bash
nsip mcp --transport http --host 127.0.0.1 --port 8080
```

This serves a single MCP endpoint at `/mcp` supporting JSON-RPC over POST and SSE via GET, with session management (`Mcp-Session-Id` header) and CORS headers.

> **Security note:** The HTTP endpoint binds to `127.0.0.1` (localhost) by default and restricts CORS to localhost origins. Do **not** expose it on a public interface without protection. For remote access, place the server behind an authenticated reverse proxy or enable `--auth` for OAuth bearer token authentication. See [OAuth Authentication](how-to/OAUTH-AUTHENTICATION.md) and [MCP Security](explanation/MCP-SECURITY.md).

### Docker transport

```json
{
  "mcpServers": {
    "nsip": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "ghcr.io/zircote/nsip", "mcp"]
    }
  }
}
```

### Verification

After configuring, verify the server starts:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | nsip mcp
```

### Tool Sets

By default all 13 tools are exposed. Use `--tools` to enable only specific sets:

```bash
nsip mcp --tools search,breed
```

| Set | Tools |
|-----|-------|
| `search` | `search`, `details`, `lineage`, `progeny`, `profile` |
| `analytics` | `compare`, `rank`, `inbreeding_check`, `mating_recommendations` |
| `flock` | `flock_summary`, `database_status` |
| `breed` | `breed_groups`, `trait_ranges` |

Server instructions are dynamically generated to match the enabled tool sets.

### Authentication

The HTTP transport supports optional OAuth 2.1 + PKCE authentication with GitHub as the identity provider. Enable with `--auth`:

```bash
export NSIP_GITHUB_CLIENT_ID="..."
export NSIP_GITHUB_CLIENT_SECRET="..."
export NSIP_AUTH_SECRET="..."        # HMAC-SHA256 secret for JWT signing
export NSIP_AUTH_BASE_URL="http://localhost:8080"
nsip mcp --transport http --port 8080 --auth
```

When enabled, the server exposes OAuth protocol endpoints (`/register`, `/authorize`, `/callback`, `/token`, `/.well-known/oauth-authorization-server`, `/.well-known/oauth-protected-resource`) and requires a `Bearer` token on the `/mcp` endpoint.

**GitHub PAT shortcut:** Clients can bypass the OAuth flow by passing a GitHub Personal Access Token directly as `Authorization: Bearer ghp_...`. The server validates the PAT via the GitHub API and caches the result for 5 minutes.

Authentication is ignored for stdio transport.

### Telemetry

When compiled with `--features telemetry`, the server uses a custom JSON log format that includes W3C trace context (`trace_id`, `span_id`) on every log line:

```bash
cargo install nsip --features telemetry
nsip mcp --transport http --port 8080
```

The telemetry feature adds an `OpenTelemetry` tracing layer. An OTLP exporter can be layered on by extending the tracer provider configuration.

Without the `telemetry` feature (the default), standard text-based tracing to stderr is used.

---

## Tool Reference

All tools return JSON results as text content. Errors use standard MCP error codes.

### search

Search for animals in the NSIP database with filters for breed, gender, status, date range, flock, and sorting.

| Parameter        | Type     | Required | Default | Description                                       |
|------------------|----------|----------|---------|---------------------------------------------------|
| `breed_group_id` | integer  | no       |         | Breed group ID to filter by                       |
| `breed_id`       | integer  | no       |         | Breed ID to filter by                             |
| `status`         | string   | no       |         | `"CURRENT"`, `"SOLD"`, or `"DEAD"`                |
| `gender`         | string   | no       |         | `"Male"`, `"Female"`, or `"Both"`                 |
| `born_after`     | string   | no       |         | Only animals born after this date (`YYYY-MM-DD`)  |
| `born_before`    | string   | no       |         | Only animals born before this date (`YYYY-MM-DD`) |
| `proven_only`    | boolean  | no       | false   | Only return proven animals                        |
| `flock_id`       | string   | no       |         | Flock ID to filter by                             |
| `sort_by`        | string   | no       |         | Trait abbreviation to sort by (e.g. `"WWT"`)      |
| `reverse`        | boolean  | no       |         | Reverse the sort order                            |
| `page`           | integer  | no       | 0       | Page number (0-indexed)                           |
| `page_size`      | integer  | no       | 15      | Results per page (1-100)                          |

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

### details

Get detailed EBV data, breed, contact info, and status for a specific animal by LPN ID.

| Parameter   | Type   | Required | Description                              |
|-------------|--------|----------|------------------------------------------|
| `animal_id` | string | yes      | LPN ID or registration number            |

**Example:**

```json
{
  "tool": "details",
  "arguments": {
    "animal_id": "430735-0032"
  }
}
```

---

### lineage

Get pedigree / ancestry tree for a specific animal including parents and grandparents.

| Parameter   | Type   | Required | Description                   |
|-------------|--------|----------|-------------------------------|
| `animal_id` | string | yes      | LPN ID of the animal          |

**Example:**

```json
{
  "tool": "lineage",
  "arguments": {
    "animal_id": "430735-0032"
  }
}
```

---

### progeny

Get paginated list of offspring for a specific animal.

| Parameter   | Type    | Required | Default | Description                   |
|-------------|---------|----------|---------|-------------------------------|
| `animal_id` | string  | yes      |         | LPN ID of the animal          |
| `page`      | integer | no       | 0       | Page number (0-indexed)       |
| `page_size` | integer | no       | 10      | Results per page              |

**Example:**

```json
{
  "tool": "progeny",
  "arguments": {
    "animal_id": "430735-0032",
    "page": 0,
    "page_size": 20
  }
}
```

---

### profile

Get complete profile for an animal: details, pedigree, and offspring in one call.

| Parameter   | Type   | Required | Description                   |
|-------------|--------|----------|-------------------------------|
| `animal_id` | string | yes      | LPN ID of the animal          |

**Example:**

```json
{
  "tool": "profile",
  "arguments": {
    "animal_id": "430735-0032"
  }
}
```

---

### breed_groups

List all breed groups and individual breeds in the NSIP database.

No parameters.

**Example:**

```json
{
  "tool": "breed_groups",
  "arguments": {}
}
```

---

### trait_ranges

Get min/max EBV trait ranges for a specific breed -- useful for understanding breed norms.

| Parameter  | Type    | Required | Description        |
|------------|---------|----------|--------------------|
| `breed_id` | integer | yes      | Breed ID to query  |

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

### compare

Compare 2-5 animals side-by-side on their EBV traits. Optionally filter to specific traits.

| Parameter    | Type            | Required | Description                                           |
|--------------|-----------------|----------|-------------------------------------------------------|
| `animal_ids` | array\<string\> | yes      | LPN IDs to compare (2-5 items)                        |
| `traits`     | string          | no       | Comma-separated trait filter (e.g. `"BWT,WWT,YWT"`)   |

**Example:**

```json
{
  "tool": "compare",
  "arguments": {
    "animal_ids": ["430735-0032", "430735-0041", "430735-0058"],
    "traits": "BWT,WWT,YWT,EMD"
  }
}
```

---

### rank

Rank animals within a breed by weighted EBV traits. Specify trait weights to prioritize breeding goals.

| Parameter  | Type                      | Required | Default | Description                                            |
|------------|---------------------------|----------|---------|--------------------------------------------------------|
| `breed_id` | integer                   | yes      |         | Breed ID to search within                              |
| `weights`  | object\<string, number\>  | yes      |         | Trait weights (e.g. `{"BWT": -1.0, "WWT": 2.0}`)      |
| `gender`   | string                    | no       |         | `"Male"`, `"Female"`, or `"Both"`                      |
| `status`   | string                    | no       |         | Animal status filter (e.g. `"CURRENT"`)                |
| `top_n`    | integer                   | no       | 10      | Number of top-ranked results to return                 |

**Example -- terminal sire selection (prioritize growth, penalize birth weight):**

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

### inbreeding_check

Calculate Wright's coefficient of inbreeding (COI) for a potential sire-dam mating. Returns COI value, traffic-light rating (Green/Yellow/Red), and shared ancestors.

| Parameter | Type   | Required | Description               |
|-----------|--------|----------|---------------------------|
| `sire_id` | string | yes      | LPN ID of the sire        |
| `dam_id`  | string | yes      | LPN ID of the dam         |

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

### mating_recommendations

Find optimal mates for an animal: searches the breed for candidates, checks inbreeding, and ranks by trait complementarity.

| Parameter       | Type    | Required | Default | Description                                              |
|-----------------|---------|----------|---------|----------------------------------------------------------|
| `animal_id`     | string  | yes      |         | LPN ID of the animal to find mates for                   |
| `breed_id`      | integer | yes      |         | Breed ID to search for potential mates                   |
| `target_traits` | string  | no       |         | Traits to optimize (comma-separated, e.g. `"WWT,NLB"`)  |
| `max_results`   | integer | no       | 5       | Maximum number of recommendations                        |

When `target_traits` is omitted, defaults to `WWT` (weight 1.0), `BWT` (weight -0.5), and `NLB` (weight 0.5).

Traits where lower is preferred (`BWT`, `DAG`, `WEC`, `FEC`) automatically receive negative weights.

**Example:**

```json
{
  "tool": "mating_recommendations",
  "arguments": {
    "animal_id": "430735-0032",
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
      "rating": "Green"
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

### flock_summary

Summarize a flock's animals: count, gender breakdown, and average EBV traits.

| Parameter  | Type    | Required | Description                          |
|------------|---------|----------|--------------------------------------|
| `flock_id` | string  | yes      | Flock ID to summarize                |
| `breed_id` | integer | no       | Breed ID to filter within the flock  |

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

### database_status

Get NSIP database last-updated date and available animal statuses.

No parameters.

**Example:**

```json
{
  "tool": "database_status",
  "arguments": {}
}
```

---

## Resource Reference

Resources provide static and dynamic data that MCP clients can read directly via URI.

### Static Resources

| URI                        | Name              | MIME Type         | Description                                             |
|----------------------------|-------------------|-------------------|---------------------------------------------------------|
| `nsip://glossary`          | EBV Trait Glossary| text/markdown     | Definitions of all 13 NSIP EBV traits with units, interpretation, and selection direction |
| `nsip://breeds`            | Breed Directory   | application/json  | Live directory of all breed groups and breeds            |
| `nsip://guide/selection`   | Selection Guide   | text/markdown     | How to use EBVs for breeding decisions -- selection steps, objectives, trade-offs |
| `nsip://guide/inbreeding`  | Inbreeding Guide  | text/markdown     | COI thresholds, inbreeding depression effects, avoidance strategies |
| `nsip://status`            | Database Status   | application/json  | Live NSIP database status with last update date         |

### Resource Templates

Resource templates use URI parameters to fetch dynamic data from the NSIP API.

| URI Template                          | Name              | MIME Type         | Description                                      |
|---------------------------------------|-------------------|-------------------|--------------------------------------------------|
| `nsip://animal/{lpn_id}`              | Animal Profile    | application/json  | Full profile for a specific animal by LPN ID     |
| `nsip://animal/{lpn_id}/pedigree`     | Animal Pedigree   | application/json  | Pedigree / lineage tree for a specific animal    |
| `nsip://animal/{lpn_id}/progeny`      | Animal Progeny    | application/json  | Offspring list for a specific animal             |
| `nsip://breed/{breed_id}/ranges`      | Breed Trait Ranges| application/json  | Min/max trait value ranges for a specific breed  |

**Template examples:**

```
nsip://animal/430735-0032          -> Full profile for animal 430735-0032
nsip://animal/430735-0032/pedigree -> Pedigree tree
nsip://animal/430735-0032/progeny  -> Offspring list
nsip://breed/486/ranges            -> Trait ranges for breed 486
```

---

## Prompt Reference

Prompts are guided workflows that fetch live data from the NSIP API and construct structured context for an LLM to provide breeding advice. Each prompt returns one or more `PromptMessage` objects with pre-fetched data embedded.

### evaluate-ram

Evaluate a ram's breeding value. Fetches the animal's EBVs and constructs a comprehensive assessment emphasizing growth traits (WWT, YWT, EMD) and carcass quality (FAT). Considers value as terminal sire vs. maternal sire.

| Argument | Required | Description                |
|----------|----------|----------------------------|
| `lpn_id` | yes      | LPN ID of the ram          |

---

### evaluate-ewe

Evaluate a ewe's breeding value. Emphasizes maternal traits (NLB, NWT, PWT) and moderate birth weight (BWT). Considers prolificacy, lamb-rearing ability, and longevity potential.

| Argument | Required | Description                |
|----------|----------|----------------------------|
| `lpn_id` | yes      | LPN ID of the ewe          |

---

### compare-breeding-stock

Compare multiple animals side-by-side with trait-by-trait analysis. Fetches details for all animals and produces a structured comparison highlighting differences, relative strengths/weaknesses, and suitability for different breeding goals.

| Argument     | Required | Description                                         |
|--------------|----------|-----------------------------------------------------|
| `animal_ids` | yes      | Comma-separated LPN IDs of animals to compare (2-5) |

---

### plan-mating

Plan a specific mating. Fetches details and lineage for both sire and dam, calculates COI and trait complementarity, and produces a mating assessment with inbreeding safety check, predicted offspring quality, and an overall recommendation (proceed/caution/avoid).

| Argument  | Required | Description           |
|-----------|----------|-----------------------|
| `sire_id` | yes      | LPN ID of the sire    |
| `dam_id`  | yes      | LPN ID of the dam     |

---

### flock-improvement

Analyze a breed or flock for trait gaps and improvement opportunities. Fetches current animals and breed trait ranges, then identifies where the group falls below or exceeds breed average with prioritized improvement recommendations.

| Argument   | Required | Description                            |
|------------|----------|----------------------------------------|
| `breed_id` | yes      | Breed ID to analyze                    |
| `flock_id` | no       | Optional flock ID to narrow analysis   |

---

### select-replacement

Find top replacement candidates within a breed. Searches by gender and sorts by a target trait to identify the best replacements, with trade-off analysis and selection criteria summary.

| Argument       | Required | Description                                     |
|----------------|----------|-------------------------------------------------|
| `breed_id`     | yes      | Breed ID to search within                       |
| `gender`       | yes      | Gender of replacement animals (`"Male"` or `"Female"`) |
| `target_trait` | yes      | Primary trait to optimize (e.g. `"WWT"`, `"NLB"`)      |

---

### interpret-ebvs

Plain-language EBV explanation. Fetches the animal's details and the full EBV glossary, then produces a farmer-friendly interpretation of each trait -- avoiding jargon and using practical terms like "heavier lambs at weaning" instead of "higher WWT EBV."

| Argument | Required | Description                    |
|----------|----------|--------------------------------|
| `lpn_id` | yes      | LPN ID of the animal           |

---

## Analytics Reference

The MCP server includes built-in analytics computed locally (no additional API calls beyond data retrieval).

### Coefficient of Inbreeding (COI)

Calculates Wright's coefficient of inbreeding from pedigree data:

```
COI = Sigma [(0.5)^(n1 + n2 + 1)]
```

Where:
- `n1` = path length (in generations) from sire to common ancestor
- `n2` = path length (in generations) from dam to common ancestor
- The sum is taken over all paths through all common ancestors

**Traffic-light thresholds:**

| Rating   | COI Range     | Interpretation                               |
|----------|---------------|----------------------------------------------|
| Green    | < 6.25%       | Acceptable -- proceed with mating            |
| Yellow   | 6.25% - 12.5% | Elevated inbreeding -- consider alternatives |
| Red      | > 12.5%       | High inbreeding -- generally avoid           |

**Reference values:**
- 6.25% is equivalent to mating half-siblings
- 12.5% is equivalent to mating full siblings or parent-offspring
- 25% is equivalent to mating identical twins

### Weighted Trait Ranking

Animals are ranked by a weighted composite score:

```
Score = Sigma (trait_value * weight * accuracy / 100)
```

For each trait where both a weight and a value exist:
- `trait_value` is the animal's EBV for that trait
- `weight` is the user-specified importance (negative weights penalize higher values)
- `accuracy` is the trait's accuracy percentage (0-100), used as a confidence scaler

Animals are sorted in descending order by composite score.

### Trait Complementarity

Predicts midparent EBV values for potential offspring:

```
predicted_offspring_EBV = (sire_EBV + dam_EBV) / 2
```

Only traits present in both sire and dam are included. This provides a first-order estimate of the genetic merit expected in offspring.

---

## EBV Trait Glossary

Quick reference for all 13 NSIP Estimated Breeding Value traits.

| Abbreviation | Name                   | Unit    | Selection Direction                  | Description                                                              |
|--------------|------------------------|---------|--------------------------------------|--------------------------------------------------------------------------|
| BWT          | Birth Weight           | lbs     | Lower is generally preferred         | Predicted difference in birth weight. Lower values reduce dystocia risk. |
| WWT          | Weaning Weight         | lbs     | Higher is preferred                  | Predicted difference in weight at weaning (60 days).                     |
| PWWT         | Post-Weaning Weight    | lbs     | Higher is preferred                  | Predicted difference in post-weaning weight.                             |
| YWT          | Yearling Weight        | lbs     | Higher is preferred                  | Predicted difference in yearling weight (365 days).                      |
| FAT          | Fat Depth              | mm      | Moderate preferred (breed-dependent) | Predicted difference in subcutaneous fat depth at 12th-13th rib.         |
| EMD          | Eye Muscle Depth       | mm      | Higher is preferred                  | Predicted difference in loin eye muscle depth.                           |
| NLB          | Number of Lambs Born   | lambs   | Higher is preferred (with caution)   | Predicted difference in lambs born per lambing.                          |
| NWT          | Number of Lambs Weaned | lambs   | Higher is preferred                  | Predicted difference in lambs weaned per lambing.                        |
| PWT          | Pounds Weaned          | lbs     | Higher is preferred                  | Total weight of lambs weaned per ewe lambing.                            |
| DAG          | Dag Score              | score   | Lower is preferred                   | Predicted difference in fecal soiling of the breech.                     |
| WGR          | Wool Growth Rate       | g/day   | Higher is preferred (wool breeds)    | Predicted difference in daily wool growth.                               |
| WEC          | Worm Egg Count         | eggs/g  | Lower is preferred                   | Predicted difference in fecal worm egg count (parasite resistance).      |
| FEC          | Fecal Egg Count        | eggs/g  | Lower is preferred                   | Predicted difference in fecal egg count (alternate parasite measure).    |

### Common Breeding Objectives

- **Terminal sire:** Emphasize WWT, YWT, EMD, FAT (moderate). Use negative BWT weight.
- **Maternal flock:** Emphasize NLB, NWT, PWT with moderate BWT.
- **Dual purpose:** Balance growth (WWT, YWT) with maternal traits (NLB, NWT).
- **Parasite resistance:** Emphasize WEC and FEC (lower is better).
