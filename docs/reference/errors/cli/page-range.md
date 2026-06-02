# `cli/page-range` — Pagination parameter out of range

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/page-range.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

A `page` or `page_size` value was out of range (`page_size` must be 1-100 for search; >= 1 for progeny).

## Recovery

Use `page >= 1` and `page_size` between 1 and 100, then retry.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/page-range.md",
  "title": "Pagination parameter out of range",
  "status": 400,
  "detail": "validation error: page_size must be between 1 and 100, got 250",
  "instance": "urn:nsip:search:2c3d4e5f-6a7b-8c9d-0e1f-2a3b4c5d6e7f",
  "exit_code": 1,
  "suggested_fix": "use page >= 1 and page_size between 1 and 100",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/page-range.md"
}
```
