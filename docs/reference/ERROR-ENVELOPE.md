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

### Configuring the `type` URI (forks and downstream)

The `type` / `docs_url` URI is `<base>/<slug>.md`. Both parts are configurable
from `Cargo.toml` via `[package.metadata.nsip]` (read at build time by
`build.rs`), so a fork can point the error catalog at its own documentation
without editing source:

```toml
[package.metadata.nsip]
# Base for every error's `type`/`docs_url`. Omit to use the default
# (the repository's docs path).
error-type-uri-base = "https://example.com/errors"

# Optional: remap individual error pages. Each key sets the slug for one error;
# unspecified errors keep their default slug.
[package.metadata.nsip.error-slugs]
api-timeout = "errors/timeout"
empty-lpn-id = "input/lpn"
```

Valid `error-slugs` keys (default slug in parentheses):

| key | default | key | default |
|-----|---------|-----|---------|
| `api-error` | `api/error` | `empty-lpn-id` | `cli/empty-lpn-id` |
| `api-not-found` | `api/not-found` | `invalid-breed-id` | `cli/invalid-breed-id` |
| `api-timeout` | `api/timeout` | `page-range` | `cli/page-range` |
| `api-connection` | `api/connection` | `empty-search` | `cli/empty-search` |
| `api-upstream-parse` | `api/upstream-parse` | `compare-arity` | `cli/compare-arity` |
| `validation` | `cli/validation` | `missing-argument` | `mcp/missing-argument` |
| `unknown-transport` | `cli/unknown-transport` | `unknown-resource` | `mcp/unknown-resource` |
| | | `invalid-cursor` | `mcp/invalid-cursor` |

**Scope:** the `error-type-uri-base` override applies to **both** the
machine-readable envelope (`type`/`docs_url`) **and** the human (`miette`)
terminal link. A per-error **slug** override applies to the **envelope only**;
the human terminal link keeps the variant's default slug. (`miette`'s `url`
attribute cannot vary by `ValidationKind`, so the human path always uses the
coarse `cli/validation.md` for validation errors regardless.)

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
