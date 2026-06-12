---
diataxis_type: how-to
---
# Releasing

End-to-end runbook for creating, monitoring, and rolling back releases of nsip.

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
| `HOMEBREW_TAP_TOKEN` | Create the release (event propagation) and push tap formulae | Fine-grained PAT with `contents: write` on `zircote/homebrew-tap`; stored in the `copilot` environment |
| `GITHUB_TOKEN` | Provided automatically | No setup needed |

crates.io publishing uses [Trusted Publishing](https://crates.io/docs/trusted-publishing)
(OIDC) — configured on crates.io for repo `zircote/nsip`, workflow
`publish.yml`, environment `copilot`. No registry token is stored.

### Permissions

- **GitHub Packages (Docker):** Settings > Actions > General > Workflow permissions > "Read and write permissions"

---

## Pre-Release Checklist

Run through this checklist before every release.

- [ ] All CI checks pass on `develop` (check [Actions](https://github.com/zircote/nsip/actions/workflows/ci.yml))
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

## Step-by-Step: Promote, Tag, and Push a Release

### 1. Promote `develop` to `main`

Open a release PR from `develop` into `main` (or run the **Release PR** workflow
via **Actions > Release PR**, which opens/updates it for you), get it reviewed, and
merge it. Then check out the updated `main`:

```bash
git checkout main
git pull origin main
```

### 2. Create an Annotated Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 3. Push the Tag

```bash
git push origin vX.Y.Z
```

This single push triggers all release automation. Tag immediately after merging the
release PR so changelog diffs stay clean.

### 4. Triggered Workflows

Pushing a `v*.*.*` tag triggers these workflows in parallel:

| Workflow | File | What it does |
|---|---|---|
| **Release** | `release.yml` | Builds binaries (5 targets), completions/man pages, and the MCPB bundle; attests everything (provenance + SBOM); fail-closed verifies all attestations; only then creates the GitHub Release with assets, checksums, and a git-cliff changelog |
| **Changelog** | `changelog.yml` | Regenerates `CHANGELOG.md` and opens a PR into `develop` |
| **Docker** | `docker.yml` | Builds multi-platform images (linux/amd64, linux/arm64), pushes to `ghcr.io/zircote/nsip` with version + `latest` tags |
| **Publish** | `publish.yml` | Runs pre-publish checks, publishes to crates.io via Trusted Publishing (OIDC), byte-verifies the registry copy, and attests it |
| **Homebrew** | `package-homebrew.yml` | After the release publishes: regenerates `nsip.rb` and `nsip-source.rb` in `zircote/homebrew-tap` |
| **Back-merge** | `back-merge.yml` | Automatically opens and auto-merges a `main -> develop` PR so the branches stay in sync after the release (no manual step) |

> **Dry run:** `gh workflow run Release` from any branch exercises the full
> build → attest → verify chain without publishing (the release job is
> tag-gated). Use this after any pipeline change.

---

## Monitoring Workflow Progress

### GitHub Actions Dashboard

- **All workflows:** https://github.com/zircote/nsip/actions
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
| Build Binaries | ~5-10 min | Platform toolchains, `--locked` drift |
| Verify Attestations | ~1 min | Missing artifact (count check), attestation API |
| Create Release | ~1 min | git-cliff config issues, `HOMEBREW_TAP_TOKEN` |
| Docker Build | ~5-10 min | Buildx multi-platform, registry auth |
| Publish (crates.io) | ~3 min | Trusted Publishing config, pre-publish checks |
| Homebrew | ~2 min | Tap push permissions, asset download |

---

## Post-Release Verification

Run through this after all workflows complete.

- [ ] **GitHub Release** exists with correct version:
  ```bash
  gh release view vX.Y.Z
  ```
- [ ] **All 5 binary assets** are attached (release-asset names carry the version, `X.Y.Z` = tag minus `v`):
  - `nsip-X.Y.Z-linux-amd64`
  - `nsip-X.Y.Z-linux-arm64`
  - `nsip-X.Y.Z-macos-amd64`
  - `nsip-X.Y.Z-macos-arm64`
  - `nsip-X.Y.Z-windows-amd64.exe`
- [ ] **Checksums** are attached (`nsip-X.Y.Z-checksums.txt`)
- [ ] **Supplementary artifacts** are attached (produced by `release.yml`):
  - `nsip-X.Y.Z-sbom.cdx.json` (CycloneDX SBOM)
  - `nsip-X.Y.Z-completions.tar.gz` (shell completions: bash, zsh, fish, powershell)
  - `nsip-X.Y.Z-man-pages.tar.gz` (man pages)
  - the MCPB bundle (`nsip-X.Y.Z.mcpb`)
- [ ] **Attestations verify** from a workstation (independent of the pipeline):
  ```bash
  gh release download vX.Y.Z --repo zircote/nsip --pattern 'nsip-X.Y.Z-linux-amd64'
  gh attestation verify nsip-X.Y.Z-linux-amd64 --repo zircote/nsip
  gh attestation verify nsip-X.Y.Z-linux-amd64 --repo zircote/nsip \
    --predicate-type https://cyclonedx.org/bom
  ```
- [ ] **Release notes** are generated correctly from conventional commits
- [ ] **Docker image** is available:
  ```bash
  docker pull ghcr.io/zircote/nsip:vX.Y.Z
  docker run --rm ghcr.io/zircote/nsip:vX.Y.Z --version
  ```
- [ ] **Docker `latest` tag** points to the new release:
  ```bash
  docker pull ghcr.io/zircote/nsip:latest
  docker run --rm ghcr.io/zircote/nsip:latest --version
  ```
- [ ] **crates.io** package updated:
  ```bash
  cargo install nsip@X.Y.Z
  # Or check: https://crates.io/crates/nsip
  ```
- [ ] **Homebrew tap** updated (`zircote/homebrew-tap` `Formula/nsip.rb` and `Formula/nsip-source.rb` show the new version):
  ```bash
  brew update && brew info zircote/tap/nsip
  ```
- [ ] **CHANGELOG.md** PR into `develop` opened by the changelog workflow
- [ ] Download and test a binary on at least one platform:
  ```bash
  wget https://github.com/zircote/nsip/releases/download/vX.Y.Z/nsip-X.Y.Z-linux-amd64
  chmod +x nsip-X.Y.Z-linux-amd64
  ./nsip-X.Y.Z-linux-amd64 --version
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
   docker pull ghcr.io/zircote/nsip:vPREVIOUS
   ```
2. **Delete the package version** via GitHub UI: Packages > nsip > Package versions > Delete
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

# Back-merge the hotfix into develop so the fix isn't lost on the next release
git checkout develop
git pull origin develop
git merge --no-ff main
git push origin develop
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

- **URL:** https://github.com/zircote/nsip/releases
- **Platforms:** Linux (amd64, arm64), macOS (amd64, arm64), Windows (amd64)
- **Attestations:** SLSA build provenance + CycloneDX SBOM attestation per asset (`gh attestation verify`), SHA-256 checksums file

### Docker (GHCR)

- **Registry:** `ghcr.io/zircote/nsip`
- **Platforms:** linux/amd64, linux/arm64
- **Base image:** distroless/cc-debian12 (minimal attack surface)
- **User:** nonroot:nonroot (unprivileged)
- **Tags:** `vX.Y.Z`, `X.Y`, `X`, `latest`, `sha-<commit>`

### crates.io

- **Package:** https://crates.io/crates/nsip
- **Note:** `publish.yml` runs automatically on every `v*.*.*` tag push. It runs pre-publish checks, publishes via Trusted Publishing (OIDC — no stored token), byte-verifies the registry copy, and attests it.

### Homebrew

- **Tap:** `zircote/homebrew-tap`
- **Formulae:** `nsip` (pre-built binaries + completions + man pages), `nsip-source` (build from source)

### Install Methods

```bash
# From Homebrew
brew install zircote/tap/nsip

# From GitHub release (Linux)
wget https://github.com/zircote/nsip/releases/download/vX.Y.Z/nsip-X.Y.Z-linux-amd64
chmod +x nsip-X.Y.Z-linux-amd64

# From Docker
docker pull ghcr.io/zircote/nsip:vX.Y.Z

# From crates.io
cargo install nsip

# From source
cargo install --git https://github.com/zircote/nsip
```

---

## Troubleshooting

| Problem | Cause | Fix |
|---|---|---|
| Release workflow fails at build | Cargo.toml version doesn't match tag | Ensure `version = "X.Y.Z"` matches tag `vX.Y.Z` |
| `expected 9 artifacts` in verify | A build/extras/mcpb job failed or uploaded nothing | Check the failing job; the count gate is intentional — never bypass it |
| Release not created (all jobs green) | Run was a `workflow_dispatch` dry run | Expected — the release job is tag-gated |
| Docker push fails | Insufficient permissions | Verify workflow permissions include `packages: write` |
| crates.io publish auth fails | Trusted Publishing config mismatch | On crates.io: crate Settings → Trusted Publishing must list repo `zircote/nsip`, workflow `publish.yml`, environment `copilot` |
| Registry crate bytes differ | CDN served stale/foreign bytes | Investigate before re-running; this gate exists to catch tampering |
| Changelog not updated | git-cliff config error | Check `cliff.toml` and the `fetch-depth: 0` checkout |
| Attestation fails | `id-token: write`/`attestations: write` missing on a job | Check the job-level permissions in `release.yml` |
| Homebrew tap not updated | `HOMEBREW_TAP_TOKEN` expired, or release authored with `GITHUB_TOKEN` | Rotate the PAT; ensure the release job uses the PAT |
| Tag push doesn't trigger workflows | Tag format wrong | Must match `v*.*.*` pattern exactly (e.g., `v1.0.0`, not `1.0.0`) |
