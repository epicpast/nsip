---
diataxis_type: reference
---
# CI/CD Workflows Reference

Comprehensive guide to every GitHub Actions workflow included in the
`zircote/nsip` repository. Workflows are organized by purpose and
annotated with trigger conditions, required secrets, and activation status.

---

## Overview Table

| Workflow | File | Trigger | Required Secrets | Status |
|---|---|---|---|---|
| CI | `ci.yml` | push, PR (`develop`/`main`), manual | `CODECOV_TOKEN` | Active |
| Release | `release.yml` | tag `v*.*.*`, manual (dry run) | `HOMEBREW_TAP_TOKEN` | Active |
| Release PR | `release-pr.yml` | manual | -- | Active |
| Back-merge | `back-merge.yml` | tag `v*.*.*`, manual | -- | Active |
| Changelog | `changelog.yml` | tag `v*.*.*`, manual | -- | Active |
| Docker (GHCR) | `docker.yml` | tag `v*.*.*`, manual | -- | Active |
| Publish to crates.io | `publish.yml` | tag `v*.*.*`, manual | -- (Trusted Publishing OIDC) | Active |
| Homebrew Package | `package-homebrew.yml` | workflow_run (after Release), release, manual | `HOMEBREW_TAP_TOKEN` | Active |
| Security Audit | `security-audit.yml` | schedule (daily), push, manual | -- | Active |
| Secrets Scan | `secrets-scan.yml` | manual | `GITLEAKS_LICENSE` | Opt-in |
| Container Scan | `container-scan.yml` | manual | -- | Opt-in |
| SBOM Generation | `sbom.yml` | manual (`workflow_dispatch`) | -- | Active |
| Dependabot Auto-Merge | `dependabot-automerge.yml` | PR (dependabot actor) | -- | Active |
| Stale Issue Management | `stale.yml` | manual | -- | Opt-in |
| Contributor Recognition | `contributors.yml` | manual | -- | Opt-in |
| Agentic Triage | `issue-triage.lock.yml` | issues, manual | -- | Opt-in |
| Copilot Setup Steps | `copilot-setup-steps.yml` | manual | -- | Active |

> **"Active"** means the workflow has at least one automatic trigger (push, PR,
> schedule, release, or tag). **"Opt-in"** means only `workflow_dispatch`
> (manual) is enabled; automatic triggers are commented out and must be
> uncommented to activate, or the workflow is disabled at the repository level.

---

## Core CI

### ci.yml

**What it does:** The primary quality gate for every change. Runs formatting,
linting, tests on three operating systems, documentation build, dependency
license/advisory checks (cargo-deny), MSRV verification, code coverage
upload, and a `pin-check` (central reusable workflow asserting every
`uses:` is pinned to a full commit SHA). A final `all-checks-pass` job gates
merge readiness.

**Trigger:** Push to `develop`/`main`, pull request to `develop`/`main`, manual.

**Required secrets:** `CODECOV_TOKEN` (optional -- coverage upload fails
gracefully without it).

**How to enable/disable:** Active by default. To change the MSRV, update the
toolchain version in the `msrv` job (currently `1.92`).

---

## Security & Compliance

### security-audit.yml

**What it does:** Runs `cargo audit` against the RustSec advisory database to
detect known vulnerabilities in dependencies.

**Trigger:** Daily schedule at 00:00 UTC, push when `Cargo.toml` or
`Cargo.lock` change, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default. Adjust the cron expression to
change frequency or remove the `schedule` trigger to run only on push/manual.

### secrets-scan.yml

**What it does:** Scans the repository history for accidentally committed
secrets using Gitleaks.

**Trigger:** Manual only (`workflow_dispatch`); the `push` and `pull_request`
triggers are commented out.

**Required secrets:** `GITLEAKS_LICENSE` (optional -- the action works without
it but may have rate limits).

**How to enable/disable:** Opt-in. Uncomment the `push` and `pull_request`
triggers under `on:` to scan automatically.

### container-scan.yml

**What it does:** Builds the Docker image locally and scans it with Trivy for
OS and dependency vulnerabilities. SARIF results upload to GitHub Security;
a human-readable table is saved as an artifact.

**Trigger:** Manual only (push/PR/schedule triggers are commented out).

**Required secrets:** None.

**How to enable/disable:** Opt-in. Uncomment the `push`, `pull_request`,
and/or `schedule` triggers to activate automatic runs.

### sbom.yml

**What it does:** Generates an on-demand Software Bill of Materials in SPDX
2.3 JSON format using `cargo-sbom` (workflow artifact only). The
authoritative release SBOM is the attested CycloneDX SBOM produced by
`release.yml`.

