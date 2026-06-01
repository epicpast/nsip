# Error Catalog

Every `nsip` error maps to a stable problem **`type`** URI that resolves to one
of the pages below. Agents consult this catalog (keyed by `type`) for the
`suggested_fix` **applicability marker**, which is not carried inline in the
envelope (see [ERROR-ENVELOPE.md](../ERROR-ENVELOPE.md)).

## Catalog

### Operation / input validation (`Error::Validation`, by [`ValidationKind`])

Each carries `status` 400, `exit_code` 1, class caller, and no `retry_after`.
Over MCP these map to the `invalid_params` JSON-RPC code (`mcp/unknown-resource`
keeps `resource_not_found`).

| `type` slug | `ValidationKind` | `suggested_fix` applicability |
|---|---|---|
| [`cli/validation`](cli/validation.md) | `Other` (generic fallback) | `maybe_incorrect` |
| [`cli/empty-lpn-id`](cli/empty-lpn-id.md) | `EmptyLpnId` | `machine_applicable` |
| [`cli/invalid-breed-id`](cli/invalid-breed-id.md) | `InvalidBreedId` | `machine_applicable` |
| [`cli/page-range`](cli/page-range.md) | `PageRange` | `machine_applicable` |
| [`cli/empty-search`](cli/empty-search.md) | `EmptySearch` | `machine_applicable` |
| [`cli/compare-arity`](cli/compare-arity.md) | `CompareArity` | `machine_applicable` |
| [`cli/unknown-transport`](cli/unknown-transport.md) | `UnknownTransport` | `machine_applicable` |
| [`mcp/missing-argument`](mcp/missing-argument.md) | `MissingArgument` | `machine_applicable` |
| [`mcp/unknown-resource`](mcp/unknown-resource.md) | `UnknownResource` | `maybe_incorrect` |

### Transport / upstream (`Api`, `NotFound`, `Timeout`, `Connection`, `Parse`)

| `type` slug | Variant | `status` | `exit_code` | Class | `suggested_fix` applicability |
|---|---|---|---|---|---|
| [`api/error`](api/error.md) | `Api` | upstream (4xx/5xx) | 1 (4xx) / 75 (429, 5xx) | caller or transient | `unspecified` (429/5xx) / none (4xx) |
| [`api/not-found`](api/not-found.md) | `NotFound` | 404 | 1 | caller | `maybe_incorrect` |
| [`api/timeout`](api/timeout.md) | `Timeout` | 504 | 75 | transient | `unspecified` |
| [`api/connection`](api/connection.md) | `Connection` | 503 | 75 | transient | `unspecified` |
| [`api/upstream-parse`](api/upstream-parse.md) | `Parse` | 502 | 3 | environment | none |

## Exit-code table

NSIP commits to this `sysexits.h`-aligned mapping:

| Exit code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Caller error — bad input, a 4xx upstream response, or resource not found. |
| `3` | Environment error — the upstream returned a body that could not be parsed. |
| `75` (`EX_TEMPFAIL`) | Transient — timeout, connection failure, `429`, or `5xx`. `retry_after` is populated when a delay is known. |

The RFC 9457 source post gives a single illustrative mapping (`429 → 2`) but
defers the full table to the CLI author. NSIP deliberately uses `EX_TEMPFAIL`
(75) for all transient classes for consistency with Unix conventions.

## Applicability markers

| Marker | Agent action |
|---|---|
| `machine_applicable` | Apply the fix and retry without human confirmation. |
| `maybe_incorrect` | Surface to a human; do not auto-apply. |
| `has_placeholders` | Fix contains slots to fill; lower confidence. |
| `unspecified` | Treat as `maybe_incorrect`. |

See [ADR-0004](../../adr/0004-dual-consumer-error-envelope.md) and
[ADR-0005](../../adr/0005-error-type-uri-policy.md) for rationale.
