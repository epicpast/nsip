# Deployment Guide

This document provides comprehensive deployment instructions for the nsip project.

## Overview

The project includes automated deployment workflows for:

- **GitHub Releases** - Multi-platform binaries
- **Docker** - Container images on GitHub Container Registry
- **crates.io** - Rust package registry

## Prerequisites

### Required Secrets

Configure these secrets in GitHub repository settings (Settings → Secrets and variables → Actions):

1. **CARGO_REGISTRY_TOKEN** - For crates.io publishing
   - Generate at: https://crates.io/settings/tokens
   - Scope: "publish-update"

2. **GITHUB_TOKEN** - Automatically provided by GitHub Actions (no setup needed)

### GitHub Packages

Enable GitHub Packages for Docker image publishing:
- Settings → Actions → General → Workflow permissions → "Read and write permissions"

## Creating a Release

### 1. Prepare Release

Update version in `Cargo.toml`:

```toml
[package]
version = "0.4.0"  # Update this
```

Run checks locally:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo deny check
```

### 2. Create and Push Tag

```bash
# Commit version bump
git add Cargo.toml
git commit -m "chore: bump version to 0.3.0"
git push

# Create annotated tag
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin v0.3.0
```

### 3. Automated Workflows

Pushing the tag automatically triggers:

1. **Release Workflow** (`release.yml`)
   - Builds binaries for all platforms
   - Generates changelog from commits
   - Creates GitHub release with artifacts

2. **Changelog Workflow** (`changelog.yml`)
   - Updates CHANGELOG.md
   - Commits changes to main branch

3. **Docker Workflow** (`docker.yml`)
   - Builds multi-platform images
   - Pushes to ghcr.io with version tag and 'latest'

4. **Publish Workflow** (`publish.yml`)
   - Runs all pre-publish checks
   - Publishes to crates.io

## Deployment Targets

### GitHub Releases

**Access:** https://github.com/zircote/nsip/releases

**Artifacts:**
- `nsip-linux-amd64` - Linux x86_64
- `nsip-linux-arm64` - Linux ARM64
- `nsip-macos-amd64` - macOS x86_64
- `nsip-macos-arm64` - macOS ARM64 (Apple Silicon)
- `nsip-windows-amd64.exe` - Windows x86_64

**Download Example:**

```bash
# Linux
wget https://github.com/zircote/nsip/releases/download/v0.1.0/nsip-linux-amd64
chmod +x nsip-linux-amd64
./nsip-linux-amd64 --version
```

### Docker (GitHub Container Registry)

**Registry:** ghcr.io/zircote/nsip

**Supported Platforms:**
- linux/amd64
- linux/arm64

**Pull and Run:**

```bash
# Latest version
docker pull ghcr.io/zircote/nsip:latest
docker run --rm ghcr.io/zircote/nsip:latest --version

# Specific version
docker pull ghcr.io/zircote/nsip:v0.1.0
docker run --rm ghcr.io/zircote/nsip:v0.1.0 --version

# With volumes
docker run --rm -v $(pwd):/data ghcr.io/zircote/nsip:latest
```

**Image Details:**
- Base: distroless/cc-debian12 (minimal attack surface)
- User: nonroot:nonroot (unprivileged)
- Healthcheck: Built-in with `--version` command
- Size: ~10-15 MB (optimized multi-stage build)

### crates.io

**Package:** https://crates.io/crates/nsip

**Install:**

```bash
# Latest version
cargo install nsip

# Specific version
cargo install nsip@0.1.0

# From source
cargo install --git https://github.com/zircote/nsip
```

**Use in Project:**

```toml
[dependencies]
nsip = "0.1"
```

## Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0) - Incompatible API changes
- **MINOR** (0.1.0) - Backwards-compatible functionality
- **PATCH** (0.0.1) - Backwards-compatible bug fixes

## Changelog

Changelogs are automatically generated from conventional commits:

- `feat:` → Added section
- `fix:` → Fixed section
- `docs:` → Documentation section
- `perf:` → Performance section
- `refactor:` → Refactored section
- `test:` → Testing section
- `chore:` → Miscellaneous section

**Example Commit:**

```bash
git commit -m "feat(auth): add JWT token validation"
```

## Rollback

### GitHub Release

Delete the release and tag:

```bash
# Delete remote tag
git push --delete origin v0.3.0

# Delete local tag
git tag -d v0.3.0

# Delete release via GitHub UI or gh CLI
gh release delete v0.3.0
```

### Docker

Images are immutable; use previous version tags:

```bash
docker pull ghcr.io/zircote/nsip:v0.1.0
```

### crates.io

**Cannot unpublish** - crates.io doesn't allow unpublishing. Options:

1. Yank the version (prevents new projects from using it):
   ```bash
   cargo yank --vers 0.3.0
   ```

2. Publish a patch version with fixes:
   ```bash
   # Update to X.Y.Z+1
   git tag -a vX.Y.Z -m "Release vX.Y.Z (fixes vA.B.C)"
   git push origin vX.Y.Z
   ```

## Monitoring

### GitHub Actions

Monitor workflow runs:
- Actions tab: https://github.com/zircote/nsip/actions

### Security Audits

Daily automated security scans run at 00:00 UTC:
- Workflow: `.github/workflows/security-audit.yml`
- Uses: cargo-audit
- Notifications: GitHub Actions UI

### Dependencies

Dependabot automatically opens PRs for:
- Cargo dependencies
- GitHub Actions versions

## Troubleshooting

### Release Workflow Fails

**Build Error:**
- Check Cargo.toml version matches tag
- Verify MSRV compatibility (1.92+)
- Test locally: `cargo build --release`

**Cross-compilation Error:**
- Linux ARM64 requires `gcc-aarch64-linux-gnu`
- macOS ARM64 requires macOS 11+ runner

### Docker Build Fails

**Context Issue:**
- Verify .dockerignore excludes target/
- Check Dockerfile paths match `crates/` structure

**Push Permission:**
- Verify GitHub Actions workflow permissions
- Check ghcr.io login succeeds

### Publish to crates.io Fails

**Token Issue:**
- Verify CARGO_REGISTRY_TOKEN secret is set
- Token scope must include "publish-update"

**Pre-publish Checks:**
- All tests must pass
- No clippy warnings
- cargo-deny checks must pass

## Best Practices

1. **Test Before Tagging**
   ```bash
   cargo build --release
   cargo test --all-features
   cargo clippy --all-targets --all-features -- -D warnings
   ```

2. **Use Conventional Commits**
   - Enables automatic changelog generation
   - Clearly communicates changes

3. **Version Bump in Separate Commit**
   ```bash
   git commit -m "chore: bump version to 0.3.0"
   git tag -a v0.3.0 -m "Release v0.3.0"
   ```

4. **Monitor Release Progress**
   - Watch GitHub Actions for workflow completion
   - Verify artifacts are uploaded
   - Test Docker image immediately after push

5. **Document Breaking Changes**
   - Use `BREAKING CHANGE:` in commit body
   - Update migration guide in CHANGELOG
