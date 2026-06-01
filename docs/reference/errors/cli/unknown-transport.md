# `cli/unknown-transport` — Unknown MCP transport

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/unknown-transport.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

The `nsip mcp --transport` value was something other than `stdio` or `http`.

## Recovery

Use `--transport stdio` or `--transport http`.
