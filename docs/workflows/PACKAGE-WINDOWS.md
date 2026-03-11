---
diataxis_type: reference
---
# Windows MSI Installer Workflow

## Overview

Builds a Windows MSI installer using [WiX Toolset](https://wixtoolset.org/)
via `cargo-wix`. Triggered on every published GitHub Release or manually.
The installer is uploaded to the release and preserved as a workflow artifact.

**Workflow:** `.github/workflows/package-windows.yml`  
**Trigger:** Release published, manual (`workflow_dispatch`)  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Runs on:** `windows-latest`  
**Timeout:** 40 minutes  
**Permissions:** `contents: write`

## Jobs

### `msi`

| Step | Details |
|------|---------|
| Checkout | Clones the repository |
| Rust setup | Stable toolchain with Cargo cache (key: `wix`) |
| Install `cargo-wix` | Via `.github/actions/install-cargo-tool` |
| Build binary | `cargo build --release` |
| Initialise WiX | `cargo wix init` — generates `wix/main.wxs` |
| Build MSI | `cargo wix --nocapture` |
| Rename installer | `nsip-<version>-x64.msi` |
| Upload artifact | `windows-msi` artifact, retained 90 days |
| Upload to release | Attaches `*.msi` to the GitHub Release (release trigger only) |

## WiX Configuration

After `cargo wix init`, the generated `wix/main.wxs` can be customised to
control the installer UI, shortcuts, environment variables, and registry
entries.

Common customisations:

```xml
<!-- Add nsip to the system PATH during installation -->
<Environment Id="PATH" Name="PATH" Value="[INSTALLDIR]"
             Permanent="no" Part="last" Action="set" System="yes" />
```

Commit the customised `wix/` directory to the repository so the workflow uses
it on every build.

## Installing

Download `nsip-<version>-x64.msi` from the [GitHub Releases page](https://github.com/zircote/nsip/releases)
and run it. The installer wizard guides the user through installation.

Silent install:

```powershell
msiexec /i nsip-0.4.0-x64.msi /quiet /norestart
```

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `cargo wix` fails | WiX Toolset not found on runner | `cargo-wix` bundles WiX; ensure the install step ran |
| Rename step fails | Version regex not matching | Check that `Cargo.toml` has `version = "x.y.z"` in the `[package]` section |
| MSI not attached to release | Workflow triggered manually | Upload manually or trigger via a release event |
| Installer UI shows wrong name | `wix/main.wxs` still references template name | Regenerate with `cargo wix init` or update `main.wxs` manually |

See also: [Package Managers distribution guide](../distribution/PACKAGE-MANAGERS.md),
[Release workflow](RELEASE.md).
