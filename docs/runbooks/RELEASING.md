# Releasing

End-to-end runbook for creating, monitoring, and rolling back releases of rust-template.

## Version Numbering (SemVer)

This project follows [Semantic Versioning 2.0.0](https://semver.org/):

| Change type | Version bump | Example | When to use |
|---|---|---|---|
| Breaking API change | **MAJOR** | `1.0.0` -> `2.0.0` | Removed public types, changed function signatures |
| New feature (backward-compatible) | **MINOR** | `0.1.0` -> `0.2.0` | New public functions, new optional fields |
| Bug fix (backward-compatible) | **PATCH** | `0.1.0` -> `0.1.1` | Fix incorrect behavior, performance improvement |

**Pre-1.0 policy:** While on `0.x.y`, MINOR bumps may include breaking changes. Document these clearly in commit messages with `BREAKING CHANGE:` in the body.

---

## Prerequisites

### Required Secrets

Configure in GitHub repository settings (**Settings > Secrets and variables > Actions**):

| Secret | Purpose | How to generate |
|---|---|---|
| `CARGO_REGISTRY_TOKEN` | Publish to crates.io | https://crates.io/settings/tokens (scope: `publish-update`) |
| `GITHUB_TOKEN` | Provided automatically | No setup needed |

### Permissions

- **GitHub Packages (Docker):** Settings > Actions > General > Workflow permissions > "Read and write permissions"

---

## Pre-Release Checklist

Run through this checklist before every release.

- [ ] All CI checks pass on `main` (check [Actions](https://github.com/zircote/rust-template/actions/workflows/ci.yml))
- [ ] Update version in `Cargo.toml`:
  ```toml
  [package]
  version = "X.Y.Z"  # New version
  ```
- [ ] Run the full local check suite:
  ```bash
  cargo fmt -- --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  cargo deny check
  cargo doc --no-deps --all-features
  ```
- [ ] Build a release binary locally to verify:
  ```bash
  cargo build --release
  ```
- [ ] Review `CHANGELOG.md` and recent commits since last tag:
  ```bash
  git log $(git describe --tags --abbrev=0)..HEAD --oneline
  ```
- [ ] Verify conventional commit messages are correct (they drive changelog generation)
- [ ] If breaking changes exist, confirm MAJOR version bump and `BREAKING CHANGE:` in commit bodies
- [ ] Commit the version bump separately:
  ```bash
  git add Cargo.toml Cargo.lock
  git commit -m "chore: bump version to X.Y.Z"
  git push
  ```

---

## Step-by-Step: Create and Push a Release Tag

### 1. Create an Annotated Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 2. Push the Tag

```bash
git push origin vX.Y.Z
```

This single push triggers all release automation.

### 3. Triggered Workflows

Pushing a `v*.*.*` tag triggers these workflows in parallel:

| Workflow | File | What it does |
|---|---|---|
| **Release** | `release.yml` | Builds binaries for 5 platform targets, generates changelog via git-cliff, creates a GitHub Release with assets |
| **Changelog** | `changelog.yml` | Regenerates `CHANGELOG.md` and commits it to `main` |
| **Docker** | `docker.yml` | Builds multi-platform images (linux/amd64, linux/arm64), pushes to `ghcr.io/zircote/rust-template` with version + `latest` tags |
| **Publish** | `publish.yml` | Runs pre-publish checks and publishes to crates.io (if enabled and tag-triggered) |
| **Signed Releases** | `signed-releases.yml` | Signs all release assets with Sigstore Cosign, generates SHA256/SHA512 checksums |

---

## Monitoring Workflow Progress

### GitHub Actions Dashboard

- **All workflows:** https://github.com/zircote/rust-template/actions
- **Filter by tag:** Click the specific workflow run triggered by the tag push

### CLI Monitoring

```bash
# List recent workflow runs
gh run list --limit 10

# Watch a specific run
gh run watch <run-id>

# View logs for a failed run
gh run view <run-id> --log-failed
```

### What to Watch For

| Stage | Expected duration | Common failure point |
|---|---|---|
| Create Release | ~1 min | git-cliff config issues |
| Build Binaries | ~5-10 min | Cross-compilation (especially ARM64 Linux) |
| Docker Build | ~5-10 min | Buildx multi-platform, registry auth |
| Publish (crates.io) | ~3 min | Token issues, pre-publish checks |
| Signed Releases | ~2 min | Cosign signing |

---

## Post-Release Verification

Run through this after all workflows complete.

- [ ] **GitHub Release** exists with correct version:
  ```bash
  gh release view vX.Y.Z
  ```
- [ ] **All 5 binary assets** are attached:
  - `rust_template-linux-amd64`
  - `rust_template-linux-arm64`
  - `rust_template-macos-amd64`
  - `rust_template-macos-arm64`
  - `rust_template-windows-amd64.exe`
- [ ] **Checksums and signatures** are attached (`SHA256SUMS`, `SHA512SUMS`, `*.sig` files)
- [ ] **Release notes** are generated correctly from conventional commits
- [ ] **Docker image** is available:
  ```bash
  docker pull ghcr.io/zircote/rust-template:vX.Y.Z
  docker run --rm ghcr.io/zircote/rust-template:vX.Y.Z --version
  ```
- [ ] **Docker `latest` tag** points to the new release:
  ```bash
  docker pull ghcr.io/zircote/rust-template:latest
  docker run --rm ghcr.io/zircote/rust-template:latest --version
  ```
- [ ] **crates.io** package updated (if publishing is enabled):
  ```bash
  cargo install rust_template@X.Y.Z
  # Or check: https://crates.io/crates/rust_template
  ```
- [ ] **CHANGELOG.md** on `main` updated by the changelog workflow
- [ ] Download and test a binary on at least one platform:
  ```bash
  wget https://github.com/zircote/rust-template/releases/download/vX.Y.Z/rust_template-linux-amd64
  chmod +x rust_template-linux-amd64
  ./rust_template-linux-amd64 --version
  ```

---

## Rollback Procedures

### Roll Back a GitHub Release

```bash
# Delete the release
gh release delete vX.Y.Z --yes

# Delete the remote tag
git push --delete origin vX.Y.Z

# Delete the local tag
git tag -d vX.Y.Z
```

### Roll Back a crates.io Publish

**You cannot unpublish from crates.io.** Your options:

1. **Yank the version** (prevents new projects from depending on it):
   ```bash
   cargo yank --version X.Y.Z
   ```
2. **Publish a fix** as a patch release:
   ```bash
   # Fix the issue, bump to X.Y.Z+1
   git tag -a vX.Y.(Z+1) -m "Release vX.Y.(Z+1) (fixes vX.Y.Z)"
   git push origin vX.Y.(Z+1)
   ```

### Roll Back Docker Images

Docker images on GHCR are immutable by tag. To mitigate:

1. **Point users to a previous version:**
   ```bash
   docker pull ghcr.io/zircote/rust-template:vPREVIOUS
   ```
2. **Delete the package version** via GitHub UI: Packages > rust-template > Package versions > Delete
3. **Re-tag `latest`** to the previous good version by re-pushing a known-good tag

---

## Hotfix Release Process

When a critical bug or security issue is found in the latest release:

### 1. Create a Hotfix Branch

```bash
# Branch from the release tag
git checkout -b hotfix/vX.Y.(Z+1) vX.Y.Z
```

### 2. Apply the Fix

```bash
# Make the fix, then:
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

### 3. Bump Version and Tag

```bash
# Update Cargo.toml to X.Y.(Z+1)
git add -A
git commit -m "fix: <description of the critical fix>"
git commit --allow-empty -m "chore: bump version to X.Y.(Z+1)"
```

### 4. Merge and Release

```bash
# Merge hotfix into main
git checkout main
git merge hotfix/vX.Y.(Z+1)
git push origin main

# Tag and push
git tag -a vX.Y.(Z+1) -m "Release vX.Y.(Z+1)"
git push origin vX.Y.(Z+1)
```

### 5. If the Bad Version Was on crates.io

```bash
# Yank the bad version
cargo yank --version X.Y.Z

# The hotfix tag push triggers automatic publish of X.Y.(Z+1)
```

---

## Changelog Generation

Changelogs are generated automatically by [git-cliff](https://git-cliff.org/) from conventional commits:

| Commit prefix | Changelog section |
|---|---|
| `feat:` | Added |
| `fix:` | Fixed |
| `docs:` | Documentation |
| `perf:` | Performance |
| `refactor:` | Refactored |
| `test:` | Testing |
| `chore:` | Miscellaneous |

**Best practices:**
- Use scoped prefixes for clarity: `feat(auth): add JWT validation`
- Include `BREAKING CHANGE:` in the commit body for breaking changes
- The release workflow (git-cliff) generates per-release notes; the changelog workflow regenerates the full `CHANGELOG.md`

---

## Deployment Targets Quick Reference

### GitHub Releases

- **URL:** https://github.com/zircote/rust-template/releases
- **Platforms:** Linux (amd64, arm64), macOS (amd64, arm64), Windows (amd64)
- **Signatures:** Cosign keyless signing + SHA256/SHA512 checksums

### Docker (GHCR)

- **Registry:** `ghcr.io/zircote/rust-template`
- **Platforms:** linux/amd64, linux/arm64
- **Base image:** distroless/cc-debian12 (minimal attack surface)
- **User:** nonroot:nonroot (unprivileged)
- **Tags:** `vX.Y.Z`, `X.Y`, `X`, `latest`, `sha-<commit>`

### crates.io

- **Package:** https://crates.io/crates/rust_template
- **Note:** Publishing is disabled by default in the template. Enable by uncommenting the tag trigger in `publish.yml` and setting `CARGO_REGISTRY_TOKEN`.

### Install Methods

```bash
# From GitHub release (Linux)
wget https://github.com/zircote/rust-template/releases/download/vX.Y.Z/rust_template-linux-amd64
chmod +x rust_template-linux-amd64

# From Docker
docker pull ghcr.io/zircote/rust-template:vX.Y.Z

# From crates.io
cargo install rust_template

# From source
cargo install --git https://github.com/zircote/rust-template
```

---

## Troubleshooting

| Problem | Cause | Fix |
|---|---|---|
| Release workflow fails at build | Cargo.toml version doesn't match tag | Ensure `version = "X.Y.Z"` matches tag `vX.Y.Z` |
| ARM64 Linux build fails | Missing cross-compiler | The workflow installs `gcc-aarch64-linux-gnu`; check the install step |
| Docker push fails | Insufficient permissions | Verify workflow permissions include `packages: write` |
| crates.io publish fails | Missing or expired token | Regenerate `CARGO_REGISTRY_TOKEN` in repo secrets |
| Changelog not updated | git-cliff config error | Check `cliff.toml` and the `fetch-depth: 0` checkout |
| Signing fails | Cosign OIDC issue | Check `id-token: write` permission in `signed-releases.yml` |
| Tag push doesn't trigger workflows | Tag format wrong | Must match `v*.*.*` pattern exactly (e.g., `v1.0.0`, not `1.0.0`) |
