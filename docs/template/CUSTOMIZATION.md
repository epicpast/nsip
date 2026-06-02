---
diataxis_type: how-to
---
# Customization Guide

This guide covers how to customize the `nsip` beyond the initial setup. For basic configuration (renaming the crate, updating metadata), see the main README.

---

## Table of Contents

1. [Adding New Modules](#1-adding-new-modules)
2. [Removing Example Code](#2-removing-example-code)
3. [Library-Only vs Binary Crate](#3-library-only-vs-binary-crate)
4. [Adjusting Lint Strictness](#4-adjusting-lint-strictness)
5. [Adjusting Supply Chain Policy](#5-adjusting-supply-chain-policy)
6. [Modifying Release Targets](#6-modifying-release-targets)
7. [Docker Customization](#7-docker-customization)
8. [Adding Property-Based Tests](#8-adding-property-based-tests)

---

## 1. Adding New Modules

### Create the module file

Create a new file under `crates/`. For example, `crates/parser.rs`:

```rust
//! Parser module for nsip.

use crate::{Error, Result};

/// Parses the given input string into a structured value.
///
/// # Arguments
///
/// * `input` - The raw input to parse.
///
/// # Returns
///
/// The parsed output.
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if the input is malformed.
///
/// # Examples
///
/// ```rust
/// use nsip::parser::parse;
///
/// let result = parse("valid input")?;
/// assert!(!result.is_empty());
/// # Ok::<(), nsip::Error>(())
/// ```
pub fn parse(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err(Error::InvalidInput("input cannot be empty".to_string()));
    }
    Ok(input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let result = parse("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_parse_empty() {
        let result = parse("");
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }
}
```

### Register the module in `crates/lib.rs`

Add a `pub mod` declaration at the top of `crates/lib.rs`:

```rust
pub mod parser;
```

### Add integration tests

Create or extend a file in `tests/`. For example, `tests/parser_test.rs`:

```rust
use nsip::parser::parse;

#[test]
fn test_parse_integration() {
    let result = parse("test input");
    assert!(result.is_ok());
}
```

### Checklist

- [ ] Module file created in `crates/`
- [ ] `pub mod` added to `crates/lib.rs`
- [ ] All public items have doc comments with `# Examples`
- [ ] Unit tests in `#[cfg(test)]` module within the file
- [ ] Integration tests added to `tests/`
- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets --all-features` passes
- [ ] `cargo doc --no-deps` builds without warnings

---

## 2. Removing Example Code

The template ships with placeholder functions (`add`, `divide`) and a `Config` builder to demonstrate patterns. Once you are ready to add your own code, remove them.

### Step 1: Clean up `crates/lib.rs`

Remove the `add` and `divide` functions, the `Config` struct and its `impl` blocks, and the entire `#[cfg(test)] mod tests` block. Keep the `Error` enum and the `Result` type alias -- you will likely need them.

Your `crates/lib.rs` should look like this after cleanup:

```rust
#![doc = include_str!("../README.md")]

use thiserror::Error;

/// Error type for `nsip` operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid input was provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// An operation failed.
    #[error("operation '{operation}' failed: {cause}")]
    OperationFailed {
        /// The operation that failed.
        operation: String,
        /// The underlying cause.
        cause: String,
    },
}

/// Result type alias for `nsip` operations.
pub type Result<T> = std::result::Result<T, Error>;

// Add your modules and public API here.
```

### Step 2: Update or remove `crates/main.rs`

If you keep a binary target, rewrite `crates/main.rs` to remove references to the example functions:

```rust
//! Binary entry point for `nsip`.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::process::ExitCode;

fn run() -> Result<(), nsip::Error> {
    // Your application logic here
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        },
    }
}
```

If you do not need a binary, see [Library-Only vs Binary Crate](#3-library-only-vs-binary-crate) below.

### Step 3: Clean up integration tests

Remove or rewrite `tests/integration_test.rs`. Delete the example tests that reference `add`, `divide`, and `Config`, and replace them with tests for your own API.

### Step 4: Update README.md

Remove the example usage snippets in `README.md` that reference `add`, `divide`, and `Config`. Replace them with documentation for your own public API.

---

## 3. Library-Only vs Binary Crate

### Removing the binary target

If your crate is library-only, remove the binary configuration:

1. Delete `crates/main.rs`.

2. Remove the `[[bin]]` section from `Cargo.toml`:

   ```toml
   # Delete these lines:
   [[bin]]
   name = "nsip"
   path = "crates/main.rs"
   ```

3. Remove the binary-related steps from the Docker and release workflows if you do not need them.

### Adding additional binaries

Add extra `[[bin]]` sections to `Cargo.toml`:

```toml
[[bin]]
name = "nsip"
path = "crates/main.rs"

[[bin]]
name = "nsip_cli"
path = "crates/cli.rs"
```

Each binary gets its own entry point file under `crates/`. Remember to add `#![allow(clippy::print_stdout, clippy::print_stderr)]` at the top of each binary file since the lint configuration forbids print macros in library code.

### Using a workspace for multiple crates

For larger projects, convert to a Cargo workspace. Replace the top-level `Cargo.toml` with a workspace manifest:

```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/cli",
    "crates/server",
]

[workspace.package]
edition = "2024"
rust-version = "1.92"
license = "MIT"
repository = "https://github.com/zircote/nsip"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

Each member crate then has its own `Cargo.toml` that inherits workspace settings:

```toml
[package]
name = "nsip_core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lints]
workspace = true
```

---

## 4. Adjusting Lint Strictness

### Lint groups in `Cargo.toml`

The template enables four Clippy lint groups in `[lints.clippy]`:

| Group | What it covers |
|-------|---------------|
| `all` | Standard Clippy lints (correctness, style, complexity, performance) |
| `pedantic` | Stricter, opinionated lints (naming conventions, API design, documentation) |
| `nursery` | Experimental lints that may have false positives |
| `cargo` | Cargo manifest issues (missing fields, feature misuse) |

To relax the overall strictness, remove a group or change its level:

```toml
[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
# nursery = { level = "warn", priority = -1 }   # Commented out to disable
cargo = { level = "warn", priority = -1 }
```

### Allowing specific lints per item

Use `#[allow()]` attributes to suppress a lint for a specific function, struct, or module:

```rust
#[allow(clippy::cast_possible_truncation)]
fn to_u32(value: u64) -> u32 {
    value as u32
}
```

For an entire module:

```rust
#[allow(clippy::too_many_lines)]
mod complex_module {
    // ...
}
```

### Denied lints

The template denies these lints at the crate level:

```toml
unwrap_used = "deny"     # Use Result and ? instead
expect_used = "deny"     # Use Result and ? instead
panic = "deny"           # Never panic in library code
todo = "deny"            # Complete all implementations
unimplemented = "deny"   # Complete all implementations
dbg_macro = "deny"       # Remove debug macros before committing
print_stdout = "deny"    # Use tracing or log instead
print_stderr = "deny"    # Use tracing or log instead
```

To relax any of these, change `"deny"` to `"warn"` or `"allow"`:

```toml
todo = "warn"            # Allow TODOs with a warning
```

Note that `print_stdout` and `print_stderr` are already allowed in `crates/main.rs` via file-level attributes. This is intentional -- binary entry points typically need to print output.

### `clippy.toml` thresholds

The `clippy.toml` file controls numeric thresholds for various lints:

| Setting | Default | Purpose |
|---------|---------|---------|
| `cognitive-complexity-threshold` | 25 | Maximum cognitive complexity per function |
| `excessive-nesting-threshold` | 4 | Maximum nesting depth |
| `too-many-lines-threshold` | 100 | Maximum function length |
| `too-many-arguments-threshold` | 7 | Maximum function parameters |
| `max-struct-bools` | 3 | Maximum bool fields in a struct |
| `max-fn-params-bools` | 3 | Maximum bool parameters in a function |
| `pass-by-value-size-limit` | 256 | Byte threshold to warn about passing large types by value |
| `type-complexity-threshold` | 250 | Threshold for overly complex types |

Adjust these to match your project's needs. For example, to allow longer functions:

```toml
too-many-lines-threshold = 200
```

Test code is exempt from several strict lints via these settings in `clippy.toml`:

```toml
allow-dbg-in-tests = true
allow-expect-in-tests = true
allow-unwrap-in-tests = true
allow-print-in-tests = true
```

### `rustfmt.toml` options

Key formatting settings you may want to adjust:

| Setting | Default | Purpose |
|---------|---------|---------|
| `max_width` | 100 | Maximum line width |
| `tab_spaces` | 4 | Spaces per indentation level |
| `imports_granularity` | `"Crate"` | How imports are grouped (`Crate`, `Module`, `Item`, `One`) |
| `group_imports` | `"StdExternalCrate"` | Import grouping order (std, external, crate) |
| `wrap_comments` | true | Wrap long comments to fit `comment_width` |
| `trailing_comma` | `"Vertical"` | Add trailing commas in multi-line constructs |
| `edition` | `"2024"` | Rust edition for parsing |

To change line width to 120 characters:

```toml
max_width = 120
comment_width = 120
```

---

## 5. Adjusting Supply Chain Policy

The `deny.toml` file configures `cargo-deny` to audit your dependency tree.

### Adding allowed licenses

The `[licenses]` section lists allowed SPDX license identifiers. To add a new license:

```toml
[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "MPL-2.0",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "CC0-1.0",
    "BSL-1.0",
    "0BSD",
    "LGPL-3.0",        # <-- Added
]
```

For crates with non-standard license files, add a `[[licenses.clarify]]` entry:

```toml
[[licenses.clarify]]
name = "some_crate"
expression = "MIT AND BSD-2-Clause"
license-files = [{ path = "LICENSE", hash = 0x12345678 }]
```

### Exempting specific advisories

If a security advisory does not apply to your usage, add its ID to the `ignore` list:

```toml
[advisories]
ignore = [
    "RUSTSEC-2024-XXXX",   # Reason: only affects feature X which we don't use
]
```

Always include a comment explaining why the advisory is exempted.

### Banning specific crates

The `[bans]` section prevents specific crates from entering your dependency tree:

```toml
[bans]
deny = [
    { name = "openssl", wrappers = [], reason = "Use rustls for TLS instead" },
    { name = "atty", wrappers = [], reason = "Use std::io::IsTerminal instead (available in Rust 1.70+)" },
    { name = "some_crate", wrappers = [], reason = "Known to be unmaintained" },
]
```

The `wrappers` field allows exceptions when a banned crate is only used as a transitive dependency of a specific wrapper crate:

```toml
{ name = "openssl", wrappers = ["openssl-sys"], reason = "Only allow via openssl-sys" }
```

### Configuring source restrictions

By default, only crates.io is allowed as a registry source:

```toml
[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

To allow a private registry or specific git repositories:

```toml
allow-registry = [
    "https://github.com/rust-lang/crates.io-index",
    "https://my-company.example.com/cargo/index",
]

allow-git = [
    "https://github.com/zircote/private-crate",
]
```

### Verifying changes

After modifying `deny.toml`, run the checks:

```bash
cargo deny check
```

---

## 6. Modifying Release Targets

The release workflow (`.github/workflows/release.yml`) builds binaries for multiple platforms using a matrix strategy.

### Default targets

| Target | OS Runner | Architecture |
|--------|-----------|-------------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Linux x86_64 |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | Linux ARM64 (cross-compiled) |
| `x86_64-apple-darwin` | `macos-latest` | macOS x86_64 |
| `aarch64-apple-darwin` | `macos-latest` | macOS ARM64 (Apple Silicon) |
| `x86_64-pc-windows-msvc` | `windows-latest` | Windows x86_64 |

### Adding a target

Add a new entry to the `matrix.include` array in `release.yml`. Each entry sets
`artifact_name` (the bare built binary, `nsip` or `nsip.exe`) and `platform`
(the platform suffix). The workflow computes the versioned release-asset name as
`nsip-${VERSION}-${platform}`, so do not hardcode a version into the matrix:

```yaml
- os: ubuntu-latest
  target: x86_64-unknown-linux-musl
  artifact_name: nsip
  platform: linux-musl-amd64
```

For musl targets, you will also need to install the musl toolchain:

```yaml
- name: Install musl tools
  if: matrix.target == 'x86_64-unknown-linux-musl'
  run: |
    sudo apt-get update
    sudo apt-get install -y musl-tools
```

### Removing a target

Delete the corresponding entry from `matrix.include`. For example, to drop Windows support, remove:

```yaml
# Delete this block:
- os: windows-latest
  target: x86_64-pc-windows-msvc
  artifact_name: nsip.exe
  platform: windows-amd64.exe
```

Also remove the target from `deny.toml`'s `[graph].targets` list so supply chain checks stay aligned.

### Cross-compilation requirements

Cross-compilation for `aarch64-unknown-linux-gnu` requires:

- `gcc-aarch64-linux-gnu` installed on the runner
- The `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` environment variable set to `aarch64-linux-gnu-gcc`

Both are already configured in the workflow. For other cross-compilation targets, you will need to install the appropriate cross-compilation toolchain and set the corresponding linker environment variable.

---

## 7. Docker Customization

The `Dockerfile` uses a multi-stage build:

1. **Builder stage** (`rust:1.92-slim`) -- compiles the binary with release optimizations.
2. **Runtime stage** (`gcr.io/distroless/cc-debian12`) -- minimal image containing only the binary.

### Changing the base image

To use a different runtime base image (for example, Debian slim instead of distroless):

```dockerfile
# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN useradd --create-home appuser
USER appuser

COPY --from=builder /app/target/release/nsip /usr/local/bin/nsip

ENTRYPOINT ["/usr/local/bin/nsip"]
```

Distroless is preferred for production because it contains no shell or package manager, reducing the attack surface. Use Debian slim if you need to debug inside the container or require additional runtime dependencies.

### Adding runtime dependencies

If your binary links against shared libraries at runtime, install them in the runtime stage. With distroless, you are limited to what is available in the base image. For additional libraries, switch to a Debian-based runtime:

```dockerfile
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*
```

### Modifying the builder stage

The builder stage uses a dependency caching strategy: it first copies `Cargo.toml` and `Cargo.lock`, builds with dummy source files to cache dependencies, then copies the real source and rebuilds. This means dependency downloads are cached across builds as long as `Cargo.toml`/`Cargo.lock` do not change.

If you add new source directories (for example, a workspace with multiple crates), update the dummy source creation:

```dockerfile
# Create dummy source to cache dependencies
RUN mkdir -p crates/core/src crates/cli/src && \
    echo "pub fn dummy() {}" > crates/core/src/lib.rs && \
    echo "fn main() {}" > crates/cli/src/main.rs
```

And update the copy step:

```dockerfile
COPY crates/ ./crates/
```

### Adding build-time dependencies

If your project needs additional system libraries at compile time, add them in the builder stage:

```dockerfile
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
```

---

## 8. Adding Property-Based Tests

The template already includes `proptest` as a dev-dependency and ships with example property tests in `tests/integration_test.rs`.

### Where property tests live

Property tests can be placed in:

- **Unit tests**: Inside `crates/*.rs` in a `#[cfg(test)]` module
- **Integration tests**: Inside `tests/` files, typically in a submodule

The template demonstrates the integration test approach:

```rust
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn add_is_commutative(a in any::<i32>(), b in any::<i32>()) {
            let a = i64::from(a);
            let b = i64::from(b);
            prop_assert_eq!(add(a, b), add(b, a));
        }

        #[test]
        fn add_zero_is_identity(n in any::<i64>()) {
            prop_assert_eq!(add(n, 0), n);
            prop_assert_eq!(add(0, n), n);
        }
    }
}
```

### Writing effective property tests

Focus on **invariants** -- properties that must always hold regardless of input:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_serialize_deserialize(input in "\\PC{1,100}") {
        let serialized = serialize(&input)?;
        let deserialized = deserialize(&serialized)?;
        prop_assert_eq!(input, deserialized);
    }

    #[test]
    fn output_is_always_valid(input in any::<u64>()) {
        let result = transform(input);
        prop_assert!(is_valid(&result));
    }
}
```

### Custom strategies

For domain-specific types, define custom generators:

```rust
use proptest::prelude::*;

fn valid_config() -> impl Strategy<Value = Config> {
    (any::<bool>(), 1..100u32, 1..3600u64).prop_map(|(verbose, retries, timeout)| {
        Config::new()
            .with_verbose(verbose)
            .with_max_retries(retries)
            .with_timeout(timeout)
    })
}

proptest! {
    #[test]
    fn config_always_has_positive_timeout(config in valid_config()) {
        prop_assert!(config.timeout_secs > 0);
    }
}
```

### Configuration

Proptest behavior can be tuned via a `proptest.toml` file or the `ProptestConfig` struct. Create `proptest.toml` in the project root to adjust the number of test cases:

```toml
# Number of successful test cases required (default: 256)
cases = 512

# Maximum number of shrink iterations (default: 4096)
max_shrink_iters = 8192
```

For detailed guidance on property-based testing strategies, see [docs/testing/PROPERTY-BASED-TESTING.md](../testing/PROPERTY-BASED-TESTING.md).
