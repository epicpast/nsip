---
diataxis_type: reference
---
# Docker Multi-Registry Distribution

## Overview

Docker image publication to container registries.

**Active workflow:** `.github/workflows/docker.yml` — publishes to GitHub
Container Registry (ghcr.io).
**Platforms:** linux/amd64, linux/arm64

> The multi-registry Docker Hub workflow (`docker-hub.yml`) is **not currently
> included** in this repository. The registry-specific recipes below (Docker
> Hub, AWS ECR, Google Artifact Registry, Azure ACR, Quay.io) are reference
> guidance for extending `docker.yml` if you need to publish beyond ghcr.io.

## Supported Registries

### 1. Docker Hub (docker.io)

**Default public registry**

```bash
# Pull image
docker pull username/nsip:latest
docker pull username/nsip:0.1.0

# Run container
docker run -it username/nsip:latest
```

**URL:** https://hub.docker.com/r/username/nsip

### 2. GitHub Container Registry (ghcr.io)

**Integrated with GitHub**

```bash
# Pull image
docker pull ghcr.io/username/nsip:latest
docker pull ghcr.io/username/nsip:0.1.0

# Run container
docker run -it ghcr.io/username/nsip:latest
```

**URL:** https://github.com/username/nsip/pkgs/container/nsip

### 3. AWS ECR (Elastic Container Registry)

**Amazon's private registry**

```bash
# Login
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin \
  123456789.dkr.ecr.us-east-1.amazonaws.com

# Pull image
docker pull 123456789.dkr.ecr.us-east-1.amazonaws.com/nsip:latest
```

**Setup:** Add to workflow:

```yaml
- name: Login to Amazon ECR
  uses: aws-actions/amazon-ecr-login@v2

- name: Build and push to ECR
  uses: docker/build-push-action@v6
  with:
    push: true
    tags: |
      123456789.dkr.ecr.us-east-1.amazonaws.com/nsip:${{ github.sha }}
      123456789.dkr.ecr.us-east-1.amazonaws.com/nsip:latest
```

### 4. Google Artifact Registry

**Google Cloud's registry**

```bash
# Login
gcloud auth configure-docker us-docker.pkg.dev

# Pull image
docker pull us-docker.pkg.dev/PROJECT/nsip/nsip:latest
```

**Setup:** Add to workflow:

```yaml
- name: Authenticate to Google Cloud
  uses: google-github-actions/auth@v2
  with:
    credentials_json: ${{ secrets.GCP_CREDENTIALS }}

- name: Setup Cloud SDK
  uses: google-github-actions/setup-gcloud@v2

- name: Configure Docker
  run: gcloud auth configure-docker us-docker.pkg.dev

- name: Build and push
  uses: docker/build-push-action@v6
  with:
    push: true
    tags: us-docker.pkg.dev/PROJECT/nsip/nsip:latest
```

### 5. Azure Container Registry (ACR)

**Microsoft Azure's registry**

```bash
# Login
az acr login --name myregistry

# Pull image
docker pull myregistry.azurecr.io/nsip:latest
```

**Setup:** Add to workflow:

```yaml
- name: Login to Azure
  uses: azure/login@v2
  with:
    creds: ${{ secrets.AZURE_CREDENTIALS }}

- name: Login to ACR
  uses: azure/docker-login@v2
  with:
    login-server: myregistry.azurecr.io
    username: ${{ secrets.ACR_USERNAME }}
    password: ${{ secrets.ACR_PASSWORD }}

- name: Build and push
  uses: docker/build-push-action@v6
  with:
    push: true
    tags: myregistry.azurecr.io/nsip:latest
```

### 6. Quay.io

**Red Hat's public registry**

```bash
# Pull image
docker pull quay.io/username/nsip:latest
```

**Setup:** Add to workflow:

```yaml
- name: Login to Quay.io
  uses: docker/login-action@v3
  with:
    registry: quay.io
    username: ${{ secrets.QUAY_USERNAME }}
    password: ${{ secrets.QUAY_TOKEN }}

- name: Build and push
  uses: docker/build-push-action@v6
  with:
    push: true
    tags: quay.io/username/nsip:latest
```

## Image Tagging Strategy

### Semantic Versioning

```yaml
tags: |
  type=semver,pattern={{version}}       # 0.1.0
  type=semver,pattern={{major}}.{{minor}}  # 0.1
  type=semver,pattern={{major}}         # 0
  type=sha                              # sha-abc1234
```

**Results:**
- `latest` - Always points to newest release
- `0.1.0` - Specific version
- `0.1` - Latest patch in minor version
- `0` - Latest minor in major version
- `sha-abc1234` - Git commit SHA

### Branch Tagging

```yaml
tags: |
  type=ref,event=branch   # main, develop
  type=ref,event=pr       # pr-123
```

## Configuration

### Required Secrets

#### Docker Hub

