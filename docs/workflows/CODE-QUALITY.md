---
diataxis_type: reference
---
# Code Quality Metrics

## Overview

Automated collection of code quality metrics including unsafe code detection, binary size analysis, and documentation coverage.

**Workflow:** `.github/workflows/code-quality.yml`  
**Tools:** `cargo-geiger`, `cargo-bloat`, `rustdoc`  
**Output:** Markdown report artifact

## Metrics Collected

### 1. Unsafe Code Analysis (cargo-geiger)

Detects unsafe code usage:
- Unsafe function calls
- Unsafe blocks
- Unsafe trait implementations
- Dependencies with unsafe code

**Why it matters:** Unsafe code bypasses Rust's safety guarantees.

### 2. Binary Size Analysis (cargo-bloat)

Analyzes what contributes to binary size:
- Size by crate
- Size by function
- Identifies bloat

**Why it matters:** Smaller binaries = faster downloads, less memory.

### 3. Documentation Coverage

Checks documentation completeness:
- Missing doc comments
- Broken doc links
- Doc test failures

**Why it matters:** Well-documented APIs are easier to use.

## Usage

### Local Analysis

```bash
# Install tools
cargo install cargo-geiger cargo-bloat

# Run unsafe code analysis
cargo geiger --all-features

# Analyze binary size
cargo build --release
cargo bloat --release --crates

# Check documentation
cargo doc --no-deps --all-features
```

### CI Reports

The workflow generates a combined report:

**Access:** Actions → Code Quality Metrics → Artifacts → code-quality-metrics

**Example Report:**
```markdown
## Unsafe Code Analysis
Functions  Expressions  Impls  Traits  Methods  Dependency
0/10       0/100        0/5    0/2     0/20     nsip

## Binary Size Analysis
File   .text   Size    Crate
 71.0%  59.0%   1.2MiB  std
  8.5%   7.1%   147KiB  nsip
  
## Documentation Coverage
Documenting nsip v0.1.0
warning: missing documentation for public function
```

## Configuration

### Unsafe Code Limits

In `Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "forbid"  # No unsafe allowed
# or
unsafe_code = "warn"    # Warn but allow
```

### Binary Size Optimization

```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization
strip = true          # Remove symbols
panic = "abort"       # Smaller panic handler
```

### Documentation Requirements

```toml
[lints.rust]
missing_docs = "warn"           # Warn on missing docs
rustdoc::broken_intra_doc_links = "deny"  # Fail on broken links
```

## Interpreting Results

### Unsafe Code Report

```
Functions  Expressions  Impls  Traits  Methods  Dependency
2/10       5/100        0/5    0/2     0/20     ✓ nsip
```

- **Functions**: 2 functions contain unsafe code
- **Expressions**: 5 unsafe expressions total
- **✓** = No unsafe in this crate's API

**Action:** Review unsafe usage, add safety comments.

### Binary Size Report

```
File   .text   Size    Crate
71.0%  59.0%   1.2MiB  std        ← Standard library
 8.5%   7.1%   147KiB  nsip
 5.2%   4.3%   89KiB   serde
```

**Action:** Consider `serde` feature flags to reduce size.

### Documentation Coverage

```
warning: missing documentation for public function `add`
  --> src/lib.rs:10
```

**Action:** Add doc comments:

```rust
/// Adds two numbers together.
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

## Improving Metrics

### Reduce Unsafe Code

```rust
// Before
unsafe {
    *ptr = value;
}

// After - use safe abstraction
vec[index] = value;
```

### Reduce Binary Size

```bash
# Analyze what's taking space
cargo bloat --release -n 20

# Enable size optimizations
[profile.release]
opt-level = "z"
strip = true
```

### Improve Documentation

```bash
# Check coverage
cargo doc --no-deps

# Run doc tests
cargo test --doc

# Generate private docs
cargo doc --no-deps --document-private-items
```

## Troubleshooting

### cargo-geiger Errors

```bash
# Update tool
cargo install cargo-geiger --force

# Clear cache
cargo clean
cargo geiger
```

### Binary Size Analysis Fails

```bash
# Ensure release build exists
cargo build --release

# Run bloat
cargo bloat --release
```

### Documentation Warnings

```bash
# See all warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## Links

- [cargo-geiger](https://github.com/rust-secure-code/cargo-geiger)
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat)
- [rustdoc Documentation](https://doc.rust-lang.org/rustdoc/)
- [Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/)
