---
applyTo: "tests/**/*.rs"
---

# Test Instructions

When generating or modifying test files in `tests/`:

## Test Structure

- Use descriptive test names: `test_<function>_<scenario>_<expected>`
- Group related tests in modules
- Use `assert_eq!` for equality, `assert!(matches!(...))` for patterns
- Test both success and error paths

## Property-Based Testing

Use `proptest` for property-based tests:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_name(input in strategy()) {
        prop_assert!(condition(input));
    }
}
```

## Test Helpers

- Place shared test utilities in `tests/common/mod.rs`
- Use `#[cfg(test)]` for unit test modules inside source files
- Use `test-case` crate for parameterized tests when available

## Assertions

- Prefer `assert_eq!(actual, expected)` over `assert!(actual == expected)`
- Use `assert!(matches!(result, Err(Error::Variant(_))))` for error matching
- Include descriptive messages: `assert_eq!(a, b, "values should match after transform")`

## No Panics in Test Setup

- Test assertions may panic (that is their purpose)
- Test setup and teardown should use `Result` where possible
- Use `#[test] fn test_name() -> Result<(), Error>` for fallible tests
