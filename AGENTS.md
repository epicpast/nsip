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
  main.rs          # Binary entry point
  client.rs        # HTTP client for the NSIP Search API
  models.rs        # Data models (SearchCriteria, AnimalDetails, etc.)
  format.rs        # Human-readable ASCII table formatting
  mcp/             # MCP server (13 tools, prompts, resources)
tests/             # Integration tests
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

## Branching & Release Workflow

- **`develop`** is the default branch and where all development happens. Branch from `develop` and open PRs **into `develop`** — CI gates every PR here.
- **`main`** is the stable/release branch. Never commit or open feature PRs directly against `main`.
- **Releasing**: open a release PR `develop → main` (the `release-pr.yml` workflow can open/update it), merge it, then tag the `main` merge commit `vX.Y.Z` and push the tag. The tag triggers all release automation (`release` — attested + fail-closed verified, `publish` — crates.io Trusted Publishing, `docker`, `back-merge`; the published release triggers `package-homebrew`).
- **Hotfixes**: branch from `main`, PR into `main`, tag, then merge `main` back into `develop`.

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
/// ```rust,no_run
/// use nsip::NsipClient;
/// # async fn example() -> Result<(), nsip::Error> {
/// let client = NsipClient::new();
/// let groups = client.breed_groups().await?;
/// # Ok(())
/// # }
/// ```
pub async fn breed_groups(&self) -> Result<Vec<BreedGroup>, Error> {
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

## NSIP MCP Server

The binary ships a built-in MCP server (`nsip mcp`) for sheep genetic evaluation with 13 tools. See [`docs/MCP.md`](docs/MCP.md) for the full API reference and [`docs/llm-guides/`](docs/llm-guides/) for ready-to-use LLM instruction templates.
