# `mcp/unknown-resource` — Unknown resource URI

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/unknown-resource.md`
- **status**: 404 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `maybe_incorrect`
- **`retry_after`**: never

## When it occurs

A `resources/read` request used a URI that matched no static resource or template. (JSON-RPC code: `resource_not_found` / -32002.)

## Recovery

Use a documented `nsip://` URI — see `nsip://glossary` or the resource/template lists. Marked `maybe_incorrect`: the correct URI is not known to the tool, so surface to a human or enumerate resources rather than guessing.
