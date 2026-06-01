# `cli/compare-arity` — Comparison requires 2 to 5 animals

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/compare-arity.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

The `compare` tool or `compare-breeding-stock` prompt was given fewer than 2 or more than 5 LPN IDs.

## Recovery

Pass between 2 and 5 LPN IDs and retry.
