---
diataxis_type: reference
---
# Release Workflow

## Overview

Automates the full release process: creates a GitHub Release with a
generated changelog, builds multi-platform binaries, generates shell
completions and man pages, produces a Software Bill of Materials (SBOM),
and packages the MCPB bundle — all triggered by a version tag. The same
tag fans out to downstream workflows that build and push Docker images,
publish the crate to crates.io, and back-merge `main` into `develop`
(see [Downstream Workflows](#downstream-workflows)).

**Workflow:** `.github/workflows/release.yml`  
**Trigger:** Push of a `v*.*.*` tag, manual (`workflow_dispatch`)  
**Required secrets:** `HOMEBREW_TAP_TOKEN` (PAT for release creation and
downstream propagation)  
**Environment:** `copilot` (required for the `create-release` job)

## Release Pipeline Overview

```
tag v*.*.*
    │
    ├─ release.yml (on: tag)
    │       ├─ create-release       Creates the GitHub Release
    │       ├─ generate-extras      Builds completions + man pages
    │       ├─ generate-sbom        Generates the SBOM
    │       ├─ build-binaries       Builds multi-platform binaries
    │       └─ package-mcpb         Packages the MCPB bundle
    │
    ├─ docker.yml   (on: tag)       Builds + pushes the GHCR image
    ├─ publish.yml  (on: tag)       Publishes the crate to crates.io
    ├─ back-merge.yml (on: tag)     Back-merges main into develop
    │
    └─ (after the above)
            │
            ├─ on: release.published ─→ sbom.yml, slsa-provenance.yml
            └─ on: workflow_run(Release completed) ─→ signed-releases.yml

changelog.yml also runs on the tag and opens a CHANGELOG PR into develop.
```

## Jobs

### `create-release`

Creates the GitHub Release:

1. Checks out the full history (`fetch-depth: 0`) for changelog generation
2. Extracts the version from the git tag (strips the `v` prefix)
3. Generates a changelog with `git-cliff` using `cliff.toml`
4. Creates the GitHub Release (published immediately — `draft: false`;
   marked pre-release for `-alpha`, `-beta`, `-rc` suffixes)

The release is created using the `HOMEBREW_TAP_TOKEN` PAT so that the
`release` event propagates to downstream workflows (Homebrew, SBOM).

**Pre-release detection:** A release is marked as pre-release if the version
contains `-alpha`, `-beta`, or `-rc`.

### `generate-extras`

Builds the release binary and generates supplementary artifacts:

- **Shell completions**: bash, zsh, fish, PowerShell  
- **Man pages**: generated via `nsip man-pages --out-dir man`

Both archives are attested with GitHub's `attest-build-provenance` action,
producing `.sigstore.json` bundle files. All four files are uploaded to the
release. Archive names carry the release version (`{version}` is the tag minus
the leading `v`, e.g. `0.6.0`).

**Artifacts uploaded:**
- `nsip-{version}-completions.tar.gz` + `.sigstore.json`
- `nsip-{version}-man-pages.tar.gz` + `.sigstore.json`

### `generate-sbom`

Generates an SPDX Software Bill of Materials:

1. Installs `cargo-sbom`
2. Generates SPDX 2.3 JSON via `cargo sbom --output-format spdx_json_2_3`
3. Attests the SBOM with build provenance
4. Uploads `nsip-{version}-sbom-spdx.json` + `.sigstore.json` to the release

### `build-binaries`

Builds release binaries for five targets using a matrix strategy:

| Target | Runner | Binary | Platform suffix |
|--------|--------|--------|-----------------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `nsip` | `linux-amd64` |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `nsip` (cross-compiled) | `linux-arm64` |
| `x86_64-apple-darwin` | `macos-latest` | `nsip` | `macos-amd64` |
| `aarch64-apple-darwin` | `macos-latest` | `nsip` | `macos-arm64` |
| `x86_64-pc-windows-msvc` | `windows-2022` | `nsip.exe` | `windows-amd64.exe` |

Each binary is:
1. Built with `cargo build --release --target <target>`
2. Stripped (Unix targets) and renamed to `nsip-{version}-{platform}` — the
   binary is uploaded as a bare renamed file, not a `.tar.gz`/`.zip` archive
3. Attested with build provenance, producing a `.sigstore.json` bundle
4. Uploaded to the GitHub Release alongside its `.sigstore.json`

**Artifacts uploaded** (`{version}` is the tag minus the leading `v`):
- `nsip-{version}-linux-amd64` + `.sigstore.json`
- `nsip-{version}-linux-arm64` + `.sigstore.json`
- `nsip-{version}-macos-amd64` + `.sigstore.json`
- `nsip-{version}-macos-arm64` + `.sigstore.json`
- `nsip-{version}-windows-amd64.exe` + `.sigstore.json`

### `package-mcpb`

Downloads the five versioned platform binaries from the release, restores their
unversioned `server/nsip-<platform>` names (the names `manifest.json` expects),
injects the version into `manifest.json`, and packages the MCPB bundle as
`nsip-{version}.mcpb`. The bundle is attested with build provenance
(`.sigstore.json`) and uploaded to the release.

## Triggering a Release

```bash
# 1. On develop: ensure CHANGELOG.md and the Cargo.toml version are up to date
# 2. Promote develop -> main via a release PR (or the Release PR workflow) and
#    merge it
# 3. Tag the main merge commit and push the single tag
git checkout main && git pull origin main
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
```

This triggers the release workflow automatically. See the
[Releasing runbook](../runbooks/RELEASING.md) for the full promote-tag-push flow
and step-by-step release checklist.

## Downstream Workflows

Pushing the tag (and the release it creates) fans out to several workflows,
each on its own trigger:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `docker.yml` | tag push `v*.*.*` | Builds and pushes Docker image to GHCR |
| `publish.yml` | tag push `v*.*.*` | Publishes the crate to crates.io |
| `back-merge.yml` | tag push `v*.*.*` | Back-merges `main` into `develop` to keep them in sync |
| `changelog.yml` | tag push `v*.*.*` | Opens a PR to update `CHANGELOG.md` |
| `sbom.yml` | `release` published | Attaches the SPDX SBOM to the release |
| `slsa-provenance.yml` | `release` published | Generates SLSA provenance attestations |
| `signed-releases.yml` | `workflow_run` after Release | Signs release binaries with Sigstore/Cosign |

> Package-manager workflows (Homebrew, Snap, Linux `.deb`/`.rpm`, Windows MSI)
> are not currently included; re-add them and wire them to the `release` event
> to attach those artifacts.

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

| Permission | Scope | Reason |
|-----------|-------|--------|
| `contents: write` | Repository | Create release, upload assets |
| `id-token: write` | OIDC | Build provenance attestation |
| `attestations: write` | Repository | Attach attestation bundles |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Release not created | `HOMEBREW_TAP_TOKEN` expired or missing | Rotate the PAT in repository secrets |
| Cross-compilation fails | Missing target toolchain | Add `rustup target add <target>` to the job steps |
| Changelog is empty | No commits since last tag | Verify commit history and `cliff.toml` patterns |
| Downstream workflows not triggered | `HOMEBREW_TAP_TOKEN` not used for release creation | Ensure release job uses the PAT, not `GITHUB_TOKEN` |

See also: [Releasing runbook](../runbooks/RELEASING.md).
