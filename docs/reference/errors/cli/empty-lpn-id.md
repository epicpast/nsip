# `cli/empty-lpn-id` — LPN ID must not be empty

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/empty-lpn-id.md`
- **status**: 400 · **exit_code**: 1 · **class**: caller
- **`suggested_fix` applicability**: `machine_applicable`
- **`retry_after`**: never

## When it occurs

An LPN ID argument (e.g. to `details`, `lineage`, `progeny`, `profile`) was empty or whitespace.

## Recovery

Provide a non-empty LPN ID. Deterministic — an agent may supply a valid ID and retry.
