---
diataxis_type: reference
---
# Dependabot Auto-Merge Workflow

## Overview

Automatically merges Dependabot pull requests for **patch** and **minor**
dependency updates, reducing the maintenance burden of keeping dependencies
current while preserving manual control over major version bumps.

**Workflow:** `.github/workflows/dependabot-automerge.yml`  
**Trigger:** Pull request opened, synchronised, or reopened — only when the
actor is `dependabot[bot]`  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Permissions:** `contents: write`, `pull-requests: write`

## Merge Policy

| Update type | Action |
|-------------|--------|
| `semver-patch` (e.g., 1.2.3 → 1.2.4) | Auto-merged (squash) |
| `semver-minor` (e.g., 1.2.x → 1.3.0) | Auto-merged (squash) |
| `semver-major` (e.g., 1.x.x → 2.0.0) | **Not** auto-merged — requires manual review |

## How It Works

1. The `dependabot/fetch-metadata` action reads the pull request and
   classifies the update type (`semver-patch`, `semver-minor`, or `semver-major`)
2. If the update type is `semver-patch` or `semver-minor`, the workflow calls
   `gh pr merge --auto --squash` to enable auto-merge
3. GitHub merges the PR automatically once all required status checks pass

The `--auto` flag means the merge is queued, not immediate. Required CI jobs
must succeed first.

## Configuring Dependabot

Dependabot is configured in `.github/dependabot.yml`. Key settings:

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
    labels:
      - dependencies
```

## Security Considerations

Auto-merging only activates when:
- The actor is literally `dependabot[bot]` (enforced by the `if:` condition)
- The update is patch or minor (major bumps always require human review)
- All branch protection checks pass

This prevents supply-chain attacks that might try to impersonate Dependabot
or introduce malicious code through a major version bump.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| PR not auto-merged | Major version update | Review manually — major bumps require human approval |
| PR not auto-merged | CI checks failing | Fix the failing checks first |
| `gh pr merge` fails | Insufficient token permissions | Verify `GITHUB_TOKEN` has `pull-requests: write` |
| Workflow not triggered | PR opened by human, not Dependabot | Correct — this workflow only handles Dependabot PRs |

See also: [Dependency Updates runbook](../runbooks/DEPENDENCY-UPDATES.md).
