# Template Configuration Guide

> How to configure your new repository after creating it from the rust-template.

This guide covers every customization point in the template, from automatic placeholder replacement to editor and AI assistant configuration.

---

## 1. Placeholder Replacement

The `template-init.yml` workflow runs automatically on the first push to `main` after you create a repo from this template. It replaces all template placeholders with your project's values.

### Placeholders

| Placeholder | Replaced With | Example |
|-------------|---------------|---------|
| `zircote/rust-template` | `your-org/your-repo` (full path) | `acme/my-cli` |
| `zircote/rust_template` | `your-org/your_crate` (full path, underscored) | `acme/my_cli` |
| `zircote` | Your GitHub org or username | `acme` |
| `rust-template` | Your repo name (hyphenated) | `my-cli` |
| `rust_template` | Your crate name (underscored) | `my_cli` |

### How It Works

1. You click **"Use this template"** on GitHub.
2. You push to `main` (or the initial commit triggers the workflow).
3. The workflow detects `name = "rust_template"` in `Cargo.toml`.
4. It runs `sed` replacements across all eligible files.
5. It regenerates `Cargo.lock` and commits the result.

### Files Excluded from Replacement

The workflow skips these paths to avoid corrupting binaries or breaking CI:

- `.git/*` -- Git internals
- `.github/workflows/*` -- CI workflow files
- `*.png`, `*.jpg`, `*.ico` -- Binary image files
- `Cargo.lock` -- Regenerated after replacement

### Manual Replacement

If you need to run replacement manually (e.g., you disabled Actions), use:

```bash
# Set your values
OWNER="your-org"
REPO="your-repo"
CRATE="$(echo "$REPO" | tr '-' '_')"

# Replace in all text files (macOS sed)
find . -type f \
  ! -path './.git/*' \
  ! -path './.github/workflows/*' \
  ! -name '*.png' ! -name '*.jpg' ! -name '*.ico' \
  ! -name 'Cargo.lock' \
  -exec sed -i '' \
    -e "s|zircote/rust-template|${OWNER}/${REPO}|g" \
    -e "s|zircote/rust_template|${OWNER}/${CRATE}|g" \
    -e "s|zircote|${OWNER}|g" \
    -e "s|rust-template|${REPO}|g" \
    -e "s|rust_template|${CRATE}|g" \
    {} +

cargo generate-lockfile
```

---

## 2. Cargo.toml Fields to Update

After placeholder replacement runs, review and update these fields in `Cargo.toml`:

