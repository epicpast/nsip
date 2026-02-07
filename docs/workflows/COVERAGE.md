# Code Coverage Tracking

## Overview

Automated code coverage measurement and tracking using [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov).

**Workflow:** `.github/workflows/coverage.yml`
**Tool:** cargo-llvm-cov
**Integration:** Codecov (optional)
**Triggers:** Push to main, PRs, Weekly schedule
**Target:** ≥80% coverage

## How It Works

1. **Instrument**: Compile with coverage instrumentation
2. **Execute**: Run all tests (unit, integration, doc)
3. **Collect**: Gather coverage data
4. **Report**: Generate HTML, LCOV, JSON reports
5. **Upload**: Send to Codecov (optional)
6. **Comment**: Post summary on PRs

## Setup

### Install Locally

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Install llvm-tools component
rustup component add llvm-tools-preview
```

### Generate Coverage

```bash
# Generate coverage for all tests
cargo llvm-cov

# Generate HTML report
cargo llvm-cov --html --open

# Generate LCOV format (for Codecov)
cargo llvm-cov --lcov --output-path lcov.info

# Generate JSON report
cargo llvm-cov --json --output-path coverage.json
```

## Understanding Reports

### Summary Output

```
Filename              Regions  Missed Regions  Coverage
---------------------------------------------------------
src/lib.rs                 45              3     93.33%
src/parser.rs              78             12     84.62%
src/utils.rs               23              0    100.00%
---------------------------------------------------------
TOTAL                     146             15     89.73%
```

**Metrics:**
- **Regions**: Code regions (branches, statements)
- **Missed Regions**: Not executed during tests
- **Coverage**: % of regions executed

### HTML Report

Interactive report showing:
- **Green**: Covered lines
- **Red**: Uncovered lines
- **Yellow**: Partially covered branches

**Access:** `target/llvm-cov/html/index.html`

### Coverage Types

1. **Line Coverage**: % of lines executed
2. **Branch Coverage**: % of conditional branches taken
3. **Function Coverage**: % of functions called

## Coverage Goals

| Coverage | Quality | Action |
|----------|---------|--------|
| < 50% | Poor ❌ | Critical gaps |
| 50-70% | Fair ⚠️ | Needs improvement |
| 70-85% | Good ✅ | Acceptable |
| > 85% | Excellent 🌟 | High quality |

**Project Target: ≥80%**

## Improving Coverage

### Identify Gaps

```bash
# Show uncovered lines
cargo llvm-cov --show-missing-lines

# Focus on specific file
cargo llvm-cov --ignore-filename-regex tests/
```

### Example: Uncovered Code

**Code:**
```rust
pub fn divide(a: i32, b: i32) -> Result<i32, Error> {
    if b == 0 {
        return Err(Error::DivideByZero);  // ❌ Not covered
    }
    Ok(a / b)  // ✅ Covered
}
```

**Test (Incomplete):**
```rust
#[test]
fn test_divide() {
    assert_eq!(divide(10, 2).unwrap(), 5);
    // Missing: error case!
}
```

**Fix: Add Error Test**
```rust
#[test]
fn test_divide_by_zero() {
    assert!(divide(10, 0).is_err());
}
```

### Common Gaps

#### 1. Error Paths

```rust
// Add tests for all error branches
#[test]
fn test_errors() {
    assert!(parse("").is_err());
    assert!(parse("invalid").is_err());
    assert!(parse("too_long_".repeat(1000).as_str()).is_err());
}
```

#### 2. Edge Cases

```rust
// Test boundaries
#[test]
fn test_boundaries() {
    assert_eq!(clamp(0, 0, 10), 0);     // min
    assert_eq!(clamp(10, 0, 10), 10);   // max
    assert_eq!(clamp(-1, 0, 10), 0);    // below min
    assert_eq!(clamp(11, 0, 10), 10);   // above max
}
```

#### 3. Conditional Branches

```rust
pub fn process(data: &[u8], validate: bool) -> Result<Output> {
    if validate {
        check_validity(data)?;  // Branch 1
    }
    // Branch 2
    parse(data)
}

#[test]
fn test_both_branches() {
    // Test with validation
    assert!(process(b"valid", true).is_ok());
    // Test without validation
    assert!(process(b"data", false).is_ok());
}
```

#### 4. Match Arms

```rust
match status {
    Status::Ready => handle_ready(),      // Test
    Status::Pending => handle_pending(),  // Test
    Status::Error(_) => handle_error(),   // Test
}
```

## Configuration

### Exclude Files

```bash
# Ignore test files
cargo llvm-cov --ignore-filename-regex tests/

