# CI/CD Workflows Reference

Comprehensive guide to every GitHub Actions workflow included in the
`zircote/rust-template` repository. Workflows are organized by purpose and
annotated with trigger conditions, required secrets, and activation status.

---

## Overview Table

| Workflow | File | Trigger | Required Secrets | Status |
|---|---|---|---|---|
| CI | `ci.yml` | push, PR, manual | `CODECOV_TOKEN` | Active |
| Release | `release.yml` | tag `v*.*.*`, manual | -- | Active |
| Changelog | `changelog.yml` | tag `v*.*.*`, manual | -- | Active |
| Docker (GHCR) | `docker.yml` | push, PR, tag, manual | -- | Active |
| Publish to crates.io | `publish.yml` | manual | `CARGO_REGISTRY_TOKEN` | Opt-in |
| Security Audit | `security-audit.yml` | schedule (daily), push, manual | -- | Active |
| CodeQL Analysis | `codeql-analysis.yml` | push, PR, schedule (weekly), manual | -- | Active |
| Secrets Scan | `secrets-scan.yml` | push, PR, manual | `GITLEAKS_LICENSE` | Active |
| Container Scan | `container-scan.yml` | manual | -- | Opt-in |
| Benchmark | `benchmark.yml` | push, PR, manual | -- | Active |
| Benchmark Regression | `benchmark-regression.yml` | PR, manual | -- | Active |
| Code Coverage | `coverage.yml` | PR, manual | `CODECOV_TOKEN` | Active |
| Test Matrix | `test-matrix.yml` | PR, manual | -- | Active |
| Mutation Testing | `mutation-testing.yml` | PR (src/tests paths), manual | -- | Active |
| Fuzz Testing | `fuzz-testing.yml` | manual | -- | Opt-in |
| Code Quality Metrics | `code-quality.yml` | PR, manual | -- | Active |
| Spell Check | `spell-check.yml` | push, PR, manual | -- | Active |
| SBOM Generation | `sbom.yml` | tag `v*.*.*`, release, manual | -- | Active |
| Signed Releases | `signed-releases.yml` | release | -- | Active |
| SLSA Provenance | `slsa-provenance.yml` | release, manual | -- | Active |
| Dependabot Auto-Merge | `dependabot-automerge.yml` | PR (dependabot actor) | -- | Active |
| Stale Issue Management | `stale.yml` | manual | -- | Opt-in |
| Contributor Recognition | `contributors.yml` | manual | -- | Opt-in |
| Template Init | `template-init.yml` | push to main, manual | -- | Active |
| Nightly Builds | `nightly.yml` | manual | -- | Opt-in |
| Deploy Documentation | `docs-deploy.yml` | manual | -- | Opt-in |
| ADR Validation | `adr-validation.yml` | push, PR (docs/adr paths), manual | -- | Active |
| ADR Viewer | `adr-viewer.yml` | push (docs/adr paths), manual | -- | Active |
| Docker Hub Multi-Registry | `docker-hub.yml` | manual | `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN` | Opt-in |
| Homebrew Package | `package-homebrew.yml` | release, manual | `HOMEBREW_TAP_TOKEN` | Active |
| Snap Package | `package-snap.yml` | release, manual | `SNAPCRAFT_TOKEN` | Active |
| Windows MSI Installer | `package-windows.yml` | release, manual | -- | Active |
| Linux Packages (deb/rpm) | `package-linux.yml` | release, manual | -- | Active |
| Copilot Setup Steps | `copilot-setup-steps.yml` | manual | -- | Active |

> **"Active"** means the workflow has at least one automatic trigger (push, PR,
> schedule, release, or tag). **"Opt-in"** means only `workflow_dispatch`
> (manual) is enabled; automatic triggers are commented out and must be
> uncommented to activate.

---

## Core CI

### ci.yml

**What it does:** The primary quality gate for every change. Runs formatting,
linting, tests on three operating systems, documentation build, dependency
license/advisory checks (cargo-deny), MSRV verification, and code coverage
upload. A final `all-checks-pass` job gates merge readiness.

**Trigger:** Push to `main`/`master`, pull request to `main`/`master`, manual.

**Required secrets:** `CODECOV_TOKEN` (optional -- coverage upload fails
gracefully without it).

**How to enable/disable:** Active by default. To skip coverage, remove or
comment out the `coverage` job. To change the MSRV, update the toolchain
version in the `msrv` job (currently `1.92`).

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

