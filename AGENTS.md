# AGENTS.md

Instructions for AI coding agents working on this Rust project.

## Project Context

- **Language**: Rust (edition 2024, MSRV 1.92)
- **Build System**: Cargo
- **Linting**: clippy with pedantic and nursery lints
- **Formatting**: rustfmt (100-char lines, 4-space indent)
- **Error Handling**: `thiserror` for custom error types
- **Testing**: Built-in test framework + proptest for property-based testing
- **Supply Chain**: cargo-deny for dependency auditing

## File Structure

```
crates/
  lib.rs           # Library entry point and public API
  main.rs          # Binary entry point (optional)
tests/             # Integration tests
benches/           # Benchmarks (criterion)
examples/          # Example programs
```

## Build and Test Commands

```bash
cargo build                                    # Build
cargo test --all-features                      # Run all tests
cargo clippy --all-targets --all-features -- -D warnings  # Lint
cargo fmt -- --check                           # Check formatting
cargo doc --no-deps                            # Build docs
cargo deny check                               # Supply chain audit
```

## Code Rules

### Never Panic in Library Code

Do not use `unwrap()`, `expect()`, or `panic!()`. Always return `Result`:

```rust
pub fn parse(input: &str) -> Result<Value, Error> {
    input.parse().map_err(Error::Parse)
}
```

### Use thiserror for Errors

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

### Document All Public Items

Include `# Examples` and `# Errors` sections:

```rust
/// Processes the input data.
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if the input is empty.
///
/// # Examples
///
/// ```rust
/// use rust_template::process;
/// let result = process("data")?;
/// # Ok::<(), rust_template::Error>(())
/// ```
pub fn process(input: &str) -> Result<Output, Error> {
    // implementation
}
```

### Prefer Borrowing Over Ownership

```rust
// Preferred
pub fn process(data: &[u8]) -> Result<Vec<u8>, Error> { ... }

// Avoid
pub fn process(data: Vec<u8>) -> Result<Vec<u8>, Error> { ... }
```

### Use const fn Where Possible

```rust
#[must_use]
pub const fn new() -> Self {
    Self { value: 0 }
}
```

## Testing Patterns

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success() {
        let result = function(valid_input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error() {
        let result = function(invalid_input);
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip(input in any::<i64>()) {
        let encoded = encode(input);
        prop_assert_eq!(decode(&encoded)?, input);
    }
}
```

## Forbidden Patterns

- `unsafe` blocks (unless explicitly justified)
- `unwrap()`, `expect()`, `panic!()` in library code
- `todo!()`, `unimplemented!()`
- `dbg!()`, `print!()`, `println!()`, `eprint!()`, `eprintln!()`
