# GitHub Actions Workflow Quick Reference

## 🚀 Quick Start

### Using Composite Actions in Your Workflow

**Before** (old way - DON'T DO THIS):
```yaml
steps:
  - uses: actions/checkout@...
  - uses: dtolnay/rust-toolchain@...
    with:
      toolchain: stable
  - uses: actions/cache@...
    with:
      path: |
        ~/.cargo/registry
        ~/.cargo/git
        target
      key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
  - run: cargo build
```

**After** (optimized - DO THIS):
```yaml
steps:
  - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2
  - uses: ./.github/actions/setup-rust-cached
    with:
      toolchain: stable
      cache-key: my-job
  - run: cargo build
```

---

## 📋 Common Patterns

### Basic Rust CI Job
```yaml
jobs:
  build:
    name: Build and Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2
      
      - uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: stable
          cache-key: build
      
      - run: cargo build --release
      - run: cargo test --all-features
```

### Clippy Linting
```yaml
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2
      
      - uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: stable
          components: clippy
          cache-key: clippy
      
      - run: cargo clippy --all-targets --all-features -- -D warnings
```

### Formatting Check
```yaml
jobs:
  fmt:
    name: Format Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2
      
      - uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: stable
          components: rustfmt
          cache-key: fmt
      
      - run: cargo fmt --all -- --check
```

### Security Audit
```yaml
jobs:
  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2
      
      - uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: stable
          cache-key: audit
      
      - uses: ./.github/actions/install-cargo-tool
        with:
          tool: cargo-audit
      
      - run: cargo audit --deny warnings
```

### Cross-Platform Testing
```yaml
jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2
      
      - uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: stable
          cache-key: test-${{ matrix.os }}
      
      - run: cargo test --all-features
```

### MSRV Check
```yaml
jobs:
  msrv:
    name: Minimum Supported Rust Version
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2
      
      - uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: "1.92"  # Your MSRV
          cache-key: msrv
      
      - run: cargo check --all-features
```

---

## 🔧 Workflow Template

Copy this template for new workflows:

```yaml
---
name: My Workflow

"on":
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:

# Cancel outdated runs for PRs
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

# Minimal permissions
permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
  CARGO_INCREMENTAL: 0
  RUSTFLAGS: "-D warnings"

jobs:
  my-job:
    name: My Job
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - name: Checkout repository
        uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683  # v4.2.2

      - name: Setup Rust with caching
        uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: stable
          cache-key: my-job

      - name: Run my commands
        run: |
          cargo build
          cargo test
```

---

## 🎯 Best Practices Checklist

### ✅ Required for ALL Workflows
- [ ] Use `permissions:` with minimal required permissions
- [ ] Use composite actions instead of direct action references
- [ ] SHA-pin all external actions with version comment
- [ ] Use concurrency control on PR workflows
- [ ] Add timeout to long-running jobs (30-45 mins)
- [ ] Use consistent action versions

### ✅ For Rust Workflows
- [ ] Use `./.github/actions/setup-rust-cached`
- [ ] Set unique `cache-key` per job
- [ ] Set `CARGO_TERM_COLOR: always`
- [ ] Set `CARGO_INCREMENTAL: 0` for CI
- [ ] Set `RUSTFLAGS: "-D warnings"`

### ✅ For Tool Installation
- [ ] Use `./.github/actions/install-cargo-tool`
- [ ] Don't install multiple tools in one step

### ⚠️ Common Mistakes to Avoid
- ❌ Using tag references (e.g., `@v4` instead of SHA)
- ❌ Not setting `cache-key` (causes cache conflicts)
- ❌ Overly permissive permissions
- ❌ No concurrency control on PR workflows
- ❌ Bypassing composite actions
- ❌ Inconsistent action versions

---

## 🔍 Debugging Tips

### Cache Not Working?
```yaml
# Check cache-hit output
- uses: ./.github/actions/setup-rust-cached
  id: cache
  with:
    cache-key: my-job

- name: Debug cache
  run: echo "Cache hit: ${{ steps.cache.outputs.cache-hit }}"
```

### Want to See Rustc Version?
```yaml
- uses: ./.github/actions/setup-rust-cached
  id: rust
  with:
    toolchain: stable

- name: Show version
  run: echo "Rustc version: ${{ steps.rust.outputs.rustc-version }}"
```

### Need Multiple Components?
```yaml
- uses: ./.github/actions/setup-rust-cached
  with:
    toolchain: stable
    components: clippy,rustfmt,llvm-tools-preview
    cache-key: full
```

### Need Cross-Compilation?
```yaml
- uses: ./.github/actions/setup-rust-cached
  with:
    toolchain: stable
    targets: wasm32-unknown-unknown,aarch64-unknown-linux-gnu
    cache-key: cross
```

---

## 📊 Current Standard Action Versions

| Action | SHA | Version | Use Via |
|--------|-----|---------|---------|
| actions/checkout | `11bd71901bbe5b1630ceea73d27597364c9af683` | v4.2.2 | Direct |
| actions/upload-artifact | `ea3f73d3e6f8268b4a40da165a72ca6a06e37770` | v4.6.2 | Direct |
| dtolnay/rust-toolchain | `7b1c307e0dcbda6122208f10795a713336a9b35a` | master | Composite |
| actions/cache | `cdf6c1fa76f9f475f3d7449005a359c84ca0f306` | v5.0.3 | Composite |
| taiki-e/install-action | `f176c07a0a40cbfdd08ee9aa8bf1655701d11e69` | v2.67.25 | Composite |

**Note**: Actions marked "Composite" should be used via composite actions, not directly.

---

## 🚦 Validation

Run this before committing workflow changes:
```bash
./scripts/validate-workflow-optimization.sh
```

---

## 📚 Documentation

- **Detailed Analysis**: [WORKFLOW_ANALYSIS.md](./WORKFLOW_ANALYSIS.md)
- **Optimization Summary**: [WORKFLOW_OPTIMIZATION_SUMMARY.md](./WORKFLOW_OPTIMIZATION_SUMMARY.md)
- **Composite Actions**: [.github/actions/README.md](./.github/actions/README.md)
- **Example Workflow**: [.github/workflows/ci.yml.new](./.github/workflows/ci.yml.new)

---

## 🆘 Need Help?

1. Check the [composite actions README](.github/actions/README.md)
2. Look at [security-audit.yml](.github/workflows/security-audit.yml) for a simple example
3. Look at [ci.yml.new](.github/workflows/ci.yml.new) for a complete example
4. Run the validation script for specific errors

---

## 🎓 Learning Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Composite Actions Guide](https://docs.github.com/en/actions/creating-actions/creating-a-composite-action)
- [Security Hardening](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)
- [Caching Dependencies](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows)

---

**Last Updated**: February 2025  
**Version**: 1.0 (Phase 1 Complete)
