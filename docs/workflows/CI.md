---
diataxis_type: reference
---
# CI Pipeline

## Overview

The primary quality gate for every change to the repository. The CI pipeline
ensures code is correctly formatted, lint-clean, tested on all supported
platforms, documented, dependency-safe, and MSRV-compatible before any merge.

**Workflow:** `.github/workflows/ci.yml`  
**Trigger:** Push to `develop`/`main`, pull requests to `develop`/`main`, manual  
**Required secrets:** `CODECOV_TOKEN` (optional — coverage upload fails gracefully without it)  
**Coverage target:** ≥ 90% line coverage

## Jobs

The pipeline runs these jobs in parallel (except `all-checks-pass`):

| Job | Timeout | Description |
|-----|---------|-------------|
| `fmt` | 10 min | Checks code formatting with `cargo fmt` |
| `clippy` | 20 min | Lints with Clippy (pedantic + nursery, warnings as errors) |
| `test` | 30 min | Runs full test suite on Linux, macOS, and Windows |
| `doc` | 15 min | Builds Rustdoc with `RUSTDOCFLAGS="-D warnings"` |
| `deny` | 10 min | Checks licenses and advisories with `cargo deny` |
| `msrv` | 20 min | Verifies the code compiles on the minimum supported Rust version (1.92) |
| `coverage` | 30 min | Generates LCOV coverage and uploads to Codecov |
| `all-checks-pass` | — | Gate job: fails if any of the above jobs fail |

### Format (`fmt`)

Runs `cargo fmt --all -- --check`. All code must be formatted with the project's
`rustfmt.toml` configuration (100-char lines, edition 2024 idioms).

**Fix locally:**
```bash
cargo fmt --all
```

### Clippy (`clippy`)

Runs Clippy with all targets and features enabled, treating warnings as errors:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Enforces pedantic and nursery lints configured in `clippy.toml`. Key denied
lints include `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`,
`dbg_macro`, `print_stdout`, and `print_stderr`.

**Fix locally:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Test (`test`)

Runs the full test suite across three operating systems using a `fail-fast: false`
matrix:

```bash
cargo test --all-features --verbose
```

**Run locally:**
```bash
cargo test --all-features
```

### Documentation (`doc`)

Builds Rustdoc with warnings as errors:

```bash
cargo doc --no-deps --all-features
```

All public items must have doc comments. Broken doc links fail the build.

**Run locally:**
```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

### Cargo Deny (`deny`)

Checks all dependencies against:
- **License policy**: only permissive licenses (MIT, Apache-2.0, BSD, etc.)
- **Security advisories**: via the RustSec advisory database
- **Banned crates**: `openssl` (use rustls), `atty` (use `std::io::IsTerminal`)

Configuration is in `deny.toml`.

**Run locally:**
```bash
cargo deny check
```

### MSRV Check (`msrv`)

Verifies the crate compiles on Rust 1.92 (the minimum supported Rust version):

```bash
cargo check --all-features
```

The MSRV is declared in `Cargo.toml` as `rust-version = "1.92"`.

**Run locally:**
```bash
rustup toolchain install 1.92
cargo +1.92 check --all-features
```

### Coverage (`coverage`)

Generates code coverage using `cargo-llvm-cov`, enforces ≥ 90% line coverage,
and uploads an LCOV report to Codecov:

```bash
cargo llvm-cov --all-features --lcov --output-path lcov.info
cargo llvm-cov --all-features --fail-under-lines 90
```

**Run locally:**
```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
cargo llvm-cov --all-features --html --open
```

### All Checks Pass (`all-checks-pass`)

A synthetic gate job that runs with `if: always()` and depends on every other
job. Branch protection rules target `all-checks-pass` so that a single status
check covers the entire pipeline. It exits with a non-zero code if any upstream
job did not succeed.

## Merge Readiness

Configure your branch protection rule to require the **`All Checks Pass`**
status check. This ensures all jobs must succeed before merging.

## Environment Variables

| Variable | Value | Purpose |
|----------|-------|---------|
| `CARGO_TERM_COLOR` | `always` | Colorized Cargo output in logs |
| `CARGO_INCREMENTAL` | `0` | Disable incremental compilation (reproducible builds) |
| `RUSTFLAGS` | `-D warnings` | Treat all Rust compiler warnings as errors |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `fmt` fails | Unformatted code | Run `cargo fmt --all` |
| `clippy` fails | Lint violation | Run `cargo clippy --all-targets --all-features -- -D warnings` |
| `test` fails on Windows only | Path separator or line-ending issue | Use `Path` types and platform-neutral assertions |
| `doc` fails | Missing doc comment or broken link | Add `///` docs to all public items |
| `deny` fails | New dependency with incompatible license or advisory | Update `deny.toml` exemptions or replace the dependency |
| `msrv` fails | Used a feature stabilized after 1.92 | Check stabilization version in Rust docs |
| `coverage` fails | Line coverage below 90% | Add tests for uncovered code paths |

See also: [CI Troubleshooting runbook](../runbooks/CI-TROUBLESHOOTING.md).
