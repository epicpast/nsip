---
status: accepted
date: 2026-02-16
---

# Use Architectural Decision Records

## Context

As `nsip` grows, we need a way to document important architectural decisions and their rationale. Without documentation, decisions can be lost over time, leading to:

- Repeated discussions on already-settled topics
- Loss of context about why certain approaches were chosen
- Difficulty onboarding new contributors
- Inconsistent architectural choices

We need a lightweight, version-controlled way to capture and track architectural decisions.

## Decision

We will use Architectural Decision Records (ADRs) to document significant architectural decisions in this project.

ADRs will:
- Be stored in `docs/adr/` directory
- Use the MADR (Markdown Architectural Decision Records) format
- Be numbered sequentially (0001, 0002, etc.)
- Include: title, status, context, decision, and consequences
- Be reviewed through pull requests like code changes
- Be validated automatically in CI using the adrscope action

## Consequences

### Positive

- **Transparency**: All architectural decisions are documented and visible
- **Context preservation**: Future maintainers understand why decisions were made
- **Collaboration**: ADRs provide a structured way to discuss and review decisions
- **Automation**: CI validates ADR format and generates browsable documentation
- **Git integration**: ADRs are versioned alongside code

### Negative

- **Overhead**: Requires discipline to document decisions as they're made
- **Learning curve**: Contributors need to understand ADR format
- **Maintenance**: ADRs may need updates as decisions evolve

### Neutral

- ADRs complement, but don't replace, code comments and documentation
- Not all decisions need an ADR—only architecturally significant ones
- ADR status can change over time (proposed → accepted → superseded/deprecated)
