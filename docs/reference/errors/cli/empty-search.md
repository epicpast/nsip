# `cli/empty-search` — Search request has no filter

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-search.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

A search or lookup carried no usable query: an empty `animal_details` search string, or a `search` with no filters.

## Recovery

Provide a non-empty query (an LPN ID or name) or at least one filter (breed, status, gender, born date, flock).
