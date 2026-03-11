---
diataxis_type: reference
---
# Agentic Workflows

## Overview

Autonomous AI agents that monitor, maintain, and improve the repository through GitHub Actions workflows. These workflows use GitHub Copilot to perform intelligent tasks without manual intervention.

**Location:** `.github/workflows/*.md` (source) → `*.lock.yml` (compiled)  
**Engine:** GitHub Copilot  
**Compilation:** `gh aw compile`

## Workflow Catalog

### CI Doctor

**Workflow:** `.github/workflows/ci-doctor.md`  
**Trigger:** `workflow_run` (on CI failure)  
**Purpose:** Automated CI failure investigation

Analyzes failed GitHub Actions workflows to identify root causes and patterns. Performs deep investigation of logs, error messages, and configurations to provide actionable remediation steps.

**Features:**
- Root cause analysis of CI failures
- Pattern detection across multiple failures
- Actionable remediation recommendations
- Automated issue creation with diagnostic reports

**Monitored Workflows:**
- Daily Perf Improver
- Daily Test Coverage Improver

**Safe Outputs:**
- Creates issues prefixed with the workflow name (for example, `CI Doctor`)
- Adds labels: `automation`, `ci`
- Can comment on existing issues

---

### Daily Documentation Review

**Workflow:** `.github/workflows/daily-docs-review.md`  
**Trigger:** `schedule: daily`, `workflow_dispatch`  
**Purpose:** Validate documentation accuracy and freshness

Performs daily review of repository documentation to ensure accuracy against authoritative external sources (github.com, githubnext.com, gh-aw resources).

**Features:**
- Documentation accuracy validation
- External source cross-referencing
- Automated correction PRs
- Cache memory for tracking state

**Network Access:**
- `*.github.com`
- `github.com`
- `*.githubnext.com`
- `githubnext.com`
- `api.github.com`
- `github.blog`

**Safe Outputs:**
- Creates PRs with `docs:` prefix
- Not draft (ready for review)

---

### Daily QA

**Workflow:** `.github/workflows/daily-qa.md`  
**Trigger:** `schedule: daily`, `workflow_dispatch`  
**Purpose:** Ad hoc quality assurance

Validates project health daily by checking builds, tests, documentation clarity, and code structure. Acts as an autonomous QA engineer.

**Features:**
- Build and test validation
- Documentation clarity checks
- Code structure analysis
- Test coverage verification
- Creates discussions for findings
- Can submit improvement PRs

**Safe Outputs:**
- Creates discussions in `q-a` category with workflow name prefix
- Comments on issues/PRs (max 5)
- Creates draft PRs with `automation`, `qa` labels

---

### Daily Repository Status

**Workflow:** `.github/workflows/daily-repo-status.md`  
**Trigger:** `schedule: daily`, `workflow_dispatch`  
**Purpose:** Repository health monitoring

Daily health check of repository status, open issues, PR backlog, and workflow performance. Provides visibility into project maintenance needs.

**Features:**
- Issue and PR backlog analysis
- Workflow performance monitoring
- Stale issue detection
- Repository metrics tracking
- Status summaries via GitHub issues

**Safe Outputs:**
- Creates issues for status reports with `[repo-status]` title prefix
- Adds labels: `report`, `daily-status`

---

### Issue Triage

**Workflow:** `.github/workflows/issue-triage.md`  
**Trigger:** `issues: [opened, reopened]`, `reaction: eyes`  
**Purpose:** Intelligent issue classification

Analyzes new and reopened issues to select appropriate labels, detect spam, gather context from similar issues, and provide analysis notes.

**Features:**
- Automatic label selection from available labels
- Spam detection
- Similar issue detection
- Debugging strategy suggestions
- Reproduction step identification
- Resource link provision

**Safe Outputs:**
- Adds labels (max 5)
- Can add analysis comment

**Network Access:** Defaults (can access 3rd-party resources in public repos)

---

### Q - Agentic Workflow Optimizer

**Workflow:** `.github/workflows/q.md`  
**Trigger:** `slash_command: /q`, `reaction: rocket`  
**Purpose:** Workflow optimization expert

Expert system that improves, optimizes, and fixes agentic workflows by investigating performance, identifying missing tools, and detecting inefficiencies.

**Features:**
- Workflow performance investigation via live logs
- Missing tool and permission detection
- Inefficiency detection (excessive tool calls)
- Common pattern extraction
- Reusable workflow step generation
- Optimization PRs

**Safe Outputs:**
- Comments (max 1)
- Creates PRs with `[q]` prefix and `automation`, `workflow-optimization` labels
- Not draft (ready for review)
- Ignores if no changes

**Tools:**
- `agentic-workflows` (workflow inspection)
- `bash: true` (all commands)
- `edit` (file modification)

---

### Update Docs

**Workflow:** `.github/workflows/update-docs.md`  
**Trigger:** `push: [main]`, `workflow_dispatch`  
**Purpose:** Documentation synchronization

Autonomous technical writer that ensures code changes are mirrored by clear, accurate documentation following Diátaxis framework and style guidelines.

**Features:**
- Code change analysis
- Documentation gap detection
- Style consistency enforcement (Google, Microsoft guidelines)
- Progressive disclosure structure
- Accessibility and i18n readiness
- Draft PRs for documentation updates

**Safe Outputs:**
- Creates draft PRs with `automation`, `documentation` labels

**Style Guidelines:**
- Diátaxis framework (tutorials, how-to, reference, explanation)
- Google Developer Style Guide
- Microsoft Writing Style Guide
- Active voice, plain English
- Progressive disclosure

