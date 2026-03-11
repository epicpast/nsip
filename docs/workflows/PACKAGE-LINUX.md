---
diataxis_type: reference
---
# Linux Packages Workflow

## Overview

Builds native Linux distribution packages — a **Debian `.deb`** and an
**RPM `.rpm`** — triggered on every published GitHub Release or manually.
Packages are uploaded to the release and preserved as workflow artifacts.

**Workflow:** `.github/workflows/package-linux.yml`  
**Trigger:** Release published, manual (`workflow_dispatch`)  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Permissions:** `contents: write`

## Jobs

### `debian-package` — Debian/Ubuntu `.deb`

| Step | Details |
|------|---------|
| Build binary | `cargo build --release` |
| Create package | `cargo deb --no-build --output target/debian/nsip.deb` (using `cargo-deb@2.7.0`) |
| Upload artifact | `debian-package` artifact, retained 90 days |
| Upload to release | Attaches `*.deb` to the GitHub Release (release trigger only) |

### `rpm-package` — RHEL/Fedora `.rpm`

| Step | Details |
|------|---------|
| Build binary | `cargo build --release` |
| Strip binary | `strip target/release/nsip` (reduces package size) |
| Create package | `cargo generate-rpm` (using `cargo-generate-rpm@0.15.1`) |
| Upload artifact | `rpm-package` artifact, retained 90 days |
| Upload to release | Attaches `*.rpm` to the GitHub Release (release trigger only) |

## Package Metadata

Package metadata is sourced from `Cargo.toml`. Ensure the following fields
are set:

```toml
[package]
name = "nsip"
version = "0.4.0"
description = "NSIP Search API client for nsipsearch.nsip.org/api"
license = "MIT"
homepage = "https://github.com/zircote/nsip"
repository = "https://github.com/zircote/nsip"

# Required for cargo-deb
[package.metadata.deb]
maintainer = "Robert Allen <zircote@gmail.com>"
depends = "$auto"
section = "utils"
```

## Installing the Packages

```bash
# Debian/Ubuntu
sudo dpkg -i nsip_0.4.0_amd64.deb

# RPM (RHEL/Fedora/CentOS)
sudo rpm -i nsip-0.4.0-1.x86_64.rpm
# or
sudo dnf install nsip-0.4.0-1.x86_64.rpm
```

## Relationship to Release Pipeline

The [Release workflow](RELEASE.md) triggers downstream packaging workflows
via the `release` event. `package-linux.yml` fires when the GitHub Release is
published (after binaries are uploaded by the Release workflow).

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `cargo deb` fails | Missing `[package.metadata.deb]` in `Cargo.toml` | Add the required metadata section |
| RPM build fails | Missing system dependencies | Check `cargo-generate-rpm` requirements for the runner OS |
| Empty `.deb` or `.rpm` | Binary not built before packaging | Verify `cargo build --release` step precedes packaging |
| Package not attached to release | Workflow triggered manually, not by release event | Use the release event or run manually and upload manually |

See also: [Package Managers distribution guide](../distribution/PACKAGE-MANAGERS.md).
