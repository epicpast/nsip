---
diataxis_type: how-to
---
# CI Troubleshooting

Common CI failure patterns and fixes for nsip. Use this runbook when a workflow fails on a pull request or push to `main`.

---

## General Debugging

### Reading Workflow Logs

1. Go to **Actions**: https://github.com/zircote/nsip/actions
2. Click the failed workflow run
3. Click the failed job (red X)
4. Expand the failed step to see the full log
5. Click **"Search logs"** (magnifying glass) to search for error messages

### CLI Approach

```bash
# List recent failed runs
gh run list --status failure --limit 10

# View a specific run
gh run view <run-id>

# View logs for failed steps only
gh run view <run-id> --log-failed

# Download full logs
gh run view <run-id> --log > ci-logs.txt

# Re-run failed jobs
gh run rerun <run-id> --failed

# Re-run the entire workflow
gh run rerun <run-id>
```

### Reproducing Locally

Most CI failures can be reproduced locally with the same commands CI uses:

```bash
# Run the full CI check suite (matches ci.yml)
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --verbose
cargo doc --no-deps --all-features
cargo deny check

# MSRV check (requires the specific toolchain)
rustup install 1.92
cargo +1.92 check --all-features
```

---

## Clippy Failures

**Workflow:** `ci.yml` (clippy job)
**Command:** `cargo clippy --all-targets --all-features -- -D warnings`

### Common Patterns

| Lint | Cause | Fix |
|---|---|---|
| `clippy::unwrap_used` | Called `.unwrap()` | Use `?`, `.unwrap_or()`, or `if let` |
| `clippy::expect_used` | Called `.expect()` | Use `?` with proper error types |
| `clippy::panic` | Used `panic!()` in library code | Return `Result` or `Option` |
| `clippy::todo` | Left `todo!()` in code | Implement the function or remove it |
| `clippy::dbg_macro` | Left `dbg!()` in code | Remove it or use proper logging |
| `clippy::print_stdout` | Used `println!()` | Use a logging crate or remove |
| `clippy::needless_pass_by_value` | Function takes ownership unnecessarily | Change parameter to `&T` |
| `clippy::redundant_clone` | Unnecessary `.clone()` | Remove the clone |
| `clippy::missing_errors_doc` | Public function returns Result without `# Errors` doc | Add `# Errors` section to doc comment |
| `clippy::missing_panics_doc` | Function can panic without `# Panics` doc | Add doc or remove the panic |
| `clippy::must_use_candidate` | Return value should have `#[must_use]` | Add the attribute |

### How to Fix

```bash
# Run clippy and see all warnings
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix what clippy can fix
cargo clippy --fix --all-targets --all-features

# Check a specific file
cargo clippy --all-features -- -D warnings 2>&1 | grep "crates/myfile.rs"
```

### When to Allow a Lint

Only suppress lints when there is a genuine reason. Add an inline allow with an explanation:

```rust
#[allow(clippy::too_many_arguments)]  // Builder pattern requires all fields
fn create_widget(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32) -> Widget {
    // ...
}
```

Never add blanket allows in `lib.rs` or `Cargo.toml` to silence clippy globally.

---

## Format Failures

**Workflow:** `ci.yml` (fmt job)
**Command:** `cargo fmt --all -- --check`

### Fix

```bash
# Format all code
cargo fmt --all

# Check what would change (without modifying)
cargo fmt --all -- --check

# Format a specific file
rustfmt src/lib.rs
```

### Editor Integration

Set up your editor to format on save:

**VS Code** (`.vscode/settings.json` -- already configured in this repo):
```json
{
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

**Neovim (with rust-analyzer):**
```lua
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.rs",
  callback = function() vim.lsp.buf.format() end,
})
```

### Configuration

This project uses `rustfmt.toml` (or settings in `Cargo.toml`). Key settings:

- **Line length:** 100 characters
- **Edition:** 2024

If `cargo fmt` produces unexpected results, check that your local Rust toolchain matches the stable channel used in CI.

---

## MSRV Failures

**Workflow:** `ci.yml` (msrv job)
**Command:** `cargo +1.92 check --all-features`
**MSRV:** Rust 1.92

### Common Causes

| Cause | Example | Fix |
|---|---|---|
| Used a newer language feature | `let-chains` (1.87+) | Check which version stabilized the feature |
| Dependency bumped its MSRV | `serde` requires newer Rust | Pin the dependency to an older version |
| Used a newer std API | `std::io::IsTerminal` (1.70+) | Use a polyfill or feature-gate |

### Diagnosing

```bash
# Install and test with the exact MSRV
rustup install 1.92
cargo +1.92 check --all-features

