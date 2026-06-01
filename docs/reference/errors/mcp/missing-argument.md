# `mcp/missing-argument` — Required argument is missing

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/missing-argument.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

An MCP prompt or tool was invoked without a required argument. The envelope `detail` names the argument.

## Recovery

Provide the named argument and retry. (JSON-RPC code: `invalid_params` / -32602.)
