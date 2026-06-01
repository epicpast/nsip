---
title: "Dual-Consumer Error Envelope (RFC 9457)"
description: "Emit CLI and MCP errors as RFC 9457 problem+json for agent consumers, with a miette diagnostic for humans"
type: adr
category: architecture
tags:
  - error-handling
  - rfc9457
  - cli
  - mcp
  - agents
status: accepted
created: "2026-06-01"
updated: "2026-06-01"
author: nsip maintainers
project: nsip
technologies:
  - rust
  - miette
  - thiserror
audience:
  - developers
related:
  - 0005-error-type-uri-policy.md
---

# Dual-Consumer Error Envelope (RFC 9457)

## Context

`nsip` is increasingly invoked by LLM agents — both as a CLI (whose stderr an
agent parses) and as an MCP server (whose tool errors an agent consumes). The
prior error surface served only the human: `main()` printed `eprintln!("Error:
{e}")` and returned a blanket `ExitCode::FAILURE` (1) for every error, with no
machine-readable structure, no per-error exit code, no retry signal, and no TTY
awareness. MCP tool errors carried only a `code` + `message` (the JSON-RPC
`data` field was always `None`).

This imbalance is a cost problem (verbose prose burns agent tokens), a
reliability problem (agents abandon recoverable failures with no retry signal),
and a convergence problem (agents apply plausible-but-wrong fixes when
applicability is ambiguous). The pattern and rationale are from Allen Walker's
*"CLI Error Messages Are a Dual-Consumer Problem"*
(<https://zircote.com/blog/2026/04/cli-error-messages-are-a-dual-consumer-problem/>).

## Decision

Every error is mapped to an [RFC 9457](https://www.rfc-editor.org/rfc/rfc9457)
Problem Details object ([`ProblemDetails`](../reference/ERROR-ENVELOPE.md)) with
the five standard members plus the agent extensions `exit_code`,
`suggested_fix`, `code_actions`, `retry_after`, and `docs_url`.

- **Dual rendering, one binary.** On a TTY (or `--format pretty`) the error is a
  `miette` graphical diagnostic; for a non-TTY consumer (or `--format json` /
  `-J`) it is `application/problem+json` on stderr. `miette`'s own JSON handler
  is **not** RFC 9457, so the JSON is a hand-rolled serde struct; `miette` is
  used only for the human rendering.
- **Per-variant exit codes** (`sysexits.h`-aligned) replace the blanket `1`.
- **MCP parity.** The same envelope is attached to MCP tool errors via the
  JSON-RPC `data` field, so MCP agent consumers get the identical contract.
- **Real retry signals.** The HTTP client parses the upstream `Retry-After`
  header and treats `429` as retryable, so `retry_after` is populated from live
  data for transient errors.
- **Applicability in the catalog.** `suggested_fix` applicability markers live
  in the [error catalog](../reference/errors/) keyed by `type`, not inline, to
  keep the envelope under ~1 KB.
- **OAuth stays RFC 6749.** The OAuth HTTP layer keeps its RFC 6749
  `{error, error_description}` responses (changing them would violate OAuth
  2.1); it only gains a `Retry-After` header on transient 503s and redacted
  secrets in `Debug`.

This mirrors the reference implementation in the sibling `git-creep` CLI.

## Consequences

- **Breaking:** exit codes now vary by error class (1 / 3 / 75) instead of
  always 1; piped/non-TTY invocations emit JSON errors by default; a bare
  `nsip` invocation emits the envelope rather than clap's raw usage text.
- Agents get a stable, low-token, actionable error contract across CLI and MCP.
- New error types are added as new `type` slugs (see
  [ADR-0005](0005-error-type-uri-policy.md)); the envelope is verified by unit,
  property, and CLI integration tests, including a 1 KB payload cap.
