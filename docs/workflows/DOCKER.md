---
diataxis_type: reference
---
# Docker Workflow

## Overview

Builds a multi-architecture Docker image and pushes it to the GitHub Container
Registry (GHCR) whenever a release tag is pushed. Attaches a build provenance
attestation to the image for supply-chain transparency.

**Workflow:** `.github/workflows/docker.yml`  
**Trigger:** Push of a `v*.*.*` tag, manual (`workflow_dispatch`)  
**Registry:** `ghcr.io/<owner>/<repo>`  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Permissions:** `contents: read`, `packages: write`, `id-token: write`,
`attestations: write`

## Image Tags

Docker metadata is extracted automatically with the following tag patterns:

| Pattern | Example |
|---------|---------|
| Full semver | `ghcr.io/zircote/nsip:0.6.0` |
| Major.minor | `ghcr.io/zircote/nsip:0.6` |
| Major only | `ghcr.io/zircote/nsip:0` |
| Short SHA | `ghcr.io/zircote/nsip:sha-a1b2c3d` |

## Build Details

| Setting | Value |
|---------|-------|
| Platforms | `linux/amd64`, `linux/arm64` |
| Cache | GitHub Actions cache (GHA mode) |
| Build arg | `RUST_VERSION=1.92` |
| Base image | Distroless (see `Dockerfile`) |

## Build Provenance Attestation

After the image is pushed, `actions/attest-build-provenance` attaches a
Sigstore-based provenance attestation to the image in GHCR. This attestation
records the exact workflow run, commit SHA, and repository that produced the
image.

Verify the attestation:

```bash
gh attestation verify oci://ghcr.io/zircote/nsip:0.6.0 \
  --repo zircote/nsip
```

## Pulling the Image

```bash
# Pull the latest stable release
docker pull ghcr.io/zircote/nsip:latest

# Pull a specific version
docker pull ghcr.io/zircote/nsip:0.6.0

# Run nsip via Docker
docker run --rm ghcr.io/zircote/nsip:0.6.0 search --breed-id 640
```

## Concurrency

The workflow sets `cancel-in-progress: false` so that a concurrent push to the
same tag never cancels an in-progress image push. This prevents partially-pushed
manifests in the registry.

## Relationship to Release Pipeline

`docker.yml` and the [Release workflow](RELEASE.md) (`release.yml`) both
trigger **directly** on the same `push` of a `v*.*.*` tag — there is no
trigger dependency between them. When you push a release tag, both
workflows start independently and run in parallel. `docker.yml` builds and
pushes the container image; `release.yml` builds binaries, SBOM, and the
GitHub Release. Neither waits on or fires the other.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Login to GHCR fails | `packages: write` permission missing | Add `packages: write` to the job permissions |
| Multi-arch build fails | QEMU not set up | The workflow uses `docker/setup-buildx-action`; ensure it runs before the build |
| Attestation step fails | `id-token: write` missing | Verify permissions block includes `id-token: write` and `attestations: write` |
| Image not visible in GHCR | Package visibility set to private | Change package visibility in repository settings |

See also: [Docker Registries distribution guide](../distribution/DOCKER-REGISTRIES.md).
