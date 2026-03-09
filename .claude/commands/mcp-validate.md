---
name: mcp-validate
description: Validate the NSIP MCP server by exercising all tools, prompts, and resources against a live instance.
allowed-tools:
  - Bash
  - Read
---

# NSIP MCP Validation Suite

Run a comprehensive validation of the NSIP MCP server. Each tool, prompt, and resource
is exercised via the `nsip` binary in stdio mode and the result is checked for correctness.

**Arguments**: `$ARGUMENTS`
- `--verbose`: Show full response payloads (default: show pass/fail summary only)
- `--skip-resources`: Skip resource tests (useful when API is unreachable)
- `--binary <path>`: Path to nsip binary (default: `cargo run --`)

## Known Test Animals

These LPN IDs come from `zircote/nsip-example` rank report data and are known-good for validation:

| Role | LPN ID | Gender | Notes |
|------|--------|--------|-------|
| RAM (sire) | `6402382024NCS310` | Male | Rank 9, DOB 2024-02-13 |
| EWE (dam) | `6401492025FLE087` | Female | Rank 1, DOB 2025-03-02 |
| EWE (dam 2) | `6401492020FLE249` | Female | Rank 2, DOB 2020-02-05 |
| Compare set | `6401492025FLE087`, `6402382024NCS310`, `6401492020FLE249`, `6401492025FLE082`, `6401492025FLE047` | Mixed | Top 5 for compare |

Use these IDs throughout the tests below — do NOT rely on dynamic search results for IDs.

## Execution Protocol

Build the binary first, then use `echo '...' | nsip mcp` to send JSON-RPC messages
over stdio. Parse the JSON-RPC response to determine pass/fail.

Track results as you go. **After EVERY test**, immediately print one line:

```
[test#] tool/endpoint — PASS|FAIL|SKIP — {brief reason}
```

Examples:
```
[1] initialize — PASS — protocol 2025-06-18, 13 tools
[5] search — PASS — returned 3 results
[18] evaluate-ram prompt — PASS — 3 messages returned
[30] nsip://glossary — PASS — text/plain, 1247 bytes
```

At the end, also print the full summary results table.

### Setup

Build the binary:
```bash
cargo build 2>/dev/null
```

Define a helper function for sending JSON-RPC to the MCP server via stdio.

**IMPORTANT**: The server reads stdin asynchronously. You MUST keep stdin open long enough
for the server to process the request. Use a subshell with `sleep` after the messages:

```bash
NSIP_BIN="./target/debug/nsip"

mcp_call() {
  local method="$1"
  local params="$2"
  local id="${3:-1}"
  local extra_args="${4:-}"
  local sleep_secs="${5:-5}"
  local init='{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"validator","version":"0.1.0"}}}'
  local initialized='{"jsonrpc":"2.0","method":"notifications/initialized"}'
  local call="{\"jsonrpc\":\"2.0\",\"id\":${id},\"method\":\"${method}\",\"params\":${params}}"
  (printf '%s\n%s\n%s\n' "$init" "$initialized" "$call"; sleep "$sleep_secs") \
    | $NSIP_BIN mcp $extra_args 2>/dev/null \
    | grep "\"id\":${id}" | head -1
}
```