**Trigger:** Manual (`workflow_dispatch`).

**Required secrets:** None.

**How to enable/disable:** Active by default (manual trigger only).

---

## Release & Publishing

The repository follows a `develop`-based branching model: all development
happens on `develop`, releases are promoted to `main` via a release PR, and the
`main` merge commit is tagged `vX.Y.Z` to trigger release automation. See the
[Releasing runbook](../runbooks/RELEASING.md) for the full procedure.

### release.yml

**What it does:** Attested delivery. Builds release binaries for five
targets (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64), shell
completions, man pages, and the MCPB bundle; attests every artifact with
SLSA build provenance; binds them all to a CycloneDX SBOM attestation; then
fail-closed verifies every attestation before creating the GitHub Release
with an auto-generated changelog (git-cliff) and a SHA-256 checksums file.
The release job is tag-gated, so a `workflow_dispatch` run is a publish-free
dry run of the whole chain.

**Trigger:** Push tag matching `v*.*.*`, manual (dry run).

**Required secrets:** `HOMEBREW_TAP_TOKEN` (PAT, so the release event
propagates to downstream workflows).

**How to enable/disable:** Active by default. Adjust the `matrix.include`
block to add or remove build targets (and update the artifact count in the
`verify` job).

### release-pr.yml

**What it does:** Opens (or updates) a release promotion pull request from
`develop` into `main` -- the first step of the release process.

**Trigger:** Manual (`workflow_dispatch`), with an optional `version` input
used in the PR title/body.

**Required secrets:** None (uses built-in `GITHUB_TOKEN`).

**How to enable/disable:** Active by default. After merging the PR it opens,
tag the `main` merge commit to trigger the release.

### back-merge.yml

**What it does:** Keeps `develop` in sync with `main` after a release. Opens (or
reuses) a pull request back-merging `main` into `develop` and enables auto-merge
so it lands once required checks pass. Skips when `develop` already contains
`main`.

**Trigger:** Push tag matching `v*.*.*` (fires alongside the release), manual.

**Required secrets:** None (uses built-in `GITHUB_TOKEN`).

**How to enable/disable:** Active by default. Requires auto-merge to be enabled
in repository settings for the PR to land automatically.

### changelog.yml

**What it does:** Generates a full `CHANGELOG.md` using git-cliff and opens a
pull request to merge it into the `develop` branch.

**Trigger:** Push tag matching `v*.*.*`, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default. Requires a `cliff.toml`
configuration file in the repository root.

### publish.yml

**What it does:** Publishes the crate to crates.io via Trusted Publishing
(OIDC -- no long-lived registry token). Runs pre-publish checks (format,
clippy, tests, docs, cargo-deny) and a dry-run publish first; after
publishing, downloads the `.crate` the registry serves, asserts it is
byte-identical to the locally packaged crate, and attaches SLSA build
provenance to it.

**Trigger:** Push tag matching `v*.*.*`, manual (publish steps are
tag-gated).

**Required secrets:** None -- Trusted Publishing is configured on crates.io
(crate Settings → Trusted Publishing → repo `zircote/nsip`, workflow
`publish.yml`, environment `copilot`).

**How to enable/disable:** Active by default.

### package-homebrew.yml

**What it does:** Regenerates both Homebrew formulae in
`zircote/homebrew-tap` after a release publishes: `nsip.rb` (pre-built
binaries for macOS arm64/x86_64 and Linux x86_64, with completions and man
pages) and `nsip-source.rb` (build from the source tarball). Computes
SHA-256 digests from the published release assets and pushes the updated
formulae to the tap. Idempotent -- a second firing for the same version is
a no-op.

**Trigger:** `workflow_run` after the **Release** workflow completes
(successful tag runs only), `release` published, manual
(`workflow_dispatch` with a `dry_run` input that prints the formulae
without pushing).

**Required secrets:** `HOMEBREW_TAP_TOKEN` (PAT scoped to the tap repo).

**How to enable/disable:** Active by default.

---

## Container

### docker.yml

**What it does:** Builds a multi-platform Docker image (linux/amd64,
linux/arm64) and pushes it to GitHub Container Registry (ghcr.io). Uses GitHub
Actions cache for layer caching. Tags follow semver (`X.Y.Z`, `X.Y`, `X`),
include a `sha-<commit>` tag, and `latest` for the newest released version.
Attaches SLSA build provenance to the image.

**Trigger:** Push tag `v*.*.*`, manual (`workflow_dispatch`).

**Required secrets:** None (uses built-in `GITHUB_TOKEN` for GHCR auth).

**How to enable/disable:** Active by default. Requires a `Dockerfile` in the
repository root.

---

## Maintenance & Automation

