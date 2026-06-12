---
diataxis_type: reference
---
# Release Workflow

## Overview

Attested delivery: builds multi-platform binaries, shell completions, man
pages, and the MCPB bundle; attaches SLSA build provenance to every
artifact; binds them all to a CycloneDX SBOM attestation; then fail-closed
verifies every attestation **before** the GitHub Release is published. A
tag publishes nothing unattested. The same tag fans out to downstream
workflows that build and push Docker images, publish the crate to
crates.io, update the Homebrew tap, and back-merge `main` into `develop`
(see [Downstream Workflows](#downstream-workflows)).

**Workflow:** `.github/workflows/release.yml`
**Trigger:** Push of a `v*.*.*` tag, manual (`workflow_dispatch` = dry run)
**Required secrets:** `HOMEBREW_TAP_TOKEN` (PAT for release creation and
downstream propagation)
**Environment:** `copilot` (required for the `release` job)

## Release Pipeline Overview

```
tag v*.*.*  (or workflow_dispatch for a publish-skipped dry run)
    │
    ├─ release.yml
    │     version ──┬─ build (5-platform matrix, attested)──┐
    │               ├─ extras (completions + man, attested) ├─ sbom ─ verify
    │               └─ mcpb  (MCPB bundle, attested) ───────┘     (fail-
    │     test ──────────────────────────────────────────┐         closed)
    │     audit ─────────────────────────────────────────┤            │
    │                                                    └──────── release
    │                                                          (tag-gated)
    ├─ docker.yml    (on: tag)      Builds + pushes the GHCR image
    ├─ publish.yml   (on: tag)      Publishes the crate to crates.io (OIDC)
    ├─ back-merge.yml (on: tag)     Back-merges main into develop
    │
    └─ after the release publishes
          └─ package-homebrew.yml   Updates the Homebrew tap formulae

changelog.yml also runs on the tag and opens a CHANGELOG PR into develop.
```

## Jobs

### `version`

Resolves the release version: the tag minus the leading `v` on tag pushes,
or the `Cargo.toml` version suffixed `-dev` on `workflow_dispatch`
(dry run).

### `test` / `audit`

Tags are not guaranteed to point at CI-green commits, so the release
pipeline re-runs `cargo test --all-features --locked` and
`cargo audit` itself. The release publishes nothing untested.

### `build`

Builds release binaries for five targets using a matrix strategy:

| Target | Runner | Binary | Platform suffix |
|--------|--------|--------|-----------------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `nsip` | `linux-amd64` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` (native) | `nsip` | `linux-arm64` |
| `x86_64-apple-darwin` | `macos-latest` | `nsip` | `macos-amd64` |
| `aarch64-apple-darwin` | `macos-latest` | `nsip` | `macos-arm64` |
| `x86_64-pc-windows-msvc` | `windows-2022` | `nsip.exe` | `windows-amd64.exe` |

Each binary is built with `--locked`, stripped (Unix), staged as
`nsip-{version}-{platform}`, attested with
`actions/attest-build-provenance`, and uploaded as a workflow artifact.
Nothing touches the release at this stage.

### `extras`

Builds the binary once and generates supplementary artifacts:

- **Shell completions** (bash, zsh, fish, PowerShell):
  `nsip-{version}-completions.tar.gz`
- **Man pages** (`nsip man-pages`): `nsip-{version}-man-pages.tar.gz`

Both archives are provenance-attested and uploaded as workflow artifacts.

### `mcpb`

Downloads the five platform-binary artifacts, restores their unversioned
`server/nsip-<platform>` names (the names `manifest.json` expects), injects
the version into `manifest.json`, packages `nsip-{version}.mcpb`, attests
it, and uploads it as a workflow artifact.

### `sbom`

Generates a CycloneDX SBOM from the source tree (`anchore/sbom-action`)
and binds **every** artifact to it with `actions/attest-sbom` — so each
binary, archive, and bundle carries both a provenance attestation and an
SBOM attestation. Uploads `nsip-{version}-sbom.cdx.json` as an artifact.

### `verify` (fail-closed gate)

Downloads all 9 artifacts (5 binaries + 2 archives + MCPB + SBOM), asserts
the count, and runs `gh attestation verify` on each artifact for **both**
predicates (build provenance and `https://cyclonedx.org/bom`). Any failure
— including a missing artifact — blocks the release.

### `release` (tag-gated)

Runs only on tag refs and only after `verify`, `test`, and `audit` pass:

1. Generates `nsip-{version}-checksums.txt` (SHA-256 over all assets)
2. Generates the changelog body with `git-cliff` (`cliff.toml`)
3. Creates the GitHub Release with all artifacts attached (marked
   pre-release for `-alpha`/`-beta`/`-rc` versions)

The release is created using the `HOMEBREW_TAP_TOKEN` PAT so that the
`release`/`workflow_run` events propagate to downstream workflows
(bot-authored events from `GITHUB_TOKEN` do not trigger workflows).

## Dry Runs

`workflow_dispatch` from any branch exercises the full
build → attest → verify chain with a `-dev` version; the `release` job is
tag-gated and skipped. Use this to validate pipeline changes without
cutting a version.

## Triggering a Release

```bash
# 1. On develop: ensure CHANGELOG.md and the Cargo.toml version are up to date
# 2. Promote develop -> main via the Release PR workflow and merge it
# 3. Tag the main merge commit and push the single tag
git checkout main && git pull origin main
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
```

See the [Releasing runbook](../runbooks/RELEASING.md) for the full
promote-tag-push flow and step-by-step release checklist.

## Downstream Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `docker.yml` | tag push `v*.*.*` | Builds and pushes Docker image to GHCR |
| `publish.yml` | tag push `v*.*.*` | Publishes the crate to crates.io via Trusted Publishing (OIDC), byte-verifies the registry copy, attests it |
| `package-homebrew.yml` | `workflow_run` after Release / `release` published | Updates `nsip.rb` (binary) and `nsip-source.rb` in `zircote/homebrew-tap` |
| `back-merge.yml` | tag push `v*.*.*` | Back-merges `main` into `develop` to keep them in sync |
| `changelog.yml` | tag push `v*.*.*` | Opens a PR to update `CHANGELOG.md` |
| `sbom.yml` | manual (`workflow_dispatch`) | On-demand SPDX SBOM for inspection (workflow artifact only). The release SBOM is the attested CycloneDX SBOM from `release.yml` |

## Verification

See [Release Attestations & Verification](../security/SIGNED-RELEASES.md)
for the consumer-side `gh attestation verify` commands.

## Changelog Generation

The release body is generated by `git-cliff` using the `cliff.toml`
configuration. Commit types are mapped to changelog sections:

| Commit prefix | Section |
|--------------|---------|
| `feat:` | Features |
| `fix:` | Bug Fixes |
| `perf:` | Performance |
| `docs:` | Documentation |
| `chore:` | Miscellaneous |

See `cliff.toml` for the full configuration.

## Permissions

Top-level `contents: read`; elevated per job:

| Job | Permissions | Reason |
|-----|------------|--------|
| `build` / `extras` / `mcpb` / `sbom` | `id-token: write`, `attestations: write` | Keyless signing + attestation storage |
| `verify` | `attestations: read` | `gh attestation verify` |
| `release` | `contents: write` | Create release, upload assets |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Release not created | `verify`, `test`, or `audit` failed | Fix the cause; the gate is intentional — never bypass it |
| `expected 9 artifacts` error | A build/extras/mcpb job failed or uploaded nothing | Check the failed job; `if-no-files-found: error` names it |
| Release not created (jobs green) | `HOMEBREW_TAP_TOKEN` expired or missing | Rotate the PAT in the `copilot` environment |
| Changelog is empty | No commits since last tag | Verify commit history and `cliff.toml` patterns |
| Downstream workflows not triggered | Release created with `GITHUB_TOKEN` instead of the PAT | Ensure the `release` job uses `HOMEBREW_TAP_TOKEN` |

See also: [Releasing runbook](../runbooks/RELEASING.md).