# Check what version stabilized a feature
# Look at the Rust release notes or use:
rustup doc --std
```

### Fixing

1. **If you used a newer feature:** Rewrite to use MSRV-compatible syntax
2. **If a dependency requires newer Rust:** Pin to the last compatible version in `Cargo.toml`:
   ```toml
   [dependencies]
   some-crate = ">=1.0, <1.5"  # 1.5 requires Rust 1.93+
   ```
3. **If MSRV needs to be bumped:** Update `rust-version` in `Cargo.toml`, update the MSRV check in `ci.yml`, and note it in the changelog as a potentially breaking change

---

## cargo-deny Failures

**Workflow:** `ci.yml` (deny job)
**Command:** `cargo deny check`
**Configuration:** `deny.toml`

### Advisory Failures

```
error[vulnerability]: RUSTSEC-2024-XXXX: <crate> - <description>
```

**Fix:** Update the affected dependency:
```bash
cargo update -p <crate-name>
cargo deny check advisories
```

**If no fix is available**, add a temporary exception in `deny.toml`:
```toml
[advisories]
ignore = [
    "RUSTSEC-2024-XXXX",  # No fix available; not exploitable in our usage. Revisit by YYYY-MM-DD.
]
```

### License Failures

```
error[rejected]: license 'GPL-3.0' is not in the allow list
```

**Fix options:**
1. Remove the dependency and find an alternative with a permissive license
2. If the license is acceptable, add it to the allow list in `deny.toml`:
   ```toml
   [licenses]
   allow = [
       # ...existing...
       "NEW-LICENSE-SPDX",
   ]
   ```

### Ban Failures

```
error[banned]: crate 'openssl' is banned
```

**Fix:** The crate (or one of its transitive dependencies) pulls in a banned crate. Check the dependency tree:
```bash
cargo tree -i openssl
```

Then either:
- Use an alternative crate that does not depend on the banned one
- Enable a feature flag that uses a different backend (e.g., `rustls` instead of `openssl`)

### Source Failures

```
error[unknown-registry]: crate 'foo' sourced from unknown registry
```

**Fix:** Only crates from crates.io are allowed. If you need a git dependency, add it to `deny.toml`:
```toml
[sources]
allow-git = ["https://github.com/owner/repo"]
```

### Running Specific Checks

```bash
cargo deny check advisories
cargo deny check licenses
cargo deny check bans
cargo deny check sources
```

---

## Test Failures

**Workflow:** `ci.yml` (test job, runs on ubuntu, macos, windows)
**Command:** `cargo test --all-features --verbose`

### Debugging Tips

```bash
# Run all tests with output
cargo test --all-features -- --nocapture

# Run a specific test
cargo test test_function_name

# Run tests in a specific module
cargo test module_name::

# Run tests matching a pattern
cargo test --all-features -- --test-threads=1 pattern

# Run only integration tests
cargo test --test integration_test

# Run only doc tests
cargo test --doc

# Show which tests are running
cargo test --all-features -- --list
```

### Platform-Specific Failures

If tests pass on one OS but fail on another:

| Symptom | Common cause | Fix |
|---|---|---|
| Path separator issues | Hardcoded `/` or `\` | Use `std::path::PathBuf` |
| Line ending issues | `\n` vs `\r\n` | Normalize with `.trim()` or use `\r?\n` in regex |
| Temp directory issues | Different temp path formats | Use `std::env::temp_dir()` |
| File permission issues | Unix permissions on Windows | Feature-gate Unix-specific code |

### Flaky Tests

If a test passes sometimes and fails sometimes:

1. Run it many times: `for i in $(seq 100); do cargo test test_name || break; done`
2. Check for race conditions in async code
3. Check for reliance on system time
4. Check for reliance on HashMap iteration order

---

## Release Workflow Failures

**Workflow:** `release.yml`
**Trigger:** Push of `v*.*.*` tag

### Build Failures

| Error | Cause | Fix |
|---|---|---|
| `Cargo.toml version mismatch` | Version in Cargo.toml does not match the tag | Ensure `version = "X.Y.Z"` matches tag `vX.Y.Z` |
| `linker 'aarch64-linux-gnu-gcc' not found` | Cross-compiler not installed | Check the ARM64 install step in the workflow |
| `error[E0658]: feature not available` | Target requires newer Rust | Check MSRV compatibility for all targets |
| `strip: unable to recognise` | Wrong strip binary for target | ARM64 Linux must use `aarch64-linux-gnu-strip` |

### Missing Release Assets

If the release is created but some binaries are missing:

```bash
# Check which assets exist
gh release view vX.Y.Z

