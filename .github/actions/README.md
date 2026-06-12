# GitHub Actions Composite Actions

This directory contains reusable composite actions for this repository's workflows.

## Available Actions

### `setup-rust-cached`

Sets up Rust toolchain with intelligent cargo caching for faster builds.

**Usage:**
```yaml
- uses: ./.github/actions/setup-rust-cached
  with:
    toolchain: stable  # stable, beta, nightly, or version like "1.92"
    components: clippy,rustfmt  # Optional
    targets: wasm32-unknown-unknown  # Optional
    cache-key: my-job  # Optional, defaults to "default"
```

**Benefits:**
- ✅ Consistent Rust toolchain installation across all workflows
- ✅ Optimized cargo caching with intelligent restore keys
- ✅ Single place to update action versions
- ✅ Includes rustc version in cache key for accuracy

**Outputs:**
- `cache-hit`: Boolean indicating if cache was hit
- `rustc-version`: Installed rustc version

**Example:**
```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout  # pin to a full 40-char SHA
      
      - name: Setup Rust
        uses: ./.github/actions/setup-rust-cached
        with:
          toolchain: stable
          components: clippy
          cache-key: build
      
      - run: cargo build --release
```

---

### `install-cargo-tool`

Installs a cargo tool using taiki-e/install-action with consistent versioning.

**Usage:**
```yaml
- uses: ./.github/actions/install-cargo-tool
  with:
    tool: cargo-deny  # Required: tool name
```

**Supported Tools:**
- `cargo-deny` - Dependency vulnerability scanner
- `cargo-llvm-cov` - Code coverage tool
- `cargo-audit` - Security audit tool
- `cargo-geiger` - Unsafe code detector
- `cargo-bloat` - Binary size analyzer
- Any tool supported by taiki-e/install-action

**Benefits:**
- ✅ Consistent tool installation method
- ✅ Automatic version management
- ✅ Single place to update install-action version

**Example:**
```yaml
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout  # pin to a full 40-char SHA
      
      - name: Install cargo-audit
        uses: ./.github/actions/install-cargo-tool
        with:
          tool: cargo-audit
      
      - run: cargo audit --deny warnings
```

---

## Why Composite Actions?

### Before (Problems):
```yaml
# Repeated in approximately 29 places across workflows (estimate)!
- uses: dtolnay/rust-toolchain  # pinned to differing SHAs in each copy
  with:
    toolchain: stable
- uses: actions/cache  # another SHA to keep in sync
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

**Issues:**
- 🔴 ~29 places to update when action versions change (estimate)
- 🔴 Inconsistent SHA versions (3 different SHAs for same action!)
- 🔴 Suboptimal cache keys
- 🔴 Maintenance nightmare

### After (Solution):
```yaml
# One line does it all!
- uses: ./.github/actions/setup-rust-cached
  with:
    toolchain: stable
```

**Benefits:**
- ✅ Update once, applies everywhere
- ✅ Consistent versions guaranteed
- ✅ Optimized cache strategy
- ✅ Easy maintenance

---

## Impact

> The figures below are approximate estimates, not measured benchmarks.

### Maintenance
- **Before**: approximately 48 places to update actions
- **After**: 1 place to update
- **Time saved**: roughly 90% (estimate)

### Consistency
- **Before**: 3 different SHAs for same action
- **After**: 100% consistent
- **Security**: Reduced attack surface

### Performance
- **Before**: ~60% cache hit rate (estimate)
- **After**: ~80% cache hit rate (estimate)
- **Speed**: approximately 15-20% faster builds (estimate)

---

## Maintenance

### Updating Action Versions

To update an action version used by these composite actions:

1. **Update the SHA in the composite action:**
   ```bash
   # Edit .github/actions/setup-rust-cached/action.yml
   # or .github/actions/install-cargo-tool/action.yml
   ```

2. **Test in a single workflow first:**
   ```bash
   # Test in a fast workflow like security-audit.yml
   ```

3. **Automatically applies to all workflows using it!**

### Finding Latest SHAs

```bash
# Get latest commit SHA for an action
gh api repos/actions/checkout/commits/main --jq '.sha'

# Or visit: https://github.com/actions/checkout/commits/main
```

### Testing Changes

```bash
# Validate all workflows
./scripts/validate-workflow-optimization.sh

# Test specific workflow locally with act (if installed)
act -j build
```

---

## Best Practices

### DO
- Use `cache-key` input to differentiate job-specific caches
- Pin composite action file updates in commits with SHA references
- Test composite action changes in one workflow before rollout
- Document any custom inputs/outputs you add

### DON'T
- Don't bypass composite actions unless necessary
- Don't create workflow-specific composite actions (keep them general)
- Don't forget to update SHA comments when updating actions
- Don't use tag references in composite actions (use SHAs)

---

## Debugging

### Cache Not Hitting?
Check the cache key includes rustc version:
```yaml
# Composite action includes this automatically:
key: ...-${{ steps.rustc-version.outputs.version }}
```

### Wrong Rust Version?
Verify the toolchain input:
```yaml
- uses: ./.github/actions/setup-rust-cached
  with:
    toolchain: "1.92"  # Exact version needed
```

### Component Not Found?
Check comma-separated format (no spaces):
```yaml
components: clippy,rustfmt  # ✓ Correct
components: clippy, rustfmt  # ✗ Wrong
```

---

## Further Reading

- [GitHub Actions: Creating Composite Actions](https://docs.github.com/en/actions/creating-actions/creating-a-composite-action)
- [Caching Dependencies](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows)
- [Security Hardening](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)

---

## Contributing

When creating new workflows, **always use these composite actions** instead of directly using:
- `dtolnay/rust-toolchain` → Use `setup-rust-cached` instead
- `taiki-e/install-action` → Use `install-cargo-tool` instead
- `actions/cache` for cargo → Use `setup-rust-cached` instead

This ensures consistency and makes future maintenance easier!
