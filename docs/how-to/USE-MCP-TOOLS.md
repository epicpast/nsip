# How to Use the MCP Server Tools

> **Problem:** You want to use the NSIP MCP server with an AI assistant (Claude Desktop, Claude Code, Cursor, etc.) to query sheep genetic data interactively.

**Prerequisites:**
- `nsip` binary installed ([installation options](../MCP.md#installation))
- An MCP-compatible client (Claude Desktop, Claude Code, or similar)

---

## Step 1: Configure the MCP Server

Add the NSIP server to your MCP client configuration.

### Claude Code

Create or edit `.mcp.json` at your project root:

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

### Claude Desktop

Edit `claude_desktop_config.json` (on macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`):

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

### Docker

If you prefer not to install the binary:

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

---

## Step 2: Verify the Server Starts

Test that the server responds to an MCP initialize request:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | nsip mcp
```

A successful response includes `"result"` with server capabilities.

---

## Step 3: Use Data Retrieval Tools

### Check Database Status

```json
{ "tool": "database_status", "arguments": {} }
```

Returns the last-updated date and available statuses.

### List Breed Groups

```json
{ "tool": "breed_groups", "arguments": {} }
```

Returns all breed groups with their breed IDs and names. Use these IDs in other tool calls.

### Get Trait Ranges for a Breed

```json
{ "tool": "trait_ranges", "arguments": { "breed_id": 486 } }
```

Returns min/max EBV values for the breed, useful for understanding norms.

---

## Step 4: Look Up Individual Animals

### Get Animal Details

```json
{
  "tool": "details",
  "arguments": { "animal_id": "430735-0032" }
}
```

Returns EBVs, breed, status, contact info, and lineage identifiers.

### Get Full Profile

```json
{
  "tool": "profile",
  "arguments": { "animal_id": "430735-0032" }
}
```

Returns details, pedigree, and offspring in a single call.

### Get Lineage

```json
{
  "tool": "lineage",
  "arguments": { "animal_id": "430735-0032" }
}
```

Returns the ancestry tree (parents, grandparents).

### Get Progeny

```json
{
  "tool": "progeny",
  "arguments": { "animal_id": "430735-0032", "page": 0, "page_size": 20 }
}
```

Returns paginated offspring list.

---

## Step 5: Search and Compare

### Search Animals

```json
{
  "tool": "search",
  "arguments": {
    "breed_id": 486,
    "gender": "Male",
    "status": "CURRENT",
    "sort_by": "WWT",
    "reverse": true,
    "page_size": 10
  }
}
```

### Compare Animals Side-by-Side

```json
{
  "tool": "compare",
  "arguments": {
    "animal_ids": ["430735-0032", "430735-0041"],
    "traits": "BWT,WWT,YWT,PEMD"
  }
}
```

---

## Step 6: Use Breeding Analytics Tools

### Rank Animals by Weighted Traits

```json
{
  "tool": "rank",
  "arguments": {
    "breed_id": 486,
    "weights": { "BWT": -1.0, "WWT": 2.0, "YWT": 1.5, "PEMD": 1.0 },
    "gender": "Male",
    "status": "CURRENT",
    "top_n": 5
  }
}
```

### Check Inbreeding Coefficient

```json
{
  "tool": "inbreeding_check",
  "arguments": {
    "sire_id": "430735-0032",
    "dam_id": "430735-0089"
  }
}
```

Returns Wright's COI with a traffic-light rating (Green/Yellow/Red).

### Get Mating Recommendations

```json
{
  "tool": "mating_recommendations",
  "arguments": {
    "animal_id": "430735-0032",
    "breed_id": 486,
    "target_traits": "WWT,PEMD,NLB",
    "max_results": 3
  }
}
```

### Summarize a Flock

```json
{
  "tool": "flock_summary",
  "arguments": {
    "flock_id": "430735",
    "breed_id": 486
  }
}
```

---

## Step 7: Use Guided Prompts

MCP prompts are pre-built workflows that fetch data and construct structured breeding advice. Ask your AI assistant to use them:

| Prompt                    | Description                                         | Required Arguments              |
|---------------------------|-----------------------------------------------------|---------------------------------|
| `evaluate-ram`            | Assess a ram's breeding value                       | `lpn_id`                        |
| `evaluate-ewe`            | Assess a ewe's breeding value                       | `lpn_id`                        |
| `compare-breeding-stock`  | Side-by-side trait analysis of 2-5 animals          | `animal_ids` (comma-separated)  |
| `plan-mating`             | Mating assessment with COI and trait complementarity | `sire_id`, `dam_id`             |
| `flock-improvement`       | Identify trait gaps and improvement opportunities   | `breed_id`, optional `flock_id` |
| `select-replacement`      | Find top replacement candidates                     | `breed_id`, `gender`, `target_trait` |
| `interpret-ebvs`          | Plain-language EBV explanation                      | `lpn_id`                        |

---

## Step 8: Access Resources

MCP resources provide static and dynamic data by URI:

```
nsip://glossary              -- EBV trait definitions
nsip://breeds                -- Live breed directory
nsip://guide/selection       -- Selection guide
nsip://guide/inbreeding      -- Inbreeding guide
nsip://status                -- Database status
nsip://animal/{lpn_id}       -- Animal profile
nsip://breed/{breed_id}/ranges -- Trait ranges for a breed
```

---

## Verify It Works

After configuration, ask your AI assistant a question like "What breeds are available in the NSIP database?" If it uses the `breed_groups` tool and returns results, the server is working correctly.

---

## See Also

- [MCP Server Reference](../MCP.md) -- complete tool, resource, and prompt documentation
- [How to Compare Animals](COMPARE-ANIMALS.md)
- [How to Filter Search Results](FILTER-SEARCH-RESULTS.md)
