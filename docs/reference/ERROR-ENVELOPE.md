# Error Envelope (RFC 9457 Problem Details)

`nsip` is a **dual-consumer** CLI: every error serves both the human at the
terminal and the LLM agent that orchestrates the command and parses its output.
On a TTY the error is rendered as a [`miette`](https://docs.rs/miette) graphical
diagnostic; for any non-TTY consumer (a pipe, a file, an agent) or when
`--format json` is passed, the error is emitted to **stderr** as an
[RFC 9457](https://www.rfc-editor.org/rfc/rfc9457) `application/problem+json`
envelope.

The same envelope is attached to MCP tool errors (in the JSON-RPC error
`data` field), so MCP agent consumers receive the identical contract.

## Format selection

| Signal | Result |
|---|---|
| `--format json` | JSON envelope |
| `--format pretty` | miette diagnostic |
| `-J` / `--json` | JSON envelope (legacy alias) |
| stderr is a TTY (no flag) | miette diagnostic |
| stderr is **not** a TTY (no flag) | JSON envelope |

`--format`/`-J` also selects the **success** output format. With no flag,
success output stays human-readable; only error rendering auto-detects the TTY.

## Schema

| Member | Type | Meaning |
|---|---|---|
| `type` | URI string | Stable problem-type identifier; the agent's dispatch key. Resolves to a page under [`errors/`](errors/). |
| `title` | string | Short, stable summary of the problem type. |
| `status` | number | HTTP-class status (mirrors the upstream HTTP status for `Api`). |
| `detail` | string | Human-readable explanation specific to this occurrence. |
| `instance` | URI string | Per-occurrence correlation handle, `urn:nsip:<command>:<uuid>`. |
| `exit_code` | number | Process exit code (`sysexits.h`-aligned; see [`errors/`](errors/)). |
| `suggested_fix` | string? | Free-text recovery action. Omitted when no deterministic fix exists. |
| `code_actions` | array | LSP-style structured edits. Currently always omitted (reserved). |
| `retry_after` | number \| string? | Delta-seconds or RFC 3339 timestamp; present only on transient errors. |
| `docs_url` | URI string | Stable documentation URL (equal to `type`). |

Optional/empty members are omitted from the JSON to keep the payload small
(under 1 KB), reducing per-`tool_result` token cost for agent consumers.

### Applicability of `suggested_fix`

Per the rustc diagnostic precedent, every `suggested_fix` carries an
applicability marker — but, for payload economy, the markers live in the
[error catalog](errors/) keyed by `type` rather than inline in the JSON. An
agent resolves the marker (`machine_applicable`, `maybe_incorrect`,
`has_placeholders`, `unspecified`) by looking up the `type` URI.

## `type` URI policy

URIs are **stable forever** and carry **no version path segment**. A problem
type's meaning never changes in place; new types may be added (non-breaking)
but existing slugs are never repurposed. Semantic changes are tracked in this
documentation and the project changelog. See
[ADR-0005](../adr/0005-error-type-uri-policy.md).

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/error.md",
  "title": "Upstream API returned an error",
  "status": 429,
  "detail": "API error (HTTP 429): rate limited",
  "instance": "urn:nsip:search:b8a9c0f3-f8fc-44a0-8c9c-f9dc78b1b7c2",
  "exit_code": 75,
  "suggested_fix": "wait for the retry_after interval before retrying",
  "retry_after": 30,
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/error.md"
}
```

## See also

- [Error catalog](errors/) — every problem type with exit code, status, and applicability.
- [ERROR-HANDLING.md](ERROR-HANDLING.md) — library-level error handling guidance.
- [ADR-0004](../adr/0004-dual-consumer-error-envelope.md) — why the dual-consumer envelope.