| Field | Default | Action |
|-------|---------|--------|
| `name` | `rust_template` | Auto-replaced by `template-init` |
| `version` | `0.1.0` | Update for releases |
| `edition` | `2024` | Leave as-is unless you need an older edition |
| `rust-version` | `1.92` | Update if changing MSRV (see section 6) |
| `authors` | `["Your Name <you@example.com>"]` | Replace with your name and email |
| `description` | `"A Rust template crate..."` | Replace with your crate's description |
| `repository` | `https://github.com/zircote/rust-template` | Auto-replaced by `template-init` |
| `homepage` | `https://github.com/zircote/rust-template` | Auto-replaced by `template-init` |
| `documentation` | `https://docs.rs/rust_template` | Auto-replaced by `template-init` |
| `license` | `MIT` | Change if using a different license |
| `keywords` | `["template", "rust", "example"]` | Replace with up to 5 relevant keywords |
| `categories` | `["development-tools"]` | Replace with applicable [crate categories](https://crates.io/category_slugs) |

### Post-Init Checklist

- [ ] Update `authors` with your real name/email
- [ ] Write a meaningful `description`
- [ ] Replace `keywords` with terms relevant to your crate
- [ ] Choose appropriate `categories` from the [crates.io category list](https://crates.io/category_slugs)
- [ ] Verify `license` matches your `LICENSE` file
- [ ] Remove the `[[bin]]` section if building a library-only crate

---

## 3. Optional Dependencies

The template `Cargo.toml` includes commented-out dependency blocks. Uncomment what you need:

### Async Runtime (tokio)

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tokio-test = "0.4"
```

### Serialization (serde)

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Structured Logging (tracing)

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### CLI Argument Parsing (clap)

Add `clap` manually -- it is not pre-included in the template:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

### Steps to Enable

1. Open `Cargo.toml`.
2. Uncomment the relevant lines under `[dependencies]` and `[dev-dependencies]`.
3. Run `cargo check` to verify resolution.
4. If the dependency has a restrictive license, add it to the `allow` list in `deny.toml`.

---

## 4. Feature Flags

The template includes an empty feature flags section in `Cargo.toml`:

```toml
[features]
default = []
# full = ["feature1", "feature2"]
```

### Defining Features

```toml
[features]
default = []
async = ["dep:tokio", "dep:tokio-test"]
serde = ["dep:serde", "dep:serde_json"]
full = ["async", "serde"]
```

### Using Features in Code

Gate modules and items behind feature flags with `cfg` attributes:

```rust
#[cfg(feature = "async")]
pub mod async_client;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
}
```

### Building and Testing with Features

```bash
# Build with a specific feature
cargo build --features async

# Build with all features
cargo build --all-features

# Test with no default features
cargo test --no-default-features

# Test with a combination
cargo test --features "async,serde"
```

### CI Consideration

The template CI runs `cargo clippy --all-targets --all-features` and `cargo test --all-features` by default. If your features are mutually exclusive, update the CI workflow matrix to test each combination separately.

---

## 5. Lint Configuration

Three configuration files control code quality rules. Adjust them to match your project's requirements.

### clippy.toml

Controls Clippy lint behavior and thresholds.

| Setting | Default | Purpose |
|---------|---------|---------|
| `msrv` | `"1.92"` | Minimum Rust version for lint suggestions |
| `cognitive-complexity-threshold` | `25` | Max cognitive complexity per function |
| `too-many-lines-threshold` | `100` | Max lines per function |
| `too-many-arguments-threshold` | `7` | Max function parameters |
| `excessive-nesting-threshold` | `4` | Max nesting depth |
| `max-struct-bools` | `3` | Max bool fields in a struct |
| `allow-unwrap-in-tests` | `true` | Permit `.unwrap()` in test code |
| `allow-expect-in-tests` | `true` | Permit `.expect()` in test code |

### rustfmt.toml

Controls code formatting rules.

| Setting | Default | Purpose |
|---------|---------|---------|
| `edition` | `"2024"` | Parsing edition |
| `max_width` | `100` | Maximum line width |
| `tab_spaces` | `4` | Spaces per indentation level |
| `imports_granularity` | `"Crate"` | How imports are grouped |
| `group_imports` | `"StdExternalCrate"` | Import section ordering |
| `wrap_comments` | `true` | Wrap long comments |
| `format_code_in_doc_comments` | `true` | Format code blocks in doc comments |
| `trailing_comma` | `"Vertical"` | Add trailing commas in multi-line contexts |

### deny.toml

Controls dependency auditing via `cargo-deny`.

| Section | What to Customize |
|---------|-------------------|
| `[advisories]` | Add crate IDs to `ignore` to suppress known advisories |
| `[licenses].allow` | Add SPDX identifiers for licenses your project permits |
| `[bans].deny` | Add crates you want to forbid (e.g., `openssl`) |
| `[bans].skip` | Allow specific duplicate dependency versions |
| `[sources].allow-git` | Add Git repository URLs for non-crates.io dependencies |

### Cargo.toml Lint Table

The `[lints.clippy]` section in `Cargo.toml` sets lint levels. Key denied lints:

```toml
unwrap_used = "deny"      # Use Result instead
expect_used = "deny"       # Use Result instead
panic = "deny"             # No panics in library code
todo = "deny"              # No incomplete code
unimplemented = "deny"     # No stubs
dbg_macro = "deny"         # No debug macros
print_stdout = "deny"      # Use tracing/log instead
print_stderr = "deny"      # Use tracing/log instead
```

To relax a lint for your project, change `"deny"` to `"warn"` or `"allow"`.

---

## 6. MSRV Policy

The current minimum supported Rust version is **1.92**.

### Changing the MSRV

Update all three locations to keep them in sync:

| File | Field | Example |
|------|-------|---------|
| `Cargo.toml` | `rust-version` | `rust-version = "1.85"` |
| `clippy.toml` | `msrv` | `msrv = "1.85"` |
| CI workflow matrix | `rust:` versions | Add your MSRV to the test matrix |

### Verification

```bash
# Install a specific toolchain to verify MSRV
rustup install 1.85
cargo +1.85 check
cargo +1.85 test
```

---

## 7. Editor Configuration

### .editorconfig

Cross-editor defaults applied automatically by editors that support [EditorConfig](https://editorconfig.org):

| Setting | Value | Scope |
|---------|-------|-------|
| `indent_style` | `space` | All files |
| `indent_size` | `4` | Default (2 for YAML/JSON) |
| `max_line_length` | `100` | All files |
| `end_of_line` | `lf` | All files |
| `charset` | `utf-8` | All files |
| `insert_final_newline` | `true` | All files |
| `trim_trailing_whitespace` | `true` | All files (except Markdown) |

### .vscode/settings.json

VS Code workspace settings:

- `rust-analyzer.check.command` set to `clippy` (lints on save)
- `rust-analyzer.check.extraArgs` includes `--all-targets --all-features`
- `editor.formatOnSave` enabled
- `editor.rulers` set to `[100]` to show the line-length guide
- `target/` directory excluded from file explorer

### .vscode/extensions.json

Recommended extensions (VS Code prompts to install these):

| Extension | Purpose |
|-----------|---------|
| `rust-lang.rust-analyzer` | Rust language server |
| `tamasfe.even-better-toml` | TOML syntax and validation |
| `serayuzgur.crates` | Inline crate version info |
| `usernamehw.errorlens` | Inline error/warning display |
| `vadimcn.vscode-lldb` | Native debugger |

### .devcontainer/

GitHub Codespaces and VS Code Dev Container configuration:

- **Base image:** `mcr.microsoft.com/devcontainers/rust:1-bookworm`
- **Post-create command:** installs `cargo-deny`, `cargo-tarpaulin`, `cargo-watch`, and runs `cargo fetch`
- **VS Code settings and extensions** mirror the workspace configuration above

To customize the dev container, edit `.devcontainer/devcontainer.json`. Common changes:

- Add system packages via `"features"` or a custom `Dockerfile`
- Add environment variables with `"containerEnv"`
- Forward ports with `"forwardPorts"`
- Install additional cargo tools in `"postCreateCommand"`

---

## 8. AI Assistant Configuration

The template includes configuration files for multiple AI coding assistants.

### CLAUDE.md

Instructions for [Claude Code](https://docs.anthropic.com/en/docs/claude-code). Located at the repo root.

- Build commands, project structure, and code style rules
- Error handling patterns and documentation requirements
- Testing conventions and CI/CD pipeline details

### AGENTS.md

Instructions for [GitHub Copilot coding agent](https://docs.github.com/en/copilot/using-github-copilot/using-copilot-coding-agent). Located at the repo root.

- Read by the Copilot coding agent when it works on issues and PRs
- Shares the same project conventions as `CLAUDE.md`

### .github/copilot-instructions.md

Instructions for [GitHub Copilot Chat](https://docs.github.com/en/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot). Loaded automatically in Copilot Chat conversations.

- Provides project-wide context for inline suggestions and chat responses

### .github/instructions/

[Path-specific instruction files](https://docs.github.com/en/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot) applied when Copilot works on matching file paths:

| File | Scope |
|------|-------|
| `rust-code.instructions.md` | Rust source files (`crates/**/*.rs`) |
| `tests.instructions.md` | Test files (`tests/**/*.rs`) |

### .github/prompts/

[Reusable prompt files](https://docs.github.com/en/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot) for common development tasks:

| Prompt | Purpose |
|--------|---------|
| `new-module.prompt.md` | Scaffold a new Rust module |
| `fix-clippy.prompt.md` | Fix Clippy lint warnings |
| `add-error-variant.prompt.md` | Add a new error type variant |
| `write-tests.prompt.md` | Generate tests for existing code |

### Customizing AI Instructions

- Edit `CLAUDE.md` and `AGENTS.md` at the repo root to update project-wide conventions.
- Add new `.instructions.md` files under `.github/instructions/` for path-specific rules.
- Add new `.prompt.md` files under `.github/prompts/` for reusable task prompts.
- All AI instruction files are regular Markdown -- placeholder replacement applies to them automatically.
