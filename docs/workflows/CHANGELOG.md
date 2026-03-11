---
diataxis_type: reference
---
# Changelog Workflow

## Overview

Automatically generates or updates `CHANGELOG.md` using `git-cliff` whenever a
version tag is pushed, then opens (or updates) a pull request to merge the
change into `main`. Can also be triggered manually for out-of-band changelog
updates.

**Workflow:** `.github/workflows/changelog.yml`  
**Trigger:** Push of a `v*.*.*` tag, manual (`workflow_dispatch`)  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Permissions:** `contents: write`, `pull-requests: write`

## How It Works

1. **Checkout** — fetches full history (`fetch-depth: 0`) so `git-cliff` can
   walk all commits
2. **Generate** — runs `git-cliff` with `cliff.toml` and writes the result to
   `CHANGELOG.md`
3. **Diff check** — skips the rest if `CHANGELOG.md` was not modified
4. **Branch** — creates (or reuses) branch `chore/changelog-<tag>`
5. **Commit** — commits the updated file as `github-actions[bot]`
6. **Pull request** — opens a PR titled `docs: update CHANGELOG.md for <tag>`
   or updates the existing open PR for that branch

## Changelog Format

Commits are grouped by conventional commit type (from `cliff.toml`):

| Commit prefix | Changelog section |
|--------------|-------------------|
| `feat:` | Features |
| `fix:` | Bug Fixes |
| `perf:` | Performance |
| `docs:` | Documentation |
| `chore:` | Miscellaneous |
| `refactor:` | Refactoring |

Commits without a recognised prefix are not included in the changelog.

## Running Locally

```bash
# Install git-cliff
cargo install git-cliff

# Preview changelog for the next version
git cliff --config cliff.toml --unreleased

# Generate the full changelog
git cliff --config cliff.toml -o CHANGELOG.md

# Generate changelog for a specific tag range
git cliff --config cliff.toml v0.3.0..v0.4.0
```

## Idempotency

The workflow is safe to re-run. If branch `chore/changelog-<tag>` already
exists on `origin` and an open PR for that branch is already present, the
workflow reuses both — it will only commit and push if `CHANGELOG.md` contains
new changes.

## Relationship to Release Workflow

The [Release workflow](RELEASE.md) generates the release body using `git-cliff`
at release time. This workflow serves as a separate automation to keep
`CHANGELOG.md` in the repository up to date. Both use the same `cliff.toml`
configuration.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| No PR created | `CHANGELOG.md` unchanged | Verify commits follow conventional commit prefixes |
| PR title refers to wrong tag | Workflow triggered manually without correct ref | Use `workflow_dispatch` and tag the ref correctly |
| Push fails | Branch protection on the changelog branch | Adjust branch protection rules or use `GITHUB_TOKEN` with write permission |
| Empty changelog | No commits since last tag or `cliff.toml` filters them out | Check `cliff.toml` commit filter patterns |
