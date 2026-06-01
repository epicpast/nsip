# `cli/invalid-breed-id` — Breed ID is invalid

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/invalid-breed-id.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

A breed ID was non-positive or could not be parsed as an integer (CLI `trait-ranges`/`search`, MCP breed_id arguments, or the `nsip://breed/{id}/ranges` resource).

## Recovery

Provide a positive integer breed id. List valid breeds/IDs with the `breed_groups` tool or `nsip://breeds`.
