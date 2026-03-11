---
diataxis_type: reference
---
# Nightly Builds Workflow

## Overview

Builds the project with the Rust **nightly** toolchain and publishes the result
as a rolling pre-release on GitHub Releases tagged `nightly`. Intended for
users who want to test the very latest code before a stable release.

**Workflow:** `.github/workflows/nightly.yml`  
**Trigger:** Manual (`workflow_dispatch`) — the daily schedule is commented out  
**Required secrets:** `GITHUB_TOKEN`  
**Permissions:** `contents: write`

> **Stability warning:** Nightly builds may be unstable. Use stable releases
> for production.

## Enabling the Daily Schedule

The daily schedule is disabled by default. To enable it, uncomment the
`schedule` block in `.github/workflows/nightly.yml`:

```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # 02:00 UTC daily
  workflow_dispatch:
```

## Jobs

### `nightly`

Runs on `ubuntu-latest`, timeout 30 minutes.

| Step | Description |
|------|-------------|
| Checkout | Clones the repository |
| Rust setup | **Nightly** toolchain with Cargo cache |
| Build | `cargo +nightly build --release --all-features` |
| Package | Creates `nsip-nightly-linux-x64.tar.gz` + SHA-256 checksum |
| Delete old release | Removes the previous `nightly` tag/release if present |
| Create release | Publishes a new `nightly` pre-release with build metadata |

## Release Naming

The rolling release is always tagged `nightly`. The release body includes:

- A warning that the build may be unstable
- The triggering commit SHA (`${{ github.sha }}`)
- The repository's last-updated timestamp

Overwriting the `nightly` tag means there is only ever one nightly release;
older nightly builds are not retained.

## Artifacts

| File | Description |
|------|-------------|
| `nsip-nightly-linux-x64.tar.gz` | Linux x86_64 binary archive |
| `nsip-nightly-linux-x64.tar.gz.sha256` | SHA-256 checksum |

## Using Nightly Builds

```bash
# Download and install the latest nightly
curl -sL \
  https://github.com/zircote/nsip/releases/download/nightly/nsip-nightly-linux-x64.tar.gz \
  | tar xz
./nsip --version
```

## Difference from Stable Releases

| Feature | Nightly | Stable |
|---------|---------|--------|
| Toolchain | `nightly` | `stable` |
| Trigger | Daily schedule / manual | Version tag |
| Pre-release flag | Yes | No (unless `-alpha`/`-beta`/`-rc`) |
| Provenance attestation | No | Yes |
| Platforms | Linux x86_64 only | 6 targets |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Build fails | Nightly toolchain regression | Check the Rust nightly issue tracker; pin a specific nightly date |
| Release not updated | `gh release delete nightly` failed | Manually delete the `nightly` tag and release, then re-run |
| Binary name wrong | Template binary name used | Update the `tar` and `shasum` commands to use `nsip` instead of `rust-template` |
