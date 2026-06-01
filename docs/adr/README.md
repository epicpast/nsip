# Architectural Decision Records

This directory contains Architectural Decision Records (ADRs) for the `nsip` project.

## What is an ADR?

An Architectural Decision Record (ADR) is a document that captures an important architectural decision made along with its context and consequences.

## Format

We use [structured-MADR](https://github.com/zircote/structured-madr) (SMADR) format, validated by [adrscope](https://github.com/zircote/adrscope) in CI.

Each ADR file must:
- Be named `NNNN-title-with-dashes.md` where `NNNN` is a zero-padded sequential number
- Include YAML frontmatter with required fields:
  - `title`, `description`, `type: adr`, `category`, `tags`
  - `status` (proposed, accepted, deprecated, superseded)
  - `created`, `updated` (ISO 8601 dates)
  - `author`, `project`
- Include the following body sections:
  - Context (background, current limitations)
  - Decision Drivers (primary, secondary)
  - Considered Options (with risk assessments)
  - Decision
  - Consequences (positive, negative, neutral)

## Workflow

### Proposing a New ADR

1. Create a new ADR file in `docs/adr/` with the next sequential number
2. Set status to "proposed"
3. Fill in the context, decision, and consequences sections
4. Submit a pull request

### Accepting an ADR

1. After discussion and approval, change status to "accepted"
2. Merge the pull request

### Superseding an ADR

1. Create a new ADR that supersedes the old one
2. Update the old ADR's status to "superseded" and link to the new ADR
3. Set the new ADR's status to "accepted"

### Deprecating an ADR

If a decision is no longer relevant but hasn't been superseded:
1. Change status to "deprecated"
2. Add a note explaining why it's deprecated

## Viewing ADRs

ADRs are plain Markdown files in `docs/adr/` and can be read directly in the
repository or any Markdown viewer. They are validated and rendered locally with
[adrscope](https://github.com/zircote/adrscope).

> The automated ADR validation/viewer workflows are not currently included in
> this repository; run adrscope locally if you need format validation or an
> HTML viewer.

## ADR Index

- [ADR-0001](0001-use-architectural-decision-records.md) - Use Architectural Decision Records
- [ADR-0002](0002-documentation-directory-structure.md) - Documentation Directory Structure
- [ADR-0003](0003-adopt-diataxis-documentation-framework.md) - Adopt Diátaxis Documentation Framework
- [ADR-0004](0004-dual-consumer-error-envelope.md) - Dual-Consumer Error Envelope (RFC 9457)
- [ADR-0005](0005-error-type-uri-policy.md) - Error type URI Policy
