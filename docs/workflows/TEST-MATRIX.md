---
diataxis_type: reference
---
# Test Matrix - Multi-Platform Testing

## Overview

Comprehensive test matrix validating code across multiple platforms, Rust versions, and feature combinations.

**Workflow:** `.github/workflows/test-matrix.yml`
**Platforms:** Linux, macOS, Windows
**Rust Versions:** Stable, Beta, Nightly, MSRV (1.80)
**Triggers:** Push to main, PRs, Weekly schedule

## Matrix Configuration

The workflow tests these combinations:

### Operating Systems
- **ubuntu-latest** - Linux (primary platform)
- **macos-latest** - macOS (Apple Silicon + Intel)
- **windows-latest** - Windows (x64)

### Rust Toolchains
- **stable** - Latest stable Rust
- **beta** - Beta channel (upcoming stable)
- **nightly** - Nightly builds (experimental features)
- **1.80** - MSRV (Minimum Supported Rust Version)

### Feature Combinations
- `--all-features` - All features enabled
- `--no-default-features` - Minimal build
- Default - Standard feature set

**Total Jobs:** ~12-15 (optimized to skip redundant combinations)

## What Gets Tested

### For All Combinations

1. **Build**: `cargo build`
2. **Unit Tests**: `cargo test`
3. **Doc Tests**: `cargo test --doc`

### Stable + Ubuntu Only

4. **Formatting**: `cargo fmt --check`
5. **Linting**: `cargo clippy -- -D warnings`
6. **Documentation**: `cargo doc --no-deps`

### Separate Jobs

7. **Integration Tests**: `cargo test --test '*'`
8. **Miri**: Undefined behavior detection (nightly)

## Understanding Results

### Success (✅)

All platforms and versions pass. Code is portable and compatible.

### Partial Failure (⚠️)

```
ubuntu-latest / stable: ✅
macos-latest / stable: ❌
windows-latest / stable: ✅
```

**Action:** Fix macOS-specific issue (likely path, filesystem, or API difference).

### MSRV Failure (❌)

```
ubuntu-latest / 1.80: ❌
ubuntu-latest / stable: ✅
```

**Action:** Code uses features newer than MSRV. Either:
- Update MSRV in `Cargo.toml`
- Remove incompatible features
- Add feature flags

## Platform-Specific Issues

### Path Separators

```rust
// ❌ Bad - Unix-only
let path = format!("{}/file.txt", dir);

// ✅ Good - Cross-platform
use std::path::PathBuf;
let path = PathBuf::from(dir).join("file.txt");
```

### Line Endings

```rust
// ❌ Bad - Assumes Unix line endings
assert_eq!(content, "line1\nline2\n");

// ✅ Good - Platform-agnostic
assert_eq!(content.lines().count(), 2);
```

### Filesystem Case Sensitivity

```rust
// ❌ Bad - macOS/Windows are case-insensitive
let file = File::open("Config.toml")?;

// ✅ Good - Exact case matching
let file = File::open("config.toml")?;
```

### Process Signals

```rust
// ❌ Bad - SIGTERM not available on Windows
std::process::Command::new("kill")
    .arg("-TERM")
    .spawn()?;

// ✅ Good - Use cross-platform crate
use subprocess::Popen;
```

## MSRV (Minimum Supported Rust Version)

Current MSRV: **1.80**

### Checking MSRV Locally

```bash
# Install specific Rust version
rustup install 1.80

# Test with MSRV
cargo +1.80 check
cargo +1.80 test
```

### Common MSRV Issues

#### Edition Features

```toml
# Cargo.toml
[package]
edition = "2024"  # Requires Rust 1.85+
rust-version = "1.80"  # Conflict!
```

**Fix:** Use `edition = "2021"` or update MSRV.

#### Newer APIs

```rust
// std::io::Read::read_buf() added in 1.78
let mut buf = Vec::new();
reader.read_buf(&mut buf)?;
```

**Fix:** Use older APIs or bump MSRV.

#### Dependency MSRV

```toml
[dependencies]
serde = "1.0"  # May require newer Rust
```

**Fix:** Pin to older compatible versions or update MSRV.

## Feature Testing

### All Features

```yaml
matrix:
  features: ['--all-features']
```

Tests maximum feature combination. Ensures no feature conflicts.

### No Default Features

```yaml
matrix:
  features: ['--no-default-features']
```

Tests minimal build. Ensures optional features don't leak into default.

### Specific Features

```bash
# Test individual features locally
cargo test --no-default-features --features feature1
cargo test --no-default-features --features feature2
cargo test --features feature1,feature2
```

## Integration Tests

Separate job runs integration tests:

```bash
cargo test --test '*' --verbose
```

**Location:** `tests/*.rs`

**Purpose:** Test public API integration, not internal units.

## Miri - Undefined Behavior Detection

Miri detects:
- Use-after-free
- Out-of-bounds memory access
- Data races (unsafe code)
- Invalid pointer arithmetic
- Uninitialized memory reads

### Running Miri Locally

```bash
# Install miri component
rustup +nightly component add miri

# Run miri tests
cargo +nightly miri test

# Run specific test
cargo +nightly miri test test_name
```

### Miri Limitations

- **Slow**: 100-1000x slower than normal tests
- **No I/O**: File/network operations unsupported
- **Nightly only**: Requires nightly Rust

### Skip Tests in Miri

```rust
#[test]
#[cfg_attr(miri, ignore)]  // Skip in miri
fn test_file_io() {
    // File I/O test
}
```

## Optimization: Reducing CI Time

### Exclude Redundant Combinations

```yaml
matrix:
  exclude:
    - os: macos-latest
      rust: beta
    - os: windows-latest
      rust: beta
```

Only test beta on Linux to save time.

### Cache Dependencies

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-${{ matrix.rust }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

Speeds up subsequent runs.

### Fail Fast

```yaml
strategy:
  fail-fast: false  # Continue all jobs even if one fails
```

See all failures, not just the first.

## Local Multi-Platform Testing

### Using Cross

```bash
# Install cross for cross-compilation
cargo install cross

# Test for different targets
cross test --target x86_64-pc-windows-gnu
cross test --target x86_64-apple-darwin
```

### Docker

```bash
# Test in Linux container
docker run --rm -v $(pwd):/app -w /app rust:latest cargo test
```

## Troubleshooting

### Tests Fail Only on Windows

**Common Causes:**
- Path separators (`/` vs `\`)
- Case-insensitive filesystem
- Different temporary directory location
- Line endings (CRLF vs LF)

**Debug:**
```rust
#[cfg(windows)]
#[test]
fn test_windows_specific() {
    println!("Temp dir: {:?}", std::env::temp_dir());
}
```

### Tests Fail on macOS Only

**Common Causes:**
- Case-insensitive but case-preserving filesystem
- Different system APIs
- Apple-specific security restrictions

### MSRV Test Fails

1. **Check dependencies**: `cargo tree --edges no-dev` for version conflicts
2. **Check APIs**: Search for recently stabilized features
3. **Update or lower MSRV**: Choose based on requirement

## Best Practices

1. **Test locally first**: Use `cross` or Docker before pushing
2. **Fix MSRV issues early**: Don't let them accumulate
3. **Use PathBuf**: Always for cross-platform file paths
4. **Skip platform-specific tests**: Use `#[cfg(target_os = "linux")]`
5. **Monitor CI time**: Optimize matrix if jobs take too long

## Links

- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Miri Documentation](https://github.com/rust-lang/miri)
- [GitHub Actions Matrix](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs)
