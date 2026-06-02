# `cli/empty-search` — Search request has no filter

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-search.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

A search or lookup carried no usable query: an empty `animal_details` search string, or a `search` with no filters.

## Recovery

Provide a non-empty query (an LPN ID or name) or at least one filter (breed, status, gender, born date, flock).

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-search.md",
  "title": "Search request has no filter",
  "status": 400,
  "detail": "validation error: search requires a query or at least one filter",
  "instance": "urn:nsip:search:9a1b2c3d-4e5f-6a7b-8c9d-0e1f2a3b4c5d",
  "exit_code": 1,
  "suggested_fix": "provide a non-empty query (an LPN ID or name) or at least one search filter",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-search.md"
}
```
