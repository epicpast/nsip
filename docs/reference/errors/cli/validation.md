# `cli/validation` — Invalid input parameters

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/validation.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `maybe_incorrect`
- **`retry_after`**: never

## When it occurs

Input failed local validation before any network request: an empty/blank
identifier, an out-of-range page or breed id, an unknown `mcp` transport, or a
malformed command line (clap parse failure, including a missing subcommand).

## Recovery

Correct the offending argument and re-run. The `detail` names the specific
problem. This is the generic `Other` fallback, so its `suggested_fix` is
advisory (`maybe_incorrect`) — an agent should verify the corrected input rather
than apply it blindly. The specific validation types (e.g.
[`cli/empty-lpn-id`](empty-lpn-id.md)) carry `machine_applicable` fixes.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/validation.md",
  "title": "Invalid input parameters",
  "status": 400,
  "detail": "validation error: lpn_id cannot be empty",
  "instance": "urn:nsip:details:0e9d6c4a-7d4f-4f4c-bf6b-a8de4b1d5f9c",
  "exit_code": 1,
  "suggested_fix": "correct the input and retry: lpn_id cannot be empty",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/validation.md"
}
```
