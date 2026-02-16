---
title: "Documentation Directory Structure"
description: "Organize docs into template guides and operational runbooks"
type: adr
category: documentation
tags:
  - documentation
  - structure
status: accepted
created: "2026-02-16"
updated: "2026-02-16"
author: nsip maintainers
project: nsip
technologies:
  - markdown
audience:
  - developers
---

# Documentation Directory Structure

## Context

As the nsip project grew to include 30+ CI/CD workflows, security tooling, release automation, and AI coding agent support, the documentation expanded organically. Files were spread across multiple subdirectories (`docs/workflows/`, `docs/security/`, `docs/distribution/`, `docs/testing/`, `docs/ux/`, `docs/observability/`) without a clear organizing principle.

This created two problems:

1. **New users** who clicked "Use this template" had no guided path from repository creation to first CI pass. They had to discover relevant documentation by browsing directories.
2. **Maintainers** performing routine operations (releases, dependency updates, security responses) had no centralized runbooks. Operational knowledge was scattered across deployment guides and workflow-specific docs.

The existing documentation was accurate but lacked audience-aware organization.

## Decision

We will organize documentation into two audience-targeted directories:

### `docs/template/` — Template adoption guides (for new users)

Files that help developers who just created a repository from this template get productive quickly:

- `GETTING-STARTED.md` — End-to-end guide from "Use this template" to first CI pass
- `CONFIGURATION.md` — Cargo.toml fields, placeholder replacement, feature flags, editor setup
- `CI-WORKFLOWS.md` — Comprehensive guide to every included workflow
- `CUSTOMIZATION.md` — Adding modules, removing examples, adjusting lints, modifying targets
- `GITHUB-TEMPLATE-FEATURES.md` — What copies from templates and what doesn't (relocated)
- `COPILOT-JUMPSTART.md` — Copilot scaffolding prompts (relocated)

### `docs/runbooks/` — Operational runbooks (for maintainers)

Step-by-step procedures for ongoing project operations:

- `RELEASING.md` — End-to-end release process
- `DEPENDENCY-UPDATES.md` — Dependabot policy, manual auditing, adding/removing dependencies
- `SECURITY-RESPONSE.md` — Vulnerability triage, fix, and disclosure
- `CI-TROUBLESHOOTING.md` — Common CI failure patterns and fixes

### Retained directories

Existing deep-dive documentation (`docs/workflows/`, `docs/security/`, `docs/distribution/`, `docs/testing/`, `docs/ux/`, `docs/observability/`) is retained as detailed reference material. The new template and runbook guides link to these files where appropriate.

A `docs/README.md` serves as a documentation index linking all guides and references.

## Consequences

### Positive

- **Reduced adoption toil**: New users have a clear guided path from template to working project
- **Faster operations**: Maintainers have centralized runbooks for routine tasks
- **Audience clarity**: Documentation is organized by who reads it and when
- **Discoverability**: `docs/README.md` index makes all documentation findable

### Negative

- **Two levels of documentation**: Some topics are covered at both the guide level (template/runbooks) and reference level (existing subdirectories), requiring cross-linking
- **Maintenance burden**: Changes to workflows may need updates in both CI-WORKFLOWS.md and the corresponding workflow-specific doc

### Neutral

- Existing workflow and reference documentation is preserved unchanged
- The `docs/adr/` directory remains at its current location
