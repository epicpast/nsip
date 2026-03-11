---
diataxis_type: reference
---
# Mutation Testing with cargo-mutants

## Overview

Automated mutation testing to validate test suite effectiveness using [cargo-mutants](https://github.com/sourcefrog/cargo-mutants).

**Workflow:** `.github/workflows/mutation-testing.yml`
**Tool:** `cargo-mutants`
**Triggers:** PR (on src/tests changes), Manual dispatch
**Goal:** Detect weak or missing tests

## How It Works

Mutation testing modifies your code (introduces "mutants") and runs tests to see if they catch the changes:

1. **Generate Mutants**: Modify code in systematic ways (e.g., change `+` to `-`, `>` to `<`)
2. **Run Tests**: Execute test suite against each mutant
3. **Score**: % of mutants caught by tests = test quality score

**Good tests catch mutants. Missed mutants = test gaps.**

## Mutation Types

cargo-mutants generates these mutations:

- **Binary operators**: `+` → `-`, `*` → `/`, `&&` → `||`
- **Comparison operators**: `>` → `<`, `==` → `!=`
- **Return values**: Return default value instead of computed value
- **Function bodies**: Replace with default/empty implementation

## Usage

### Local Testing

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation tests
cargo mutants

# Test specific file
cargo mutants --file src/lib.rs

# Limit execution time
cargo mutants --timeout 300

# Generate JSON output
cargo mutants --output mutants.out --json
```

### CI Integration

The workflow runs automatically on PRs when `src/` or `tests/` change. Results posted as PR comment.

**View Results:**
- PR comments (automatic)
- Actions → Artifacts → mutation-test-report

## Understanding Results

### Summary Output

```
Total mutants: 50
Caught: 45
Missed: 5
Timeout: 0
Score: 90%
```

- **Total**: Number of mutations generated
- **Caught**: Mutations detected by tests (good!)
- **Missed**: Mutations not caught (test gaps)
- **Timeout**: Mutations causing infinite loops
- **Score**: `(caught / total) * 100`

**Target: ≥80% mutation score**

### Missed Mutants Report

```
Function: calculate_total
File: src/lib.rs:42
Mutation: Changed + to -
Status: MISSED

This mutant survived testing, indicating missing test coverage.
```

**Action:** Add test case that would fail if `+` became `-`.

## Improving Mutation Score

### Example: Missed Mutant

**Original Code:**
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Test (Inadequate):**
```rust
#[test]
fn test_add() {
    assert_eq!(add(2, 2), 4);
}
```

**Mutant:** `a + b` → `a - b`
**Result:** Test passes with `2 - 2 = 0` (wrong!)

**Fix: Add More Test Cases**
```rust
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);  // Would fail if + became -
    assert_eq!(add(0, 5), 5);
    assert_eq!(add(-1, 1), 0);
}
```

### Common Patterns

#### 1. Test Edge Cases

```rust
// Catch comparison mutations
#[test]
fn test_bounds() {
    assert!(is_valid(0));    // boundary
    assert!(is_valid(100));  // boundary
    assert!(!is_valid(101)); // just outside
}
```

#### 2. Test Error Paths

```rust
// Ensure error conditions are tested
#[test]
fn test_error() {
    assert!(parse("").is_err());
    assert!(parse("invalid").is_err());
}
```

#### 3. Test Return Values

```rust
// Don't just check success, verify values
#[test]
fn test_compute() {
    assert_eq!(compute(5), 25);  // Not just assert!(compute(5) > 0)
}
```

## Configuration

### Exclude Files

Create `.cargo-mutants.toml`:

```toml
[mutants]
exclude_files = [
    "src/generated.rs",
    "tests/fixtures/*.rs"
]
```

### Timeouts

```yaml
# In workflow
cargo mutants --timeout 300  # 5 minutes per mutant
```

### Target Specific Functions

```bash
# Test only changed functions
cargo mutants --file src/lib.rs --re "fn calculate"
```

## Interpreting Low Scores

**Score < 50%**: Critical test gaps
**Score 50-80%**: Needs improvement
**Score ≥80%**: Good coverage
**Score ≥95%**: Excellent coverage

### Why Mutants Survive

1. **Missing tests**: Function not tested at all
2. **Weak assertions**: Tests don't verify actual behavior
3. **Dead code**: Code that never executes (remove it)
4. **Equivalent mutants**: Mutation doesn't change behavior (rare)

## Troubleshooting

### Too Slow

```bash
# Run in parallel
cargo mutants --jobs 4

# Test changed files only
cargo mutants --file src/changed_file.rs
```

### Timeouts

```bash
# Increase timeout for slow tests
cargo mutants --timeout 600
```

### False Positives

Some mutants are equivalent (don't change behavior):

```rust
// These are equivalent
fn example() -> bool { true }
fn example() -> bool { return true; }
```

**Action:** Accept these or skip with `#[mutants::skip]`.

## Best Practices

1. **Run locally** before pushing to catch issues early
2. **Focus on critical paths** first (public API, core logic)
3. **Don't chase 100%** - diminishing returns above 90%
4. **Use with coverage** - mutation testing complements code coverage
5. **Add tests incrementally** - fix one missed mutant at a time

## Skipping Mutations

```rust
#[mutants::skip]  // Skip entire function
pub fn generated_code() -> i32 {
    42
}
```

## Links

- [cargo-mutants Documentation](https://mutants.rs/)
- [Mutation Testing Overview](https://en.wikipedia.org/wiki/Mutation_testing)
- [Best Practices Guide](https://mutants.rs/guide.html)
