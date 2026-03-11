---
diataxis_type: reference
---
# Homebrew Package Workflow

## Overview

Updates the [Homebrew tap](https://github.com/zircote/homebrew-tap) with new
`nsip` formulae whenever a release completes. Generates both a **binary formula**
(pre-built binaries) and a **source formula** (builds from source via `cargo`).

**Workflow:** `.github/workflows/package-homebrew.yml`  
**Trigger:** Successful completion of the Release workflow, manual
(`workflow_dispatch`)  
**Required secrets:** `HOMEBREW_TAP_TOKEN` — PAT with write access to the tap
repository  
**Runs on:** `macos-latest`  
**Environment:** `copilot`

## Formulae

### `nsip` (binary formula)

Installs pre-built binaries downloaded from the GitHub Release, with shell
completions (bash, zsh, fish) and man pages.

| Platform | Binary |
|----------|--------|
| macOS ARM64 | `nsip-macos-arm64` |
| macOS x86_64 | `nsip-macos-amd64` |
| Linux x86_64 | `nsip-linux-amd64` |

### `nsip-source` (source formula)

Builds `nsip` from the release source tarball using `cargo install`. Also
generates completions and man pages at install time.

## Installation (after workflow runs)

```bash
# Add the tap
brew tap zircote/tap

# Install pre-built binary
brew install nsip

# Or build from source
brew install nsip-source
```

## Dry Run

Use the `workflow_dispatch` input to preview the generated formulae without
pushing to the tap:

1. Go to **Actions → Homebrew Package → Run workflow**
2. Enter the target version (e.g., `0.4.0`)
3. Check **Dry run** — the workflow prints the formulae to the log but does
   not push

## Version Resolution

When triggered by the Release workflow, the version is read from the
`workflow_run.head_branch` (which must be a `v*.*.*` tag). When triggered
manually, the `version` input is used (the leading `v` is stripped
automatically).

## SHA-256 Checksums

The workflow downloads each release asset from the GitHub API and computes its
SHA-256 checksum. These checksums are embedded directly in the generated
formulae so Homebrew can verify downloads.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Workflow not triggered | Release workflow failed or `head_branch` is not a tag | Ensure the Release workflow succeeds with a proper version tag |
| Formula push fails | Expired `HOMEBREW_TAP_TOKEN` | Rotate the PAT in repository secrets |
| Wrong version in formula | Non-semver `head_branch` | Verify the release tag matches `v<semver>` |
| Asset SHA mismatch | Asset was modified after upload | Do not modify release assets after publishing |

See also: [Package Managers distribution guide](../distribution/PACKAGE-MANAGERS.md),
[Release workflow](RELEASE.md).
