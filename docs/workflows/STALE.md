---
diataxis_type: reference
---
# Stale Issue Management Workflow

## Overview

Automatically marks inactive issues and pull requests as stale, then closes
them if no further activity occurs. Reduces the maintenance burden of an
accumulating backlog. The daily schedule is disabled by default.

**Workflow:** `.github/workflows/stale.yml`  
**Trigger:** Manual (`workflow_dispatch`) — daily schedule is commented out  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Permissions:** `issues: write`, `pull-requests: write`

## Staleness Thresholds

| Item | Days before stale | Days before close |
|------|-------------------|-------------------|
| Issues | 60 | 14 |
| Pull requests | 30 | 7 |

## Labels

| Label | Applied to | Meaning |
|-------|-----------|---------|
| `stale` | Issues and PRs | Item has not had recent activity |

The `stale` label must exist in the repository's label list. Create it if
missing:

```bash
gh label create stale --color e4e669 --description "No recent activity"
```

## Exemptions

Items with these labels are **never** marked stale:

| Label | Exempts |
|-------|---------|
| `pinned` | Issues and PRs |
| `security` | Issues only |
| `good first issue` | Issues only |
| `work-in-progress` | PRs only |

## Enabling the Daily Schedule

The daily schedule is disabled by default. To enable it, uncomment the
`schedule` block in `.github/workflows/stale.yml`:

```yaml
on:
  schedule:
    - cron: '0 0 * * *'  # 00:00 UTC daily
  workflow_dispatch:
```

## Stale Messages

**Issue stale message:**

> This issue has been automatically marked as stale because it has not had
> recent activity. It will be closed if no further activity occurs within 14 days.
> If this issue is still relevant, please add a comment to keep it open.

**PR stale message:**

> This PR has been automatically marked as stale because it has not had
> recent activity. It will be closed if no further activity occurs within 7 days.

## Reopening Closed Items

Stale-closed items can always be reopened. Add a comment explaining why the
issue or PR is still relevant and remove the `stale` label.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Items not marked stale | Schedule disabled | Enable the schedule or run manually |
| Security issues closed | `security` label not applied | Apply the `security` label to prevent auto-closure |
| WIP PRs closed | `work-in-progress` label not applied | Apply the exemption label |
| `stale` label missing | Label not created in repository | Create the label with `gh label create stale` |