### codeql-analysis.yml

**What it does:** Performs GitHub CodeQL static analysis on Rust code (via the
`cpp` extractor). Results surface in the repository Security tab.

**Trigger:** Push to `main`, pull request to `main`, weekly schedule (Monday
06:00 UTC), manual.

**Required secrets:** None.

**How to enable/disable:** Active by default. Remove the `schedule` trigger to
limit to push/PR only.

### secrets-scan.yml

**What it does:** Scans the repository history for accidentally committed
secrets using Gitleaks.

**Trigger:** Every push, every pull request, manual.

**Required secrets:** `GITLEAKS_LICENSE` (optional -- the action works without
it but may have rate limits).

**How to enable/disable:** Active by default. Remove or comment out the entire
file to disable.

### container-scan.yml

**What it does:** Builds the Docker image locally and scans it with Trivy for
OS and dependency vulnerabilities. SARIF results upload to GitHub Security;
a human-readable table is saved as an artifact.

**Trigger:** Manual only (push/PR/schedule triggers are commented out).

**Required secrets:** None.

**How to enable/disable:** Opt-in. Uncomment the `push`, `pull_request`,
and/or `schedule` triggers to activate automatic runs.

### sbom.yml

**What it does:** Generates a Software Bill of Materials in SPDX 2.3 JSON
format using `cargo-sbom`. On a published release, the SBOM is attached as a
release asset.

**Trigger:** Push tag `v*.*.*`, release published, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default on tags and releases.

### signed-releases.yml

**What it does:** Downloads all assets from a published GitHub release, signs
each with Sigstore Cosign (keyless OIDC), generates SHA-256 and SHA-512
checksum files, signs the checksums, and uploads everything back to the
release. Appends verification instructions to the release notes.

**Trigger:** Release published.

**Required secrets:** None (uses GitHub OIDC for keyless signing).

**How to enable/disable:** Active by default. Remove the file to disable
signing.

### slsa-provenance.yml

**What it does:** Builds the release binary, generates SLSA Level 3 provenance
attestation using the official `slsa-framework/slsa-github-generator`, and
uploads the provenance to the release.

**Trigger:** Release published, manual.

**Required secrets:** None (uses GitHub OIDC).

**How to enable/disable:** Active by default on releases.

---

## Testing & Quality

### benchmark.yml

**What it does:** Runs `cargo bench --workspace` and uploads Criterion results
as artifacts.

**Trigger:** Push to `main`/`master`, pull request to `main`/`master`, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default. Requires benchmarks in the
`benches/` directory.

### benchmark-regression.yml

**What it does:** Compares benchmark results against a cached baseline from
the main branch. Posts a performance report as a PR comment and flags
regressions. Updates the baseline cache on main-branch pushes.

**Trigger:** Pull request to `main`/`master`, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default on PRs.

### coverage.yml

**What it does:** Generates detailed code coverage reports with
`cargo-llvm-cov` in LCOV, HTML, and JSON formats. Uploads to Codecov, posts a
summary as a PR comment, and checks against an 80% coverage threshold.

**Trigger:** Pull request, manual (weekly schedule is commented out).

**Required secrets:** `CODECOV_TOKEN` (optional -- upload fails gracefully
without it).

**How to enable/disable:** Active on PRs by default. Uncomment the `schedule`
trigger for periodic coverage runs. Adjust the threshold in the
`Check coverage threshold` step.

### test-matrix.yml

**What it does:** Runs the full test suite across a matrix of operating systems
(Ubuntu, macOS, Windows) and Rust toolchains (stable, beta, nightly, MSRV
1.92). Includes integration tests, Miri undefined-behavior detection, and a
summary report.

**Trigger:** Pull request, manual (weekly schedule is commented out).

**Required secrets:** None.

**How to enable/disable:** Active on PRs by default. Uncomment the `schedule`
trigger for weekly runs. Adjust the matrix in the `strategy` block.

### mutation-testing.yml

**What it does:** Runs `cargo-mutants` to evaluate test suite effectiveness by
introducing source code mutations and checking whether tests catch them. Posts
results (including missed mutants) as a PR comment.

**Trigger:** Pull request when `src/**`, `tests/**`, `Cargo.toml`, or
`Cargo.lock` change; manual with optional target file input.

**Required secrets:** None.

