# `cli/invalid-breed-id` — Breed ID is invalid

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/invalid-breed-id.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

A breed ID was non-positive or could not be parsed as an integer (CLI `trait-ranges`/`search`, MCP breed_id arguments, or the `nsip://breed/{id}/ranges` resource).

## Recovery

Provide a positive integer breed id. List valid breeds/IDs with the `breed_groups` tool or `nsip://breeds`.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/invalid-breed-id.md",
  "title": "Breed ID is invalid",
  "status": 400,
  "detail": "validation error: breed_id must be a positive integer, got 0",
  "instance": "urn:nsip:trait-ranges:1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d",
  "exit_code": 1,
  "suggested_fix": "provide a positive integer breed id (see the breed_groups tool)",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/invalid-breed-id.md"
}
```
