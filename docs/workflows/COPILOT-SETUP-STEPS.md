---
diataxis_type: reference
---
# Copilot Setup Steps Workflow

## Overview

Prepares a reproducible Rust development environment for GitHub Copilot coding
agent sessions. The workflow installs all tooling required to lint, build, and
test the project so that Copilot can make code changes and verify them
immediately.

**Workflow:** `.github/workflows/copilot-setup-steps.yml`  
**Trigger:** Manual (`workflow_dispatch`) — invoked automatically by GitHub
Copilot when it starts a coding agent task  
**Required secrets:** None  
**Timeout:** 20 minutes

## What It Sets Up

| Step | Details |
|------|---------|
| Checkout | Clones the repository |
| Rust toolchain | Stable toolchain with `clippy` and `rustfmt` components, with Cargo cache |
| `cargo-deny` | Installed via `.github/actions/install-cargo-tool` |
| Dependencies | `cargo fetch` — pre-downloads all crate dependencies |
| Release binary | `cargo build --release` — builds `nsip` and populates the Cargo cache |
| PATH | Adds `target/release/` to `GITHUB_PATH` so `nsip` is available as a command |

After these steps complete, the Copilot agent can run `nsip`, `cargo test`,
`cargo clippy`, and `cargo fmt` without any additional setup.

## Why This Workflow Exists

GitHub Copilot coding agents run in ephemeral containers. This workflow acts
as the environment bootstrap, equivalent to a `devcontainer.json` setup step
but for GitHub-hosted runners. By pre-building the release binary and warming
the Cargo cache, subsequent Copilot tool invocations are significantly faster.

## Relationship to devcontainer

The repository also ships a `.devcontainer/devcontainer.json` for local and
Codespaces use. The copilot-setup-steps workflow targets the GitHub Copilot
hosted-runner environment where devcontainers are not available.

## Permissions

| Permission | Scope | Reason |
|-----------|-------|--------|
| `contents: read` | Repository | Checkout code |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `cargo build` fails | Compilation error in `main` branch | Fix the build before triggering Copilot tasks |
| `nsip` not found after setup | PATH not propagated | Verify `echo "..." >> "$GITHUB_PATH"` step ran successfully |
| Slow setup | Empty Cargo cache | Subsequent runs will be faster once cache is seeded |
