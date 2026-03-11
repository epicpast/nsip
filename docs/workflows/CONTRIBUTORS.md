---
diataxis_type: reference
---
# Contributor Recognition Workflow

## Overview

Generates a `CONTRIBUTORS.md` file that lists every unique author from the
commit history, together with aggregate repository statistics. Designed to be
run on demand or on a monthly schedule (schedule is disabled by default).

**Workflow:** `.github/workflows/contributors.yml`  
**Trigger:** Manual (`workflow_dispatch`); monthly schedule is commented out  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Permissions:** `contents: write`

## What It Produces

The workflow creates or overwrites `CONTRIBUTORS.md` with:

- A deduplicated list of contributor names and email addresses, sorted
  alphabetically, excluding bots (`github-actions`, `dependabot`)
- A **Statistics** section containing:
  - Total unique contributor count
  - Total commit count
  - Date of the first commit
  - Timestamp of the last update

## Enabling the Monthly Schedule

The monthly schedule is disabled by default. To enable it, uncomment the
`schedule` block in `.github/workflows/contributors.yml`:

```yaml
on:
  schedule:
    - cron: '0 0 1 * *'  # 1st of every month at 00:00 UTC
  workflow_dispatch:
```

## Running Locally

```bash
# Generate contributors list manually
echo "# Contributors" > CONTRIBUTORS.md
echo "" >> CONTRIBUTORS.md
echo "Thank you to all our contributors!" >> CONTRIBUTORS.md
echo "" >> CONTRIBUTORS.md
git log --format='%aN <%aE>' | sort -u | \
  grep -v 'github-actions\|dependabot' | \
  while read line; do echo "- $line" >> CONTRIBUTORS.md; done
```

## Commit Behaviour

The workflow only commits and pushes if `CONTRIBUTORS.md` actually changed.
The commit message includes `[skip ci]` to prevent a CI feedback loop.

## Permissions

| Permission | Scope | Reason |
|-----------|-------|--------|
| `contents: write` | Repository | Commit and push `CONTRIBUTORS.md` |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| File not updated | No new commits since last run | Run manually to force regeneration |
| Bot names appear in list | `grep` filter too narrow | Extend the `grep -v` pattern in the workflow |
| Push fails | Branch protection on `main` | Run the workflow from a fork or exempt the bot in branch protection |