**How to enable/disable:** Active by default on PRs that touch source or test
files.

### fuzz-testing.yml

**What it does:** Runs `cargo-fuzz` against all fuzz targets in `fuzz/fuzz_targets/`. If crashes are found, automatically opens a GitHub issue
with crash details. Caches the fuzz corpus between runs.

**Trigger:** Manual only (daily schedule is commented out). Accepts `duration`
(seconds) and `target` inputs.

**Required secrets:** None.

**How to enable/disable:** Opt-in. Uncomment the `schedule` trigger for daily
fuzzing. Requires fuzz targets in the `fuzz/` directory.

### code-quality.yml

**What it does:** Collects code quality metrics including unsafe code analysis
(`cargo-geiger`), binary size breakdown (`cargo-bloat`), and documentation
coverage. Uploads a combined report as an artifact.

**Trigger:** Pull request to `main`/`master`, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default on PRs.

### spell-check.yml

**What it does:** Checks spelling across all project files using
`crate-ci/typos`. Configured via `.typos.toml`. Runs with
`continue-on-error: true` so typos do not block merges.

**Trigger:** Push to `main`/`master`, pull request, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default. Remove `continue-on-error: true`
to make spelling errors blocking.

---

## Release & Publishing

### release.yml

**What it does:** Creates a GitHub Release with an auto-generated changelog
(via git-cliff), then builds release binaries for five targets: Linux
x86_64, Linux aarch64, macOS x86_64, macOS aarch64, and Windows x86_64.
Binaries are stripped, renamed with platform suffixes, and uploaded as release
assets.

**Trigger:** Push tag matching `v*.*.*`, manual.

**Required secrets:** None (uses built-in `GITHUB_TOKEN`).

**How to enable/disable:** Active by default. Adjust the `matrix.include`
block to add or remove build targets.

### changelog.yml

**What it does:** Generates a full `CHANGELOG.md` using git-cliff and commits
it to the `main` branch.

**Trigger:** Push tag matching `v*.*.*`, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default. Requires a `cliff.toml`
configuration file in the repository root.

### publish.yml

**What it does:** Publishes the crate to crates.io. Runs pre-publish checks
(format, clippy, tests, docs, cargo-deny) and a dry-run publish before the
actual publish step.

**Trigger:** Manual only (tag push trigger is commented out).

**Required secrets:** `CARGO_REGISTRY_TOKEN`.

**How to enable/disable:** Opt-in. Uncomment the `push: tags:` trigger and
configure `CARGO_REGISTRY_TOKEN` in repository secrets to enable automatic
publishing on tag push.

---

## Container & Packaging

### docker.yml

**What it does:** Builds a multi-platform Docker image (linux/amd64,
linux/arm64) and pushes it to GitHub Container Registry (ghcr.io). Uses GitHub
Actions cache for layer caching. Tags follow semver and include `latest` for
the default branch.

**Trigger:** Push to `main`/`master`, push tag `v*.*.*`, pull request to
`main`/`master` (build only, no push), manual.

**Required secrets:** None (uses built-in `GITHUB_TOKEN` for GHCR auth).

**How to enable/disable:** Active by default. Requires a `Dockerfile` in the
repository root.

### docker-hub.yml

**What it does:** Builds multi-platform Docker images and publishes to both
Docker Hub and GHCR simultaneously. Updates the Docker Hub repository
description from `README.md`.

**Trigger:** Manual only (tag push trigger is commented out).

**Required secrets:** `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN`.

**How to enable/disable:** Opt-in. Uncomment the `push: tags:` trigger and
configure Docker Hub secrets.

### package-homebrew.yml

**What it does:** Generates or updates a Homebrew formula in a separate tap
repository (`<owner>/homebrew-tap`) with the correct version, source URL, and
SHA-256 hash.

**Trigger:** Release published, manual (with version and dry-run inputs).

**Required secrets:** `HOMEBREW_TAP_TOKEN` (PAT with write access to the tap
repository).

**How to enable/disable:** Active on releases by default. Requires the
`<owner>/homebrew-tap` repository to exist.

### package-snap.yml

**What it does:** Builds a Snap package using `snapcraft` and optionally
publishes to the Snap Store. Attaches the `.snap` file to the GitHub release.

**Trigger:** Release published, manual.

**Required secrets:** `SNAPCRAFT_TOKEN` (required only for Snap Store
publishing).

