# `cli/empty-lpn-id` — LPN ID must not be empty

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-lpn-id.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

An LPN ID argument (e.g. to `details`, `lineage`, `progeny`, `profile`) was empty or whitespace.

## Recovery

Provide a non-empty LPN ID. Deterministic — an agent may supply a valid ID and retry.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-lpn-id.md",
  "title": "LPN ID must not be empty",
  "status": 400,
  "detail": "validation error: lpn_id cannot be empty",
  "instance": "urn:nsip:details:0e9d6c4a-7d4f-4f4c-bf6b-a8de4b1d5f9c",
  "exit_code": 1,
  "suggested_fix": "provide a non-empty LPN ID",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-lpn-id.md"
}
```
