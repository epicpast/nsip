# `cli/compare-arity` — Comparison requires 2 to 5 animals

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/compare-arity.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

The `compare` tool or `compare-breeding-stock` prompt was given fewer than 2 or more than 5 LPN IDs.

## Recovery

Pass between 2 and 5 LPN IDs and retry.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/compare-arity.md",
  "title": "Comparison requires 2 to 5 animals",
  "status": 400,
  "detail": "validation error: compare requires 2 to 5 LPN IDs, got 1",
  "instance": "urn:nsip:compare:3f8a2c1d-5e6b-4a7c-9d0e-1f2a3b4c5d6e",
  "exit_code": 1,
  "suggested_fix": "pass between 2 and 5 LPN IDs",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/compare-arity.md"
}
```