**How to enable/disable:** Active on releases by default. Requires a
`snap/snapcraft.yaml` file.

### package-windows.yml

**What it does:** Builds a Windows MSI installer using `cargo-wix`. Attaches
the `.msi` file to the GitHub release.

**Trigger:** Release published, manual.

**Required secrets:** None.

**How to enable/disable:** Active on releases by default. Requires WiX
configuration (auto-generated by `cargo wix init`).

### package-linux.yml

**What it does:** Builds Debian (`.deb`) and RPM (`.rpm`) packages using
`cargo-deb` and `cargo-generate-rpm`. Attaches both to the GitHub release.

**Trigger:** Release published, manual (with version input).

**Required secrets:** None.

**How to enable/disable:** Active on releases by default. Requires
`[package.metadata.deb]` and `[package.metadata.generate-rpm]` sections in
`Cargo.toml`.

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

### template-init.yml

**What it does:** Automatically renames the project when a new repository is
created from this template. Replaces `zircote` with the new owner,
`rust-template` with the new repo name, and `rust_template` with the
corresponding crate name (underscored) across all non-workflow files. Becomes
a no-op once `Cargo.toml` no longer contains `name = "rust_template"`.

**Trigger:** Push to `main`, manual. Skipped entirely for the
`zircote/rust-template` repository itself.

**Required secrets:** None.

**How to enable/disable:** Active by default. Safe to delete after the initial
repository rename is complete.

### nightly.yml

**What it does:** Builds the project with the Rust nightly toolchain, packages
the binary as a tarball with a SHA-256 checksum, and creates a rolling
`nightly` pre-release on GitHub (replacing any previous nightly release).

**Trigger:** Manual only (daily 2 AM schedule is commented out).

**Required secrets:** None.

**How to enable/disable:** Opt-in. Uncomment the `schedule` trigger for daily
nightly builds.

---

## Documentation & ADRs

### docs-deploy.yml

**What it does:** Builds API documentation with `cargo doc` and optionally
builds an mdBook user guide (if `book.toml` exists). Creates a landing page
and deploys everything to GitHub Pages.

**Trigger:** Manual only (push and release triggers are commented out).

**Required secrets:** None (uses GitHub Pages OIDC).

**How to enable/disable:** Opt-in. Uncomment the `push` and/or `release`
triggers. Requires GitHub Pages to be enabled in repository settings with
source set to "GitHub Actions".

### adr-validation.yml

**What it does:** Validates Architecture Decision Records in `docs/adr/` using
[adrscope](https://github.com/zircote/adrscope). Checks formatting and
generates statistics. Posts validation results as a PR comment.

**Trigger:** Push to `main`/`master` and pull request when files in
`docs/adr/**` change, manual.

**Required secrets:** None.

**How to enable/disable:** Active by default on ADR file changes.

### adr-viewer.yml

**What it does:** Generates an HTML viewer for all ADRs using adrscope and
uploads it as a build artifact.

**Trigger:** Push to `main`/`master` when files in `docs/adr/**` change,
manual.

**Required secrets:** None.

**How to enable/disable:** Active by default on ADR file changes.

---

## AI Coding Agent

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
   with `#`).
3. Uncomment the desired triggers. For example, to enable nightly builds:

   ```yaml
   on:
     schedule:
       - cron: '0 2 * * *'
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
| `CODECOV_TOKEN` | `ci.yml`, `coverage.yml` | Upload coverage reports to Codecov | Optional |
| `CARGO_REGISTRY_TOKEN` | `publish.yml` | Authenticate with crates.io for publishing | Yes (if publishing) |
| `GITLEAKS_LICENSE` | `secrets-scan.yml` | Gitleaks commercial license key | Optional |
| `DOCKERHUB_USERNAME` | `docker-hub.yml` | Docker Hub account username | Yes (if using Docker Hub) |
| `DOCKERHUB_TOKEN` | `docker-hub.yml` | Docker Hub access token | Yes (if using Docker Hub) |
| `HOMEBREW_TAP_TOKEN` | `package-homebrew.yml` | PAT with write access to the homebrew-tap repo | Yes (if using Homebrew) |
| `SNAPCRAFT_TOKEN` | `package-snap.yml` | Snap Store credentials for publishing | Optional (build works without it) |

Configure secrets at **Settings > Secrets and variables > Actions > New
repository secret** in the GitHub repository settings.
