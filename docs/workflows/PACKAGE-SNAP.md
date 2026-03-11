---
diataxis_type: reference
---
# Snap Package Workflow

## Overview

Builds a [Snap](https://snapcraft.io/) package from `snap/snapcraft.yaml` and
optionally publishes it to the Snap Store. Disabled by default; the release
trigger is commented out and requires a `SNAPCRAFT_TOKEN` secret.

**Workflow:** `.github/workflows/package-snap.yml`  
**Trigger:** Manual (`workflow_dispatch`) — the `release` trigger is commented out  
**Required secrets:** `SNAPCRAFT_TOKEN` (for store publishing)  
**Permissions:** `contents: write`

> This workflow is **not active by default**. See [Enabling](#enabling) below.

## Jobs

### `snap`

| Step | Details |
|------|---------|
| Checkout | Clones the repository |
| Build snap | `snapcore/action-build` — builds the snap using `snap/snapcraft.yaml` |
| Upload artifact | `snap-package` artifact, retained 90 days |
| Upload to release | Attaches the `.snap` to the GitHub Release (release trigger only) |
| Publish to store | Publishes to the `stable` channel (release trigger + `main` branch only) |

## Snap Configuration

The snap is defined in `snap/snapcraft.yaml`. Key fields:

```yaml
name: nsip
base: core22
version: git
summary: NSIP sheep genetic evaluation CLI
description: |
  Query the NSIP Search API for sheep genetic evaluation data.
  Includes an MCP server for AI assistant integration.
grade: stable
confinement: strict
```

## Enabling

1. Register the snap name on [snapcraft.io](https://snapcraft.io/)
2. Export a `SNAPCRAFT_TOKEN`: `snapcraft export-login snapcraft.credentials`
3. Add `SNAPCRAFT_TOKEN` as a repository secret
4. Uncomment the `release` trigger in `.github/workflows/package-snap.yml`:

   ```yaml
   on:
     release:
       types: [published]
     workflow_dispatch:
   ```

## Installing

```bash
# Install from the Snap Store (once published)
sudo snap install nsip

# Install a locally-built snap for testing
sudo snap install --dangerous nsip_<version>_amd64.snap
```

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Build fails | `snapcraft.yaml` syntax error | Validate locally: `snapcraft --debug` |
| Store publish skipped | Not triggered by `release` event or not on `main` | Check the `if:` conditions |
| `SNAPCRAFT_TOKEN` error | Expired credentials | Re-export and update the secret |
| Snap confinement issues | `strict` confinement blocking file access | Test with `devmode` confinement first |

See also: [Package Managers distribution guide](../distribution/PACKAGE-MANAGERS.md).
