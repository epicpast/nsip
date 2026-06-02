---
diataxis_type: how-to
---
# Jumpstart Your Project with Copilot

When you create a new repository from this template, GitHub offers an optional **"Jumpstart your project with Copilot"** prompt. Paste one of the prompts below into that field, and Copilot will open a pull request that scaffolds your project.

> **How it works:** After repository creation, Copilot reads your prompt, generates the files, and opens a PR for your review. All 18 CI/CD workflows, security tooling, Docker setup, and linting configuration are preserved automatically.

---

## Default Prompt

Use this generic prompt for any Rust project. Copilot infers your crate name from the repo.

```text
Rename nsip/nsip/zircote to match this repo everywhere. In crates/lib.rs add types, thiserror errors, public functions with doc comments and examples. In crates/main.rs create a binary using the library. Update Cargo.toml metadata and README.md. Do NOT modify .github/workflows/ or config files. Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

---

## Example Use-Case Prompts

### 1. CLI Tool — File Search Utility

```text
Rename nsip/nsip/zircote to match this repo. Scaffold a CLI search tool. Add clap (derive), regex, ignore, colored. Create crates/search.rs, pattern.rs, output.rs, walker.rs. In lib.rs define SearchResult, SearchConfig (builder), errors. In main.rs build clap CLI with pattern+path args. Write tests and proptest. Update README.md. Do NOT modify .github/workflows/ or config files. Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### 2. REST API Server — Task Manager

```text
Rename nsip/nsip/zircote to match this repo. Scaffold a REST API with axum. Uncomment tokio+serde in Cargo.toml, add axum, serde_json, uuid, tower-http. Create crates/routes.rs, models.rs (Task), state.rs, error.rs. In lib.rs expose build_router(). In main.rs bind 0.0.0.0:3000. Write tests for CRUD /tasks. Update README.md. Do NOT modify .github/workflows/ or config files. Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### 3. Library Crate — Data Validation

```text
Rename nsip/nsip/zircote to match this repo. Scaffold a validation library. Add serde, serde_json. Remove [[bin]] and crates/main.rs. Create crates/rule.rs (Required, MinLength, Range, Email), schema.rs, result.rs, types.rs. In lib.rs provide fluent API. Write unit+doc tests and proptest. Update README.md. Do NOT modify .github/workflows/ or config files. Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### 4. System Utility — Log Aggregator

```text
Rename nsip/nsip/zircote to match this repo. Scaffold an async log aggregator. Uncomment tokio+tracing, add clap (derive), notify, chrono, serde, colored. Create crates/watcher.rs, parser.rs, filter.rs, output.rs. In lib.rs define LogEntry, FilterConfig (builder). In main.rs build clap CLI. Write tests+proptest. Update README.md. Do NOT modify .github/workflows/ or config files. Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### 5. WebAssembly Module — Text Processing

```text
Rename nsip/nsip/zircote to match this repo. Scaffold a WASM text library. Add wasm-bindgen, js-sys, unicode-segmentation, serde. Set crate-type=["cdylib","rlib"]. Remove [[bin]] and crates/main.rs. Create crates/transform.rs, analyze.rs, sanitize.rs. Re-export via lib.rs. Write tests and proptest. Update README.md. Do NOT modify .github/workflows/ or config files. Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### 6. MCP Server — AI Tool Provider

```text
Rename nsip/nsip/zircote to match this repo. Scaffold an MCP server. Uncomment tokio+serde, add serde_json, rmcp (transport-io, server). Create crates/server.rs (stdio), tools.rs (#[tool]: file_stats, json_format, hash_digest), types.rs. In lib.rs expose build_server(). In main.rs start stdio. Write tests. Update README.md. Do NOT modify .github/workflows/ or config files. Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

---

## Tips for Writing Your Own Prompt

- **Start with the rename** — always begin with "Rename nsip/nsip/zircote to match this repo" so Copilot updates all references
- **List dependencies** — mention which commented-out deps in Cargo.toml to uncomment and what new ones to add
- **Module structure** — source files go under `crates/`, not `src/`
- **Library-only crates** — tell Copilot to remove the `[[bin]]` section and `crates/main.rs`
- **Always verify** — end with `Run: cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`
- **Protect CI/CD** — always include "Do NOT modify .github/workflows/ or config files"
- **500-character limit** — the Jumpstart field accepts up to 500 characters; keep prompts concise
