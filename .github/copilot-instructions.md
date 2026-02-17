# GitHub Copilot Instructions

## Project Context

- **Rust 1.92+** (2024 edition), source in `crates/`, not `src/`
- **Binary**: `crates/main.rs` — Library: `crates/lib.rs`
- **MCP server**: `crates/mcp/` — launched via `nsip mcp` (stdio transport)
- **Clippy**: pedantic + nursery + cargo lints with `#[deny(unwrap_used, expect_used, panic)]`
- **Error handling**: `thiserror` for error types, `Result<T>` alias, `?` propagation
- **Builder pattern**: `const fn` constructors, `#[must_use]` on value-returning functions
- **Parameters**: prefer `&str` over `String`, `&[T]` over `Vec<T>`, `Cow<'_, str>` for flexible returns
- **Formatting**: rustfmt with 100-char lines, `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`
- **Supply chain**: `cargo-deny` enforces permissive licenses only, bans `openssl` (use rustls)
- **`unsafe` code is forbidden**

See `crates/**/*.rs` and `tests/**/*.rs` scoped instructions for detailed coding and test guidelines.

## Commands

```bash
just                  # List all recipes
just check            # Full CI check (fmt + clippy + test + doc + deny)
just test             # Run all tests
just lint             # Clippy with CI flags
just fmt              # Format code
just deny             # Supply chain audit
```

<details>
<summary>Raw cargo commands</summary>

```bash
cargo build                                              # Build
cargo test --all-features                                # Run all tests
cargo clippy --all-targets --all-features -- -D warnings # Lint
cargo fmt                                                # Format
cargo doc --no-deps                                      # Generate docs
cargo deny check                                         # Supply chain audit
```

</details>

## NSIP MCP Server

The binary includes an MCP server (`nsip mcp`) for sheep genetic evaluation.
Configure in `.mcp.json`. See `docs/MCP.md` for the full API reference and
`docs/llm-guides/` for ready-to-use LLM instruction templates.

Available tools: `search`, `details`, `lineage`, `progeny`, `profile`,
`breed_groups`, `trait_ranges`, `compare`, `rank`, `inbreeding_check`,
`mating_recommendations`, `flock_summary`, `database_status`.

See `.github/instructions/nsip-mcp.instructions.md` for full tool reference,
workflow recipes, and formatting rules.

## Flock Actions

Issues with the `flock-action` label request automated breeding analyses. Parse
the issue form fields, call the appropriate MCP tools, and produce a report PR
in `reports/`. See `.github/instructions/flock-action.instructions.md`.