### dependabot-automerge.yml

**What it does:** Automatically enables auto-merge (squash) for Dependabot PRs
that are patch or minor version updates. Major version updates are left for
manual review.

**Trigger:** Pull request opened, synchronized, or reopened (runs only when the
actor is `dependabot[bot]`).

**Required secrets:** None (uses built-in `GITHUB_TOKEN`).

**How to enable/disable:** Active by default. Requires branch protection rules
and auto-merge to be enabled in repository settings.

### stale.yml

**What it does:** Marks issues as stale after 60 days of inactivity (closed
after 14 more days) and PRs as stale after 30 days (closed after 7 more days).
Exempts issues labeled `pinned`, `security`, or `good first issue`, and PRs
labeled `pinned` or `work-in-progress`.

**Trigger:** Manual only (daily schedule is commented out).

**Required secrets:** None.

**How to enable/disable:** Opt-in. Uncomment the `schedule` trigger to enable
daily runs. Adjust day counts and exempt labels as needed.

### contributors.yml

**What it does:** Generates a `CONTRIBUTORS.md` file from git history with
contributor names, total commit counts, and project statistics. Commits
changes back to the repository.

**Trigger:** Manual only (monthly schedule is commented out).

**Required secrets:** None.

**How to enable/disable:** Opt-in. Uncomment the `schedule` trigger for
monthly updates.

---

## AI Coding Agent

### issue-triage.lock.yml

**What it does:** An agentic (gh-aw) workflow that triages new GitHub issues --
labels, summarizes, and routes them. The `.lock.yml` file is generated from
`issue-triage.md`; edit the `.md` source and recompile with `gh aw compile`.

**Trigger:** Issue events, manual.

**Required secrets:** Configured by the gh-aw engine (e.g. a Copilot or model
token).

**How to enable/disable:** Disabled at the repository level by default. Enable
in **Actions** to activate. Never hand-edit the `.lock.yml`.

### copilot-setup-steps.yml

**What it does:** Prepares the CI environment for GitHub Copilot coding agent
sessions. Installs the Rust stable toolchain with clippy and rustfmt, caches
the cargo registry, installs cargo-deny, and pre-fetches all dependencies.

**Trigger:** Manual (`workflow_dispatch` only -- invoked automatically by
Copilot when it needs a coding environment).

**Required secrets:** None.

**How to enable/disable:** Active by default (manual trigger only). Remove the
file to prevent Copilot from using this environment setup.

---

## Enabling/Disabling Workflows

### Activating an opt-in workflow

1. Open the workflow file in `.github/workflows/`.
2. Locate the commented-out triggers under the `on:` key (lines prefixed
   with `#`), or re-enable the workflow in **Actions** if it is disabled at the
   repository level.
3. Uncomment the desired triggers. For example, to enable the stale bot:

   ```yaml
   on:
     schedule:
       - cron: '0 0 * * *'
     workflow_dispatch:
   ```

4. If the workflow requires secrets, configure them in **Settings > Secrets
   and variables > Actions** before the first run.
5. Commit and push the change.

### Disabling an active workflow

**Option A -- Comment out triggers:** Replace automatic triggers with only
`workflow_dispatch:` so the workflow can still be run manually but will not
trigger automatically.

**Option B -- Delete the file:** Remove the workflow YAML file from
`.github/workflows/` entirely. The workflow will stop appearing in the
Actions tab.

**Option C -- Disable via GitHub UI:** Navigate to **Actions > (workflow
name) > ... > Disable workflow**. This preserves the file but prevents all
runs.

### Running any workflow manually

Every workflow includes a `workflow_dispatch` trigger. Navigate to
**Actions > (workflow name) > Run workflow** in the GitHub UI to start a
manual run on any branch.

---

## Required Secrets Summary

The table below lists every secret referenced across all workflows. Secrets
marked "built-in" are provided automatically by GitHub Actions and require no
configuration.

| Secret | Used By | Purpose | Required |
|---|---|---|---|
| `GITHUB_TOKEN` | multiple | GitHub API access (releases, PRs, packages) | Built-in |
| `CODECOV_TOKEN` | `ci.yml` | Upload coverage reports to Codecov | Optional |
| `HOMEBREW_TAP_TOKEN` | `release.yml`, `package-homebrew.yml` | Create the release with a PAT (event propagation) and push tap formulae | Yes |
| `GITLEAKS_LICENSE` | `secrets-scan.yml` | Gitleaks commercial license key | Optional |

crates.io publishing uses Trusted Publishing (OIDC) and needs no secret.

Configure secrets at **Settings > Secrets and variables > Actions > New
repository secret** in the GitHub repository settings.
