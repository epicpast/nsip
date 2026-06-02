# `api/not-found` — Requested resource was not found

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/not-found.md`
- **status**: 404 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `maybe_incorrect`
- **`retry_after`**: never

## When it occurs

The NSIP API returned HTTP 404 — the requested animal, flock, or other resource
does not exist at the given identifier.

## Recovery

Verify the identifier exists (e.g. via `nsip search`). The suggested fix is
`maybe_incorrect`: the right identifier is not known to the tool, so an agent
should surface this to a human or search rather than guess.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/not-found.md",
  "title": "Requested resource was not found",
  "status": 404,
  "detail": "not found: resource not found at http://nsipsearch.nsip.org/api/...",
  "instance": "urn:nsip:details:1d2e3f4a-5b6c-7d8e-9f0a-1b2c3d4e5f6a",
  "exit_code": 1,
  "suggested_fix": "verify the identifier exists in the NSIP database (try `nsip search`)",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/not-found.md"
}
```
