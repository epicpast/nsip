# `mcp/missing-argument` — Required argument is missing

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/missing-argument.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

An MCP prompt or tool was invoked without a required argument. The envelope `detail` names the argument.

## Recovery

Provide the named argument and retry. (JSON-RPC code: `invalid_params` / -32602.)

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/missing-argument.md",
  "title": "Required argument is missing",
  "status": 400,
  "detail": "validation error: lpn_id",
  "instance": "urn:nsip:details:6a7b8c9d-0e1f-2a3b-4c5d-6e7f8a9b0c1d",
  "exit_code": 1,
  "suggested_fix": "provide the required argument: lpn_id",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/missing-argument.md"
}
```
