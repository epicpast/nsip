# `mcp/invalid-cursor` — Invalid pagination cursor

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/invalid-cursor.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

A paginated MCP list request (`tools/list`, `resources/list`,
`resources/templates/list`, `prompts/list`) was given a `cursor` that could
not be decoded as an offset, or whose offset was past the end of the result
set. Cursors are opaque tokens returned in a prior page's `nextCursor`; they
must not be synthesized or mutated. (JSON-RPC code: `invalid_params` / -32602.)

## Recovery

Restart pagination without a cursor (begin from the first page), then follow
the `nextCursor` values verbatim. Deterministic — an agent may drop the bad
cursor and retry.
