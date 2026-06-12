# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **NSIP sheep genetic evaluation CLI and library**. The crate name is `nsip` (Rust edition 2024, MSRV 1.92). It ships both a library (`crates/lib.rs`) and a binary (`crates/main.rs`). Source lives in `crates/`, not the standard `src/` directory.

## Build Commands

[`just`](https://github.com/casey/just) is the local task runner. Run `just` to list all recipes.

```bash
just                  # List all recipes
just check            # Full CI check (fmt + clippy + test + doc + deny + coverage)
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

- `crates/lib.rs` — Library root: exports `NsipClient`, `SearchCriteria`, `Error` (thiserror), `Result<T>`, `mcp` module, `models` module
- `crates/main.rs` — Binary entry point with `run() -> Result` pattern returning `ExitCode`
- `crates/client.rs` — HTTP client for the NSIP Search API (`NsipClient`)
- `crates/models.rs` — Data models (`AnimalDetails`, `Lineage`, `Progeny`, `SearchResults`, etc.)
- `crates/format.rs` — Human-readable ASCII table formatting (binary-only module)
- `crates/mcp/` — MCP server with 13 tools, guided prompts, and resource templates
- `tests/integration_test.rs` — Integration tests including property-based tests (proptest)
- `tests/cli_test.rs` — CLI integration tests

### Key Patterns

- **Error handling**: `thiserror` + `miette::Diagnostic` for error types, `Result<T>` alias, `?` propagation. Fallible-API variants carry `#[source]` (cause chain) and `retry_after`. Never `unwrap`/`expect`/`panic!` in library code.
- **Dual-consumer error envelope (RFC 9457)**: every `Error` maps to a `ProblemDetails` (`crates/problem.rs`) via `to_problem_details(command)`. The binary renders errors as a `miette` diagnostic on a TTY and `application/problem+json` on stderr for non-TTY / `--format json` / `-J` (`crates/render.rs`). MCP tool errors carry the same envelope in the JSON-RPC `data` field. Exit codes are `sysexits`-aligned (1 caller, 3 upstream-parse, 75 transient). `type` URIs are stable repo-docs URLs (no version path) resolving to `docs/reference/errors/`. Input failures are typed per-operation via `Error::Validation { kind: ValidationKind, .. }` (each kind → its own `type`/title/`suggested_fix`); MCP validation paths route through `crate::mcp::problem_error`, which envelopes them and picks the JSON-RPC code by class. See `docs/reference/ERROR-ENVELOPE.md` and ADR-0004/0005. Adding an error variant **or `ValidationKind`** requires a catalog page under `docs/reference/errors/`.
- **Builder pattern**: `SearchCriteria::new().with_breed_id(640).with_status("CURRENT")` using `const fn` and `#[must_use]`
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
- **CLI tests**: `tests/cli_test.rs`
- **Property tests**: `proptest` crate in `tests/integration_test.rs::property_tests` module
- **Parameterized tests**: `test-case` crate available in dev-dependencies
- **Doc tests**: all public API examples must compile

## CI/CD

The CI pipeline (`.github/workflows/ci.yml`) runs: fmt, clippy, test (Linux/macOS/Windows matrix), doc build, cargo-deny, MSRV check (1.92), and coverage (cargo-llvm-cov to Codecov).

Release workflow triggers on version tags (`v*`): builds multi-platform binaries (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64), generates changelog via git-cliff, publishes to crates.io, builds Docker images (distroless base) to ghcr.io.

## Branching & Release Workflow

- **`develop`** is the default branch and the home of all active development. Branch features from `develop` and open PRs **into `develop`**. CI gates every PR here.
- **`main`** is the stable/release branch. Never commit or open feature PRs directly against `main`.
- **Releasing** (follow exactly — do NOT hand-roll these steps):
  1. **Bump the version** in `Cargo.toml` on `develop` (sync `Cargo.lock`).
  2. **Open the release PR via the `Release PR` workflow** — `gh workflow run "Release PR" -f version=X.Y.Z` (or Actions → Release PR). Do **NOT** open it with a manual `gh pr create` — the workflow exists for this and produces the standardized PR; hand-rolling bypasses it and lets it bit-rot.
  3. **Get the release PR reviewed**, then merge it into `main`.
  4. **Tag the `main` merge commit** `vX.Y.Z` and push the tag. The tag — not a branch push — triggers `release.yml` (fail-closed: binaries/extras/MCPB are built, attested, and verified BEFORE the GitHub Release publishes), `publish.yml` (crates.io via Trusted Publishing OIDC), `docker.yml`, and `back-merge.yml`; the published release then triggers `package-homebrew.yml` (updates `zircote/homebrew-tap`). Tag immediately after merging so changelog diffs stay clean. A `workflow_dispatch` of `release.yml` is a dry run: same chain, publish skipped.
- **Hotfixes**: branch from `main`, PR into `main`, tag the release, then merge `main` back into `develop`.
- **Do not reinvent provided automation.** The repo ships workflows for release steps (`release-pr.yml`, `changelog.yml`, `back-merge.yml`, etc.) — use them rather than equivalent manual `gh`/`git` commands, so the documented flow stays exercised and honest.

## Code Style Rules

- Prefer `&str` over `String` and `&[T]` over `Vec<T>` in function parameters
- Use `Cow<'_, str>` for flexible string returns
- Use `const fn` where possible, `#[must_use]` on value-returning functions
- All public items require doc comments with `# Arguments`, `# Returns`, `# Errors`, `# Examples`
- Group imports: std, external crates, crate-local
- `unsafe` code is forbidden (`unsafe_code = "forbid"` in Cargo.toml)

## NSIP MCP Server

The binary ships a built-in MCP server (`nsip mcp`) for sheep genetic evaluation with 13 tools, 7 guided prompts, and resource templates. See [`docs/MCP.md`](docs/MCP.md) for the full API reference and [`docs/llm-guides/`](docs/llm-guides/) for ready-to-use LLM instruction templates.

### MCP Features

- **Tool sets**: `--tools search,breed` to expose only specific tool categories
- **OAuth 2.1 + PAT auth**: `--auth` on HTTP transport with GitHub identity provider (env: `NSIP_GITHUB_CLIENT_ID`, `NSIP_GITHUB_CLIENT_SECRET`, `NSIP_AUTH_SECRET`, `NSIP_AUTH_BASE_URL`)
- **Telemetry**: `--features telemetry` enables `OpenTelemetry` trace context in JSON logs
