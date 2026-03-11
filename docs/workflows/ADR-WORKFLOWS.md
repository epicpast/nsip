---
diataxis_type: reference
---
# ADR Workflows

## Overview

Two workflows automate the management and publication of Architectural Decision
Records (ADRs) stored in `docs/adr/`. They validate ADR format on every change
and optionally generate an HTML viewer for GitHub Pages.

**Workflows:**
- `.github/workflows/adr-validation.yml` — Validates ADR format and posts PR statistics
- `.github/workflows/adr-viewer.yml` — Generates an HTML ADR browser artifact

**Tool:** [`zircote/adrscope`](https://github.com/zircote/adrscope)

---

## ADR Validation Workflow

**Workflow:** `.github/workflows/adr-validation.yml`  
**Trigger:** Push to `main`/`master` or pull requests that touch `docs/adr/**`, manual

### What It Does

1. **Validates ADR format** — Checks that each ADR file (`[0-9]*.md`) follows
   the required structure (title, date, status, context, decision, consequences).
2. **Generates statistics** — Counts ADRs by status (Proposed, Accepted,
   Deprecated, Superseded).
3. **Comments on PRs** — Posts a summary table with ADR statistics when a PR
   modifies `docs/adr/`.

### PR Comment Format

When you open or update a pull request that changes ADR files, the workflow
posts a comment like:

```
## ADR Validation Results

📊 Statistics:
- Total ADRs: 4
- Accepted: 3
- Proposed: 1
- Deprecated: 0
- Superseded: 0

✅ All ADRs validated successfully!
```

### Permissions

| Permission | Scope | Reason |
|-----------|-------|--------|
| `contents: read` | Repository | Checkout code |
| `pull-requests: write` | Repository | Post PR comments |

---

## ADR Viewer Workflow

**Workflow:** `.github/workflows/adr-viewer.yml`  
**Trigger:** Push to `main`/`master` that touches `docs/adr/**`, manual  
**Status:** Active (builds artifact); GitHub Pages deployment is opt-in

### What It Does

1. Generates an HTML site from all ADR Markdown files using `adrscope html`
2. Uploads the result as a workflow artifact named `adr-viewer` (retained 90 days)

### Enabling GitHub Pages Deployment

The workflow generates the viewer artifact but does not publish to GitHub Pages
by default. To enable Pages deployment:

1. Add a `deploy` job to `adr-viewer.yml` using `actions/deploy-pages`
2. Enable GitHub Pages in **Settings → Pages**, choosing **GitHub Actions** as
   the source
3. Ensure the `pages: write` and `id-token: write` permissions are present

See [adrscope GitHub Pages deployment](https://github.com/zircote/adrscope#github-pages-deployment)
for step-by-step instructions.

---

## ADR File Format

ADRs must follow this structure to pass validation:

```markdown
# ADR-NNNN: Title

**Date:** YYYY-MM-DD  
**Status:** Proposed | Accepted | Deprecated | Superseded

## Context

Why is this decision needed? What forces are at play?

## Decision

What was decided?

## Consequences

What are the results of the decision?
```

Files must be named `NNNN-kebab-case-title.md` (four-digit zero-padded number
followed by a hyphen and a lowercase title).

---

## Adding a New ADR

```bash
# 1. Determine the next ADR number
ls docs/adr/ | grep -E '^[0-9]' | sort | tail -1

# 2. Create the file
cp docs/adr/0001-use-architectural-decision-records.md \
   docs/adr/0004-your-decision-title.md

# 3. Edit with your decision content
# 4. Open a pull request — the validation workflow runs automatically
```

See [docs/adr/README.md](../adr/README.md) for the full ADR process.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Validation fails | ADR missing required section | Add the missing section (Context, Decision, or Consequences) |
| Validation fails | Filename doesn't match pattern | Rename to `NNNN-kebab-title.md` |
| No PR comment | Workflow didn't trigger | Ensure the PR touches a file under `docs/adr/` |
| Viewer artifact empty | No ADR files in `docs/adr/` | Add at least one valid ADR |