1. Go to https://hub.docker.com/settings/security
2. Create Access Token
3. Add GitHub secrets:
   - `DOCKERHUB_USERNAME`
   - `DOCKERHUB_TOKEN`

#### GitHub Container Registry

No setup needed - uses `GITHUB_TOKEN` automatically.

**Make package public:**
1. Go to package settings
2. Change visibility to Public

### Multi-Platform Builds

Current platforms: `linux/amd64`, `linux/arm64`

**Add more platforms:**

```yaml
platforms: linux/amd64,linux/arm64,linux/arm/v7
```

**Supported architectures:**
- `linux/amd64` - x86_64 (Intel/AMD)
- `linux/arm64` - ARM 64-bit (Apple Silicon, ARM servers)
- `linux/arm/v7` - ARM 32-bit (Raspberry Pi)
- `linux/386` - x86 32-bit
- `linux/ppc64le` - PowerPC
- `linux/s390x` - IBM Z

## Image Metadata

### Labels

Automatically added via `docker/metadata-action`:

```dockerfile
LABEL org.opencontainers.image.source="https://github.com/USER/REPO"
LABEL org.opencontainers.image.description="Modern Rust template"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.version="0.1.0"
```

### Attestations

Add provenance and SBOM:

```yaml
- name: Build and push
  uses: docker/build-push-action@v6
  with:
    provenance: true
    sbom: true
```

## Registry-Specific Features

### Docker Hub

- **Auto README sync** - Updates description from GitHub README
- **Webhooks** - Trigger on image push
- **Vulnerability scanning** - Free for public images
- **Download stats** - Track pull metrics

### GitHub Container Registry

- **Tight integration** - Links to repository automatically
- **Package permissions** - Inherit repo permissions
- **Free for public** - Unlimited public images
- **Packages API** - Programmatic access

### AWS ECR

- **Private by default** - No public registry
- **Scanning** - Amazon Inspector integration
- **Lifecycle policies** - Auto-delete old images
- **IAM integration** - AWS permissions

## Security Best Practices

### 1. Use Specific Tags

```bash
# ❌ Bad - Can break on updates
FROM rust:latest

# ✅ Good - Pinned version
FROM rust:1.92-slim
```

### 2. Sign Images

```yaml
- name: Install cosign
  uses: sigstore/cosign-installer@v3

- name: Sign image
  run: |
    cosign sign --yes \
      username/nsip:${{ github.sha }}
```

### 3. Scan for Vulnerabilities

```yaml
- name: Run Trivy
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: username/nsip:latest
    format: 'sarif'
    output: 'trivy-results.sarif'
```

### 4. Use Minimal Base Images

```dockerfile
# ✅ Distroless - No shell, minimal attack surface
FROM gcr.io/distroless/cc-debian12

# ✅ Alpine - Small but has shell
FROM alpine:latest

# ⚠️ Debian slim - Larger but more compatible
FROM debian:12-slim
```

## Cache Optimization

### GitHub Actions Cache

```yaml
cache-from: type=gha
cache-to: type=gha,mode=max
```

**Benefits:**
- Reuses layers between builds
- Faster builds (minutes → seconds)
- No external cache storage needed

### Registry Cache

```yaml
cache-from: type=registry,ref=username/nsip:buildcache
cache-to: type=registry,ref=username/nsip:buildcache,mode=max
```

## Troubleshooting

### Authentication Fails

```bash
# Verify token
echo $DOCKERHUB_TOKEN | docker login -u $DOCKERHUB_USERNAME --password-stdin

# Check ghcr.io permissions
echo $GITHUB_TOKEN | docker login ghcr.io -u $GITHUB_ACTOR --password-stdin
```

### Multi-Platform Build Fails

```bash
# Check QEMU
docker run --rm --privileged multiarch/qemu-user-static --reset -p yes

# Build specific platform
docker buildx build --platform linux/arm64 .
```

### Image Too Large

```bash
# Analyze layers
docker history username/nsip:latest

# Use dive for interactive analysis
dive username/nsip:latest
```

**Optimization tips:**
- Use multi-stage builds
- Combine RUN commands
- Remove build artifacts
- Use .dockerignore

## Monitoring

### Download Metrics

**Docker Hub:**
```bash
# Via Hub API
curl https://hub.docker.com/v2/repositories/username/nsip/
```

**GitHub Container Registry:**
```bash
# Via GitHub API
gh api /users/username/packages/container/nsip
```

### Vulnerability Alerts

Enable on:
- Docker Hub: Settings → Vulnerability Scanning
- GitHub: Settings → Security → Dependabot
- AWS ECR: Auto-enabled with Inspector

## Links

- [Docker Hub](https://hub.docker.com/)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [Docker Build Push Action](https://github.com/docker/build-push-action)
- [Multi-platform Builds](https://docs.docker.com/build/building/multi-platform/)
- [OCI Image Spec](https://github.com/opencontainers/image-spec)
