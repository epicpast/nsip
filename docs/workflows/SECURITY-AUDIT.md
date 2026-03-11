---
diataxis_type: reference
---
# Security Audit Workflow

## Overview

Runs `cargo audit` against the [RustSec advisory database](https://rustsec.org/)
to detect known vulnerabilities in Rust dependencies. Runs daily and on every
change to dependency files.

**Workflow:** `.github/workflows/security-audit.yml`  
**Trigger:** Daily schedule (00:00 UTC), push when `Cargo.toml` or `Cargo.lock`
change, manual  
**Required secrets:** None  
**Output:** Fails the workflow if any unignored advisory is found; creates
GitHub Issues for daily run failures

## Jobs

### `audit` — Enforced Audit

The primary job. Runs on every trigger and gates the push/PR pipeline.

```bash
cargo audit --deny warnings --ignore RUSTSEC-2023-0071
```

**Current ignores:**

| Advisory | Crate | Reason |
|----------|-------|--------|
| `RUSTSEC-2023-0071` | `rsa` (via `jsonwebtoken`) | Timing side-channel in RSA; this project only uses HMAC-SHA256, never RSA |

If `cargo audit` exits non-zero (advisory found), the job fails.

### `audit-full` — Informational Full Audit

Runs **only on the daily schedule** with `continue-on-error: true`. Executes
`cargo audit --deny warnings` without any ignores to surface advisories that
are currently suppressed. Failures here are informational and do not block PRs.

```bash
cargo audit --deny warnings
```

This job helps track when fixes become available for suppressed advisories.

## What `cargo audit` Checks

`cargo audit` reads `Cargo.lock` and compares every dependency against the
[RustSec Advisory Database](https://github.com/RustSec/advisory-db). It detects:

- **Vulnerabilities**: Known security vulnerabilities (CVEs and RustSec IDs)
- **Unmaintained crates**: Crates no longer receiving security updates
- **Unsound code**: Published advisories for unsoundness (not a CVE, but a
  safety concern)
- **Yanked versions**: Crate versions yanked from crates.io

## Running Locally

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit (matches CI — with project ignores)
cargo audit --deny warnings --ignore RUSTSEC-2023-0071

# Run full audit (no ignores, matches informational job)
cargo audit --deny warnings

# Output in JSON format for scripting
cargo audit --json
```

## Responding to Advisories

### 1. Update the dependency

The preferred fix. Update the affected crate to a patched version:

```bash
# Update a specific crate
cargo update -p <crate-name>

# Update all dependencies
cargo update
```

Then verify: `cargo audit --deny warnings`

### 2. Replace the dependency

If no patched version exists, replace the crate with an alternative:

```toml
# Cargo.toml
[dependencies]
# Replace affected-crate with a secure alternative
```

### 3. Add a temporary ignore (last resort)

Only if the advisory does not apply to this project's usage:

```bash
# In CI (security-audit.yml), add --ignore RUSTSEC-YYYY-XXXX
# Document the reason as a comment
```

Always document:
- Which advisory is ignored
- Why it does not apply to this project
- A ticket/PR to track resolution

See the [Security Response runbook](../runbooks/SECURITY-RESPONSE.md) for
the full vulnerability triage and disclosure process.

## Permissions

| Permission | Scope | Reason |
|-----------|-------|--------|
| `contents: read` | Repository | Checkout code |
| `issues: write` | Repository | Create issues for daily audit failures |

## Integration with Cargo Deny

The CI pipeline also runs `cargo deny check` (in `ci.yml`), which includes
its own advisory check using the same RustSec database. The two tools are
complementary:

| Tool | When | Scope |
|------|------|-------|
| `cargo audit` | Daily + on dependency change | Vulnerabilities, unmaintained, yanked |
| `cargo deny` | Every CI run | Licenses, advisories, banned crates, sources |

`cargo deny` is the **blocking** check on every PR; `cargo audit` provides
additional daily monitoring.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Advisory found, no fix available | Upstream patching in progress | Add a documented `--ignore` and track in an issue |
| Daily job creates duplicate issues | Persistent unresolved advisory | Add ignore or resolve the dependency |
| `cargo audit` not installed | Tool missing from cache | The workflow uses `.github/actions/install-cargo-tool` |
