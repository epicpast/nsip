# `mcp/unknown-resource` — Unknown resource URI

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/unknown-resource.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `maybe_incorrect`
- **`retry_after`**: never

> **Note:** The RFC 9457 envelope `status` is `400` (this is a `Validation` arm → `(exit_code 1, status 400)`); the `404`-flavoured `resource_not_found` / `-32002` is the JSON-RPC wire code, not the envelope status.

## When it occurs

A `resources/read` request used a URI that matched no static resource or template. (JSON-RPC code: `resource_not_found` / -32002.)

## Recovery

Use a documented `nsip://` URI — see `nsip://glossary` or the resource/template lists. Marked `maybe_incorrect`: the correct URI is not known to the tool, so surface to a human or enumerate resources rather than guessing.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/unknown-resource.md",
  "title": "Unknown resource URI",
  "status": 400,
  "detail": "validation error: unknown resource: nsip://glosary",
  "instance": "urn:nsip:resources/read:7c2f1a4b-9d3e-4f5a-8b6c-1e2d3f4a5b6c",
  "exit_code": 1,
  "suggested_fix": "use a documented nsip:// resource URI (see nsip://glossary)",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/mcp/unknown-resource.md"
}
```
