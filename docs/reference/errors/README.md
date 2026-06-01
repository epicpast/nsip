# Error Catalog

Every `nsip` error maps to a stable problem **`type`** URI that resolves to one
of the pages below. Agents consult this catalog (keyed by `type`) for the
`suggested_fix` **applicability marker**, which is not carried inline in the
envelope (see [ERROR-ENVELOPE.md](../ERROR-ENVELOPE.md)).

## Catalog

| `type` slug | Variant | `status` | `exit_code` | Class | `suggested_fix` applicability |
|---|---|---|---|---|---|
| [`cli/validation`](cli/validation.md) | `Validation` | 400 | 1 | caller | `machine_applicable` |
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