# Re-run just the failed matrix job
gh run rerun <run-id> --failed
```

### Missing Secrets

If the publish or signing step fails:

```bash
# Verify secrets are configured (you can't read them, just confirm they exist)
gh secret list
```

Secrets needed:
- crates.io publishing uses Trusted Publishing (OIDC) — no secret; check the crates.io Trusted Publishing config instead
- `HOMEBREW_TAP_TOKEN` (environment `copilot`) for release creation and the Homebrew tap
- `GITHUB_TOKEN` is automatic

---

## Docker Build Failures

**Workflow:** `docker.yml`
**Trigger:** Push of a `v*.*.*` tag, or manual (`workflow_dispatch`)

### Common Issues

| Error | Cause | Fix |
|---|---|---|
| `failed to solve: Dockerfile: not found` | Missing Dockerfile | Ensure `Dockerfile` exists in repo root |
| `COPY failed: file not found` | Wrong path in Dockerfile | Check paths match `crates/` structure |
| `denied: denied` on push | Missing permissions | Verify `packages: write` in workflow and "Read and write permissions" in repo settings |
| `ERROR: Multiple platforms not supported` | Buildx not configured | The workflow sets up Buildx; check that step |
| Build arg mismatch | `RUST_VERSION` arg wrong | Check `build-args` in the workflow matches a valid Rust version |

### Testing Docker Locally

```bash
# Build for current platform only
docker build -t nsip:test .

# Run the test image
docker run --rm nsip:test --version

# Build for multiple platforms (requires buildx)
docker buildx build --platform linux/amd64,linux/arm64 -t nsip:test .
```

---

## Dependabot Merge Conflicts

When Dependabot PRs have merge conflicts in `Cargo.lock`:

### Automatic Resolution

Close and reopen the PR to trigger Dependabot to rebase:

```bash
gh pr close <number>
gh pr reopen <number>
```

Or comment on the PR:

```bash
gh pr comment <number> --body "@dependabot rebase"
```

### Manual Resolution

```bash
# Checkout the Dependabot branch
gh pr checkout <number>

# Merge main into it
git merge main

# Resolve Cargo.lock conflicts by regenerating it
git checkout --theirs Cargo.lock
cargo update

# Or if specific crate needs updating
cargo update -p <crate-from-dependabot-pr>

# Commit and push
git add Cargo.lock
git commit -m "chore: resolve merge conflict in Cargo.lock"
git push
```

### Useful Dependabot Commands

Comment these on any Dependabot PR:

| Command | Effect |
|---|---|
| `@dependabot rebase` | Rebase the PR on the latest base branch |
| `@dependabot recreate` | Close and recreate the PR from scratch |
| `@dependabot merge` | Merge the PR (if CI passes) |
| `@dependabot squash and merge` | Squash merge the PR |
| `@dependabot cancel merge` | Cancel a pending auto-merge |
| `@dependabot ignore this major version` | Stop updates for this major version |
| `@dependabot ignore this dependency` | Stop all updates for this dependency |

---

## Documentation Failures

**Workflow:** `ci.yml` (doc job)
**Command:** `cargo doc --no-deps --all-features`
**Environment:** `RUSTDOCFLAGS="-D warnings"`

### Common Causes

| Error | Cause | Fix |
|---|---|---|
| `unresolved link` | Broken intra-doc link `[`Type`]` | Fix the type name or path in the doc comment |
| `missing documentation` | Public item without `///` doc | Add documentation to the public item |
| `private item in public doc` | Doc references a private type | Make the type public or remove the reference |
| `broken code example` | Doc test does not compile | Fix the code in the ```` ```rust ```` block |

### Testing Documentation Locally

```bash
# Build docs with warnings as errors (matches CI)
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Open the docs in a browser
cargo doc --open

# Run only doc tests
cargo test --doc
```

---

## Coverage Failures

**Workflow:** `ci.yml` (coverage job) -- this job does not block merging (`fail_ci_if_error: false` on Codecov upload)

If coverage generation fails:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage locally
cargo llvm-cov --all-features --lcov --output-path lcov.info

# Generate an HTML report
cargo llvm-cov --all-features --html
open target/llvm-cov/html/index.html
```

---

## Quick Reference: CI Jobs and Commands

| CI Job | Local equivalent | Blocks merge? |
|---|---|---|
| Format | `cargo fmt --all -- --check` | Yes |
| Clippy | `cargo clippy --all-targets --all-features -- -D warnings` | Yes |
| Test (ubuntu) | `cargo test --all-features --verbose` | Yes |
| Test (macos) | Same (run on macOS) | Yes |
| Test (windows) | Same (run on Windows) | Yes |
| Documentation | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | Yes |
| Cargo Deny | `cargo deny check` | Yes |
| MSRV | `cargo +1.92 check --all-features` | Yes |
| Coverage | `cargo llvm-cov --all-features` | No |
| Security Audit | `cargo audit --deny warnings` | No (scheduled) |

### Run Everything at Once

```bash
cargo fmt --all -- --check \
  && cargo clippy --all-targets --all-features -- -D warnings \
  && cargo test --all-features --verbose \
  && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features \
  && cargo deny check \
  && cargo +1.92 check --all-features
```