---

### Agentic Maintenance

**Workflow:** `.github/workflows/agentics-maintenance.yml`  
**Trigger:** `schedule: daily (00:37 UTC)`, `workflow_dispatch`  
**Purpose:** Housekeeping for expired agentic workflow outputs

Auto-generated by `gh-aw v0.56.2`. Closes discussions, issues, and pull requests that were created by agentic workflows and have passed their configured `expires` deadline. Also supports manual lifecycle operations on the `gh-aw` toolchain itself.

**Features:**
- Closes expired discussions, issues, and pull requests
- Supports manual operations: `disable`, `enable`, `update`, `upgrade`
- Permission-gated: manual operations require admin or maintainer access
- Schedule frequency driven by the shortest `expires` value across all agentic workflows

**Jobs:**
- `close-expired-entities` — runs on schedule; closes expired items across discussions, issues, and PRs
- `run_operation` — runs on `workflow_dispatch` with a non-empty operation; executes the requested `gh-aw` lifecycle command

> **Note:** This file is **auto-generated** by `gh-aw`. Do not edit it manually. Run `gh aw compile` to regenerate after changing expiration settings in any agentic workflow source file.

---

## Compilation and Deployment

### Source Files

Agentic workflows use `.md` source files with YAML frontmatter:

```markdown
---
description: Workflow description
on:
  push:
    branches: [main]
permissions: read-all
network: defaults
safe-outputs:
  create-pull-request:
    draft: true
tools:
  bash: true
timeout-minutes: 15
source: githubnext/agentics/workflows/template.md@COMMIT_SHA
---

# Workflow Name

Agent instructions go here...
```

### Compilation

Convert `.md` source to `.lock.yml` executable workflow:

```bash
# Compile all workflows
gh aw compile

# Compile specific workflow
gh aw compile .github/workflows/update-docs.md
```

**Output:** `.lock.yml` file (auto-generated, do not edit manually)

`gh aw compile` also regenerates `agentics-maintenance.yml` whenever any workflow uses the `expires` field. This file follows the same "do not edit manually" rule.

### Lockfile Attributes

`.lock.yml` files are marked in `.gitattributes`:
- `linguist-generated=true` (excluded from language stats)
- `merge=ours` (prefer local version in merge conflicts)

`agentics-maintenance.yml` is likewise auto-generated and should be treated as a build artifact.

---

## Network Configuration

### Firewall Modes

1. **`network: defaults`** - Default GitHub Actions network access
2. **`network: { firewall: true, allowed: [...] }`** - Restricted access to specific domains

### Examples

**Unrestricted:**
```yaml
network: defaults
```

**Restricted:**
```yaml
network:
  firewall: true
  allowed:
    - "*.github.com"
    - "api.github.com"
```

---

## Safe Outputs

Agentic workflows can create GitHub resources via safe output tools:

### Available Tools

- `create-issue`
- `update-issue`
- `add-comment`
- `create-pull-request`
- `add-labels`
- `create-discussion`

### Configuration

```yaml
safe-outputs:
  create-pull-request:
    title-prefix: "docs: "
    draft: false
    labels: [automation, documentation]
    max: 1
  add-comment:
    target: "*"  # all issues and PRs
    max: 5
```

---

## Tools Configuration

### Common Tool Sets

**Full bash access:**
```yaml
tools:
  bash: true
```

**Restricted bash commands:**
```yaml
tools:
  bash:
    - ls
    - find
    - grep
    - cat
```

**GitHub API access:**
```yaml
tools:
  github:
    toolsets: [all]
    # Or specific: [issues, pull-requests, discussions]
```

**Other tools:**
```yaml
tools:
  edit: {}
  web-fetch: {}
  memory: cache-memory
  agentic-workflows: {}
```

---

## Monitoring and Debugging

### Workflow Logs

View execution logs:
```bash
gh run list --workflow=update-docs.lock.yml
gh run view <run-id> --log

# For the maintenance workflow
gh run list --workflow=agentics-maintenance.yml
```

### Testing Locally

Manually trigger workflows:
```bash
gh workflow run update-docs.lock.yml
gh workflow run daily-qa.lock.yml

# Trigger a maintenance operation manually
gh workflow run agentics-maintenance.yml -f operation=update
```

### Optimization

Use `/q` slash command in issues/PRs to invoke the Q workflow optimizer for investigating and improving workflow performance.

---

## Best Practices

1. **Edit source files (`.md`), not lockfiles (`.lock.yml`)**
2. **Run `gh aw compile` after editing source files**
3. **Test with `workflow_dispatch` trigger before relying on automation**
4. **Use `draft: true` for PRs that need human review**
5. **Set appropriate `timeout-minutes` to prevent runaway agents**
6. **Use `max:` limits on safe outputs to prevent spam**
7. **Use `cache-memory` tool for state tracking across runs**
8. **Reference upstream templates via `source:` field for updates**
9. **Set `expires:` on auto-created issues/PRs/discussions** so `agentics-maintenance.yml` can close them automatically

---

## Related Documentation

- [ADR-0001: Use Architectural Decision Records](../adr/0001-use-architectural-decision-records.md)
- [ADR-0003: Adopt Diátaxis Documentation Framework](../adr/0003-adopt-diataxis-documentation-framework.md)
- [CI Troubleshooting](../runbooks/CI-TROUBLESHOOTING.md)
- [Code Quality Metrics](CODE-QUALITY.md)

---

## See Also

- [GitHub Agentic Workflows Documentation](https://github.com/githubnext/agentics)
- [Diátaxis Documentation Framework](https://diataxis.fr/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
