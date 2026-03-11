---
diataxis_type: reference
---
# Docker Hub Workflow

## Overview

A template workflow for publishing the Docker image to **multiple registries**
simultaneously — both Docker Hub and the GitHub Container Registry (GHCR).
Disabled by default; the primary Docker publishing path uses
[`docker.yml`](DOCKER.md) (GHCR only).

**Workflow:** `.github/workflows/docker-hub.yml`  
**Trigger:** Manual (`workflow_dispatch`) — the `push` tag trigger is commented out  
**Registries:** Docker Hub + `ghcr.io`  
**Required secrets:** `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN`

> This workflow is **not active** by default. See [Enabling](#enabling) below.

## Image Tags

The same tag strategy is applied to both registries:

| Pattern | Example |
|---------|---------|
| Full semver | `zircote/nsip:0.4.0` |
| Major.minor | `zircote/nsip:0.4` |
| Major only | `zircote/nsip:0` |
| Short SHA | `zircote/nsip:sha-a1b2c3d` |

## Enabling

1. Configure Docker Hub secrets in the repository:
   - `DOCKERHUB_USERNAME` — your Docker Hub username or organisation
   - `DOCKERHUB_TOKEN` — a Docker Hub access token with `Read & Write` scope

2. Uncomment the `push` trigger in `.github/workflows/docker-hub.yml`:

   ```yaml
   on:
     push:
       tags:
         - 'v*'
     workflow_dispatch:
   ```

3. (Optional) Disable the single-registry `docker.yml` to avoid duplicate pushes
   to GHCR.

## Docker Hub Description Sync

The workflow includes `peter-evans/dockerhub-description` to automatically sync
the repository's `README.md` to the Docker Hub description page on every
publish. No additional configuration is needed beyond the secrets above.

## Build Details

| Setting | Value |
|---------|-------|
| Platforms | `linux/amd64`, `linux/arm64` |
| QEMU | Enabled via `docker/setup-qemu-action` |
| Cache | GitHub Actions cache |

## Pulling the Image

Once enabled:

```bash
# From Docker Hub
docker pull zircote/nsip:0.4.0

# From GHCR (also published by this workflow)
docker pull ghcr.io/zircote/nsip:0.4.0
```

## Difference from `docker.yml`

| Feature | `docker.yml` | `docker-hub.yml` |
|---------|-------------|-----------------|
| Registry | GHCR only | Docker Hub + GHCR |
| Build provenance | Yes (Sigstore) | No |
| Description sync | No | Yes (Docker Hub) |
| Active by default | Yes | No |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Login to Docker Hub fails | Expired or incorrect `DOCKERHUB_TOKEN` | Regenerate the access token in Docker Hub settings |
| Description not updated | `README.md` too large for Docker Hub | Keep the top-level `README.md` concise |
| Duplicate GHCR push | Both `docker.yml` and `docker-hub.yml` active | Disable the GHCR images block in `docker-hub.yml` or disable `docker.yml` |

See also: [Docker Registries distribution guide](../distribution/DOCKER-REGISTRIES.md).