# Ignore generated code
cargo llvm-cov --ignore-filename-regex generated/
```

### Coverage Threshold

```bash
# Fail if below 80%
coverage=$(cargo llvm-cov --summary-only | grep -oP 'TOTAL.*\K\d+\.\d+')
if (( $(echo "$coverage < 80" | bc -l) )); then
    echo "Coverage ${coverage}% below threshold 80%"
    exit 1
fi
```

### Feature Flags

```bash
# Coverage with all features
cargo llvm-cov --all-features

# Coverage with specific features
cargo llvm-cov --features feature1,feature2
```

## Codecov Integration

### Setup

1. **Sign up**: https://codecov.io/
2. **Add repository**: GitHub integration
3. **Get token**: Settings → Repository Upload Token
4. **Add secret**: GitHub repo → Settings → Secrets → `CODECOV_TOKEN`

### Upload

```bash
# Generate LCOV
cargo llvm-cov --lcov --output-path lcov.info

# Upload to Codecov
bash <(curl -s https://codecov.io/bash) -f lcov.info
```

**CI Integration:** Automatic via workflow.

### Codecov Features

- **PR Comments**: Coverage diff on PRs
- **Trends**: Coverage over time
- **Badges**: Display in README
- **Sunburst**: Visual coverage map

### Badge

Add to `README.md`:

```markdown
[![codecov](https://codecov.io/gh/USER/REPO/branch/main/graph/badge.svg)](https://codecov.io/gh/USER/REPO)
```

## Advanced Usage

### Differential Coverage

```bash
# Coverage for changed files only (in PR)
git diff --name-only main | grep '\.rs$' | xargs cargo llvm-cov --include-ffi
```

### Profiling Hot Spots

```bash
# Generate profdata for analysis
cargo llvm-cov --no-report --profdata-output rust_template.profdata

# Analyze with llvm-profdata
llvm-profdata show rust_template.profdata
```

### Doc Test Coverage

```bash
# Include documentation tests
cargo llvm-cov --doc
```

### Workspace Coverage

```bash
# Coverage across workspace
cargo llvm-cov --workspace

# Exclude workspace members
cargo llvm-cov --workspace --exclude member1
```

## CI Workflow Details

### What It Does

1. Runs all tests with instrumentation
2. Generates HTML, LCOV, JSON reports
3. Uploads to Codecov (if token configured)
4. Comments PR with summary
5. Uploads artifacts (30-day retention)

### Access Reports

**In CI:**
- Actions → Workflow Run → Artifacts → coverage-report

**In PR:**
- Automated comment with coverage %

## Troubleshooting

### Zero Coverage

```bash
# Ensure llvm-tools installed
rustup component add llvm-tools-preview

# Clean and rebuild
cargo clean
cargo llvm-cov
```

### Incomplete Coverage

```bash
# Include all test types
cargo llvm-cov --all-targets

# Include doc tests
cargo llvm-cov --doc
```

### Slow Coverage

```bash
# Parallel execution
cargo llvm-cov -- --test-threads=4

# Skip slow tests
cargo llvm-cov -- --skip slow_test
```

### Codecov Upload Fails

```bash
# Check token
echo $CODECOV_TOKEN

# Manual upload with verbose
bash <(curl -s https://codecov.io/bash) -f lcov.info -v
```

## Best Practices

1. **Aim for ≥80%** - Good balance of quality and effort
2. **Test error paths** - Don't just test happy path
3. **Exclude test code** - Focus on production code
4. **Use integration tests** - Cover real usage patterns
5. **Track trends** - Coverage should improve over time
6. **Don't game metrics** - Meaningful tests > coverage %

## What Coverage Doesn't Measure

Coverage shows **execution**, not:
- **Correctness**: Executed code may still be wrong
- **Edge cases**: May miss unusual inputs
- **Logic errors**: All branches covered ≠ all cases tested
- **Race conditions**: Concurrency issues invisible

**Use with:** Mutation testing, property-based testing, manual review.

## Ignoring Code

```rust
// Ignore unreachable safety invariants
#[cfg(not(tarpaulin_include))]
fn internal_safety_check() {
    unreachable!("Safety invariant violated");
}

// Ignore debug-only code
#[cfg(debug_assertions)]
fn debug_only_function() {
    // Not covered in release builds
}
```

## Links

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov Documentation](https://docs.codecov.com/)
- [LLVM Coverage Mapping](https://llvm.org/docs/CoverageMappingFormat.html)
- [Coverage Best Practices](https://testing.googleblog.com/2020/08/code-coverage-best-practices.html)
