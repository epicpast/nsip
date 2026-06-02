# `cli/unknown-transport` — Unknown MCP transport

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/unknown-transport.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

The `nsip mcp --transport` value was something other than `stdio` or `http`.

## Recovery

Use `--transport stdio` or `--transport http`.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/unknown-transport.md",
  "title": "Unknown MCP transport",
  "status": 400,
  "detail": "validation error: unknown transport 'grpc'",
  "instance": "urn:nsip:mcp:4e5f6a7b-8c9d-0e1f-2a3b-4c5d6e7f8a9b",
  "exit_code": 1,
  "suggested_fix": "use --transport stdio or --transport http",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/unknown-transport.md"
}
```
