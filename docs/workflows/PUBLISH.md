---
diataxis_type: reference
---
# Publish to crates.io Workflow

## Overview

Runs the full suite of pre-publish checks and publishes the `nsip` crate to
[crates.io](https://crates.io/crates/nsip) on every `v*.*.*` tag push.
Manual dispatch runs a dry run by default.

**Workflow:** `.github/workflows/publish.yml`  
**Trigger:** Push of a `v*.*.*` tag, manual (`workflow_dispatch`)  
**Required secrets:** `CARGO_REGISTRY_TOKEN` — crates.io API token  
**Environment:** `copilot` (provides secret access controls)  
**Permissions:** `contents: read`

## Jobs

### `publish`

Runs on `ubuntu-latest`, timeout 30 minutes.

| Step | Description |
|------|-------------|
| Checkout | Clones the repository |
| Rust setup | Stable toolchain with `rustfmt` and `clippy` |
| Install `cargo-deny` | For license and advisory checks |
| Verify package | `cargo package --list` + `cargo package --allow-dirty` |
| Pre-publish checks | `fmt`, `clippy`, `test`, `doc`, `deny` |
| Dry run | `cargo publish --dry-run` — always executes |
| Publish | `cargo publish --token $CARGO_REGISTRY_TOKEN` — only on `refs/tags/v*` |

## Pre-publish Checks

The workflow re-runs the full quality gate before publishing:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
cargo deny check
```

A failed check **blocks publication**. This prevents publishing broken or
non-compliant crates.

## Publishing Manually

For an out-of-band publish (e.g., to recover from a failed workflow):

```bash
# Ensure you are on the correct tag
git checkout v0.4.0

# Run pre-publish checks
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo deny check

# Dry run
cargo publish --dry-run

# Publish (requires a crates.io token)
cargo publish --token <your-token>
```

## crates.io API Token

Create a crates.io API token:

1. Log in to [crates.io](https://crates.io/)
2. Go to **Account Settings → API Tokens → New Token**
3. Name it `nsip-publish` and select the `publish-new` and `publish-update` scopes
4. Add it as the `CARGO_REGISTRY_TOKEN` repository secret

## Relationship to Release Pipeline

The [Release workflow](RELEASE.md) builds binaries and creates the GitHub
Release. This workflow handles the separate concern of publishing the library
crate to crates.io. Both are triggered by the same version tag.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `cargo publish` fails with 403 | Expired or revoked `CARGO_REGISTRY_TOKEN` | Rotate the token on crates.io |
| Version already published | Tag pushed twice or previous partial publish | Bump the patch version and re-tag |
| Dry run passes but publish fails | Network error or crates.io outage | Retry after a few minutes |
| Pre-publish check fails | Lint or test regression | Fix the issue in a patch release |

See also: [Releasing runbook](../runbooks/RELEASING.md).
