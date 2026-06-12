---
diataxis_type: reference
---
# Publish to crates.io Workflow

## Overview

Runs the full suite of pre-publish checks and publishes the `nsip` crate to
[crates.io](https://crates.io/crates/nsip) on every `v*.*.*` tag push via
[Trusted Publishing](https://crates.io/docs/trusted-publishing) (OIDC — no
stored registry token). After publishing, it downloads the `.crate` the
registry serves, asserts it is byte-identical to the locally packaged
crate, and attaches SLSA build provenance to it. Manual dispatch runs a dry
run (publish steps are tag-gated).

**Workflow:** `.github/workflows/publish.yml`  
**Trigger:** Push of a `v*.*.*` tag, manual (`workflow_dispatch`)  
**Required secrets:** none (Trusted Publishing)  
**Environment:** `copilot` (bound in the crates.io Trusted Publishing config)  
**Permissions:** `contents: read`; job-level `id-token: write`,
`attestations: write`

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
| Authenticate | `rust-lang/crates-io-auth-action` mints a short-lived OIDC token — only on `refs/tags/v*` |
| Publish | `cargo publish` with the minted token — only on `refs/tags/v*` |
| Byte-verify | Downloads the published `.crate` from `static.crates.io` and asserts SHA-256 equality with the local package |
| Attest | `actions/attest-build-provenance` over the registry-served `.crate` |

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

## Trusted Publishing Configuration

One-time setup on crates.io (already done for this crate):

1. Log in to [crates.io](https://crates.io/) and open the `nsip` crate
2. Go to **Settings → Trusted Publishing → Add**
3. GitHub repository `zircote/nsip`, workflow `publish.yml`, environment
   `copilot`

The workflow exchanges its GitHub OIDC token for a short-lived (~30 min)
crates.io token at run time. There is no long-lived secret to rotate or
leak. (For an out-of-band manual publish you still need a personal
crates.io API token.)

## Relationship to Release Pipeline

The [Release workflow](RELEASE.md) builds binaries and creates the GitHub
Release. This workflow handles the separate concern of publishing the library
crate to crates.io. Both are triggered by the same version tag.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Authentication step fails | Trusted Publishing config mismatch | crates.io crate Settings → Trusted Publishing must list repo `zircote/nsip`, workflow `publish.yml`, environment `copilot` |
| `registry crate bytes differ` | CDN served stale/foreign bytes | Investigate before re-running; this gate exists to catch tampering |
| Version already published | Tag pushed twice or previous partial publish | Bump the patch version and re-tag |
| Dry run passes but publish fails | Network error or crates.io outage | Retry after a few minutes |
| Pre-publish check fails | Lint or test regression | Fix the issue in a patch release |

See also: [Releasing runbook](../runbooks/RELEASING.md).