For tools that make upstream API calls (search, details, lineage, progeny, profile, compare,
rank, inbreeding_check, mating_recommendations, flock_summary, trait_ranges, prompts/get),
use `sleep_secs=8` or higher. For local-only calls (tools/list, prompts/list, resources/*),
`sleep_secs=2` is sufficient.

### Phase 1: Lifecycle & Capabilities (tests 1-3)

1. **initialize**: Send initialize request
   - PASS if: response contains `protocolVersion`, `serverInfo` object, and `capabilities` includes `tools`, `resources`, `prompts`
   - Record `protocolVersion` and tool/resource/prompt capability flags

2. **tools/list**: `tools/list` with `{}`
   - PASS if: response contains `tools` array with exactly 13 entries
   - Record all tool names for later reference
   - Verify these tools are present: `search`, `details`, `lineage`, `progeny`, `profile`, `compare`, `rank`, `inbreeding_check`, `mating_recommendations`, `flock_summary`, `database_status`, `breed_groups`, `trait_ranges`

3. **tools/list (filtered)**: Start server with `--tools search,breed` and call `tools/list`
   - PASS if: response contains only 7 tools (5 search + 2 breed)
   - Verify: `search`, `details`, `lineage`, `progeny`, `profile`, `breed_groups`, `trait_ranges`
   - Verify absent: `compare`, `rank`, `flock_summary`

### Phase 2: Search Tools (tests 4-8)

These tests exercise the 5 search-category tools. All require the NSIP API to be reachable.

4. **search**: `tools/call` with `name: "search"`, `arguments: {"breed_id": 640, "status": "CURRENT", "page_size": 3}`
   - PASS if: response contains `content` array with text content, no `isError`

5. **details**: `tools/call` with `name: "details"`, `arguments: {"lpn_id": "6402382024NCS310"}`
   - PASS if: response contains animal details JSON with EBV data

6. **lineage**: `tools/call` with `name: "lineage"`, `arguments: {"lpn_id": "6401492025FLE087"}`
   - PASS if: response contains lineage data (sire/dam information)

7. **progeny**: `tools/call` with `name: "progeny"`, `arguments: {"lpn_id": "6401492020FLE249"}`
   - PASS if: response returns without error (may have 0 progeny)

8. **profile**: `tools/call` with `name: "profile"`, `arguments: {"lpn_id": "6402382024NCS310"}`
   - PASS if: response contains profile data with EBV summaries

### Phase 3: Analytics Tools (tests 9-12)

9. **compare**: `tools/call` with `name: "compare"`, `arguments: {"lpn_ids": ["6401492025FLE087", "6402382024NCS310", "6401492020FLE249"]}`
   - PASS if: response contains comparison data with multiple animals

10. **rank**: `tools/call` with `name: "rank"`, `arguments: {"breed_id": 640, "weights": {"WWT": 2.0}, "status": "CURRENT", "top_n": 5}`
    - PASS if: response contains ranked animals sorted by weighted score

11. **inbreeding_check**: `tools/call` with `name: "inbreeding_check"`, `arguments: {"sire_id": "6402382024NCS310", "dam_id": "6401492025FLE087"}`
    - PASS if: response contains COI analysis or graceful error for unrelated animals

12. **mating_recommendations**: `tools/call` with `name: "mating_recommendations"`, `arguments: {"lpn_id": "6402382024NCS310", "breed_id": 640, "max_results": 3}`
    - PASS if: response contains mating recommendation data

### Phase 4: Flock & Breed Tools (tests 13-16)

13. **flock_summary**: `tools/call` with `name: "flock_summary"`, `arguments: {"flock_id": "640238", "breed_id": 640}`
    - PASS if: response contains flock summary with averages and total_count

14. **database_status**: `tools/call` with `name: "database_status"`, `arguments: {}`
    - PASS if: response contains database status information

15. **breed_groups**: `tools/call` with `name: "breed_groups"`, `arguments: {}`
    - PASS if: response contains breed groups array with breed names and IDs

16. **trait_ranges**: `tools/call` with `name: "trait_ranges"`, `arguments: {"breed_id": 640}`
    - PASS if: response contains trait ranges with min/max/avg values

### Phase 5: Prompts (tests 17-23)

17. **prompts/list**: `prompts/list` with `{}`
    - PASS if: response contains `prompts` array with exactly 7 entries
    - Verify names: `evaluate-ram`, `evaluate-ewe`, `compare-breeding-stock`, `plan-mating`, `flock-improvement`, `select-replacement`, `interpret-ebvs`

18. **evaluate-ram prompt**: `prompts/get` with `name: "evaluate-ram"`, `arguments: {"lpn_id": "6402382024NCS310"}`
    - PASS if: response contains `messages` array with role/content pairs

19. **evaluate-ewe prompt**: `prompts/get` with `name: "evaluate-ewe"`, `arguments: {"lpn_id": "6401492025FLE087"}`
    - PASS if: response contains `messages` array

20. **compare-breeding-stock prompt**: `prompts/get` with `name: "compare-breeding-stock"`, `arguments: {"lpn_ids": "6401492025FLE087,6402382024NCS310,6401492020FLE249"}`
    - PASS if: response contains `messages` array with comparison context

21. **plan-mating prompt**: `prompts/get` with `name: "plan-mating"`, `arguments: {"sire_id": "6402382024NCS310", "dam_id": "6401492025FLE087"}`
    - PASS if: response contains `messages` array with mating plan context

22. **flock-improvement prompt**: `prompts/get` with `name: "flock-improvement"`, `arguments: {"breed_id": "640"}`
    - PASS if: response contains `messages` array with flock analysis

23. **select-replacement prompt**: `prompts/get` with `name: "select-replacement"`, `arguments: {"breed_id": "640", "gender": "Male", "target_trait": "WWT"}`
    - PASS if: response contains `messages` array with selection criteria

### Phase 6: Resources (tests 24-32)

24. **resources/list**: `resources/list` with `{}`
    - PASS if: response contains `resources` array with 5 static resources
    - Verify URIs: `nsip://glossary`, `nsip://breeds`, `nsip://guide/selection`, `nsip://guide/inbreeding`, `nsip://status`

25. **resources/templates/list**: `resources/templates/list` with `{}`
    - PASS if: response contains `resourceTemplates` array with 4 templates
    - Verify: `nsip://animal/{lpn_id}`, `nsip://animal/{lpn_id}/pedigree`, `nsip://animal/{lpn_id}/progeny`, `nsip://breed/{breed_id}/ranges`

26. **nsip://glossary**: `resources/read` with `uri: "nsip://glossary"`
    - PASS if: response contains text content with EBV terminology definitions

27. **nsip://breeds**: `resources/read` with `uri: "nsip://breeds"`
    - PASS if: response contains breed listing data

28. **nsip://guide/selection**: `resources/read` with `uri: "nsip://guide/selection"`
    - PASS if: response contains selection guide text

29. **nsip://guide/inbreeding**: `resources/read` with `uri: "nsip://guide/inbreeding"`
    - PASS if: response contains inbreeding guide text

30. **nsip://status**: `resources/read` with `uri: "nsip://status"`
    - PASS if: response contains API status information

31. **nsip://animal/{lpn_id}**: `resources/read` with `uri: "nsip://animal/6402382024NCS310"`
    - PASS if: response contains animal detail data

32. **nsip://breed/{breed_id}/ranges**: `resources/read` with `uri: "nsip://breed/640/ranges"`
    - PASS if: response contains trait range data for breed 640

### Phase 7: Transport & Configuration (tests 33-36)

33. **stdio transport**: Verify `cargo run -- mcp` accepts stdio JSON-RPC (already validated by all prior tests)
    - PASS if: all prior tests used stdio successfully

34. **HTTP transport startup**: `cargo run -- mcp --transport http --port 0 &` (port 0 for random assignment)
    - PASS if: process starts and logs "listening on" message
    - Kill the process after verification

35. **--tools filtering**: `echo '<tools/list>' | cargo run -- mcp --tools search`
    - PASS if: only 5 search tools returned

36. **unknown tool set**: `cargo run -- mcp --tools invalid_set 2>&1`
    - PASS if: process exits with error about unknown tool set
    - **NOTE**: This is a negative test — an error is the EXPECTED outcome.

### Phase 8: Report

Print the final results table:

```
## NSIP MCP Validation Report

**Server**: nsip-mcp v{version}
**Protocol**: {protocol_version}
**Date**: {current date/time}
**Tools**: {tool_count} registered
**Prompts**: {prompt_count} registered
**Resources**: {static_count} static, {template_count} templates
**Result**: {PASS_COUNT}/{TOTAL} passed, {SKIP_COUNT} skipped, {FAIL_COUNT} failed

| # | Category | Test | Result | Notes |
|---|----------|------|--------|-------|
| 1 | Lifecycle | initialize | PASS | protocol {version} |
| 2 | Lifecycle | tools/list | PASS | 13 tools |
| 3 | Lifecycle | tools/list (filtered) | PASS | 7 tools with --tools search,breed |
| 4 | Search | search | PASS | {N} results |
| 5 | Search | details | PASS | animal data returned |
| 6 | Search | lineage | PASS | pedigree data |
| 7 | Search | progeny | PASS | {N} progeny |
| 8 | Search | profile | PASS | EBV profile |
| 9 | Analytics | compare | PASS | {N} animals compared |
| 10 | Analytics | rank | PASS | ranked by WWT |
| 11 | Analytics | inbreeding_check | PASS | COI calculated |
| 12 | Analytics | mating_recommendations | PASS | recommendations returned |
| 13 | Flock | flock_summary | PASS | {N} animals, averages computed |
| 14 | Flock | database_status | PASS | status returned |
| 15 | Breed | breed_groups | PASS | {N} breed groups |
| 16 | Breed | trait_ranges | PASS | ranges for breed 640 |
| 17 | Prompts | prompts/list | PASS | 7 prompts |
| 18 | Prompts | evaluate-ram | PASS | {N} messages |
| 19 | Prompts | evaluate-ewe | PASS | {N} messages |
| 20 | Prompts | compare-breeding-stock | PASS | {N} messages |
| 21 | Prompts | plan-mating | PASS | {N} messages |
| 22 | Prompts | flock-improvement | PASS | {N} messages |
| 23 | Prompts | select-replacement | PASS | {N} messages |
| 24 | Resources | resources/list | PASS | 5 static resources |
| 25 | Resources | templates/list | PASS | 4 templates |
| 26 | Resources | nsip://glossary | PASS | glossary text |
| 27 | Resources | nsip://breeds | PASS | breed list |
| 28 | Resources | nsip://guide/selection | PASS | selection guide |
| 29 | Resources | nsip://guide/inbreeding | PASS | inbreeding guide |
| 30 | Resources | nsip://status | PASS | API status |
| 31 | Resources | nsip://animal/{id} | PASS | animal detail |
| 32 | Resources | nsip://breed/640/ranges | PASS | trait ranges |
| 33 | Transport | stdio | PASS | all tests via stdio |
| 34 | Transport | HTTP startup | PASS | listener started |
| 35 | Config | --tools filtering | PASS | 5 search tools only |
| 36 | Config | unknown tool set | PASS | error returned |
```

If any test failed, add a **Failures** section with details.

If `--verbose` was passed, also show the raw response for each tool call inline.
