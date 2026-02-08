# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **GitHub template repository** for Rust crates. The crate name is `nsip` (Rust edition 2024, MSRV 1.92). It ships both a library (`crates/lib.rs`) and a binary (`crates/main.rs`). Source lives in `crates/`, not the standard `src/` directory.

## Build Commands

[`just`](https://github.com/casey/just) is the local task runner. Run `just` to list all recipes.

```bash
just                  # List all recipes
just check            # Full CI check (fmt + clippy + test + doc + deny)
just test             # Run all tests
just lint             # Clippy with CI flags
just fmt              # Format code
just deny             # Supply chain audit
just coverage         # LCOV coverage report
just msrv             # Check against MSRV 1.92
just miri             # Miri undefined behavior detection
```

<details>
<summary>Raw cargo commands</summary>

```bash
cargo build                                              # Build
cargo test --all-features                                # Run all tests
cargo test test_name                                     # Run specific test
cargo test -- --nocapture                                # Run tests with stdout
cargo clippy --all-targets --all-features -- -D warnings # Lint (CI uses -D warnings)
cargo fmt                                                # Format
cargo fmt -- --check                                     # Check formatting
cargo deny check                                         # Supply chain audit
cargo doc --no-deps --all-features                       # Build docs
cargo +nightly miri test                                 # UB detection

# Full CI check (run before pushing)
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test && cargo doc --no-deps && cargo deny check
```

</details>

## Architecture

### Source Layout

- `crates/lib.rs` — Library root: exports `Error` (thiserror), `Result<T>`, `Config` (builder pattern), `add()`, `divide()`
- `crates/main.rs` — Binary entry point with `run() -> Result` pattern returning `ExitCode`
- `tests/integration_test.rs` — Integration tests including property-based tests (proptest)

### Key Patterns

- **Error handling**: `thiserror` for error types, `Result<T>` alias, `?` propagation. Never `unwrap`/`expect`/`panic!` in library code.
- **Builder pattern**: `Config::new().with_verbose(true).with_max_retries(5)` using `const fn` and `#[must_use]`
- **Binary structure**: `main()` returns `ExitCode`, delegates to `run()` which returns `Result`. Binary code allows `#[allow(clippy::print_stdout, clippy::print_stderr)]`.

### Lint Configuration

Clippy runs with **pedantic + nursery + cargo** lints. Key denied lints: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr`. Tests are exempt (`allow-unwrap-in-tests = true`, etc. in `clippy.toml`).

Thresholds from `clippy.toml`: max 100 lines/function, max 7 params, cognitive complexity 25, nesting depth 4.

### Formatting

Configured in `rustfmt.toml`: 100-char line width, edition 2024, `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`, `trailing_comma = "Vertical"`, `brace_style = "SameLineWhere"`. Uses `version = "Two"` (nightly features).

### Supply Chain Security

`deny.toml` enforces: only permissive licenses (MIT, Apache-2.0, BSD, etc.), crates.io-only sources, bans `openssl` (use rustls) and `atty` (use `std::io::IsTerminal`).

## Testing

- **Unit tests**: `#[cfg(test)] mod tests` inside source files
- **Integration tests**: `tests/integration_test.rs`
- **Property tests**: `proptest` crate in `tests/integration_test.rs::property_tests` module
- **Parameterized tests**: `test-case` crate available in dev-dependencies
- **Doc tests**: all public API examples must compile

## CI/CD

The CI pipeline (`.github/workflows/ci.yml`) runs: fmt, clippy, test (Linux/macOS/Windows matrix), doc build, cargo-deny, MSRV check (1.92), and coverage (cargo-llvm-cov to Codecov).

Release workflow triggers on version tags (`v*`): builds multi-platform binaries (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64), generates changelog via git-cliff, publishes to crates.io, builds Docker images (distroless base) to ghcr.io.

## Code Style Rules

- Prefer `&str` over `String` and `&[T]` over `Vec<T>` in function parameters
- Use `Cow<'_, str>` for flexible string returns
- Use `const fn` where possible, `#[must_use]` on value-returning functions
- All public items require doc comments with `# Arguments`, `# Returns`, `# Errors`, `# Examples`
- Group imports: std, external crates, crate-local
- `unsafe` code is forbidden (`unsafe_code = "forbid"` in Cargo.toml)

## NSIP MCP Server

The binary ships a built-in MCP server (`nsip mcp`) for sheep genetic evaluation with 13 tools, 7 guided prompts, and resource templates. See [`docs/MCP.md`](docs/MCP.md) for the full API reference and [`docs/llm-guides/`](docs/llm-guides/) for ready-to-use LLM instruction templates.
