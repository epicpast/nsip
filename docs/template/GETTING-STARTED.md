---
diataxis_type: tutorial
---
# Getting Started

> You just created a repository from **zircote/nsip**. This guide walks you through every step from creation to your first green CI run.

---

## 1. Create Your Repository

- [ ] Go to [zircote/nsip](https://github.com/zircote/nsip) and click **"Use this template"** > **"Create a new repository"**.
- [ ] Choose an **owner** (your user or an organization).
- [ ] Name your repository (e.g., `my-awesome-crate`).
- [ ] Select **Public** or **Private** visibility.
- [ ] Click **"Create repository"**.

> **Optional:** During creation, GitHub offers a **"Jumpstart your project with Copilot"** field. You can paste a prompt there to have Copilot scaffold your project with real types, functions, and tests in an auto-opened PR. See [Copilot Jumpstart](COPILOT-JUMPSTART.md) for ready-made prompts.

---

## 2. Initialize the Project

After creating your repository from the template, replace the template
placeholders across the repository with your project's values:

| Template placeholder | Replaced with | Example |
|---|---|---|
| `zircote/nsip` | `your-org/your-repo` | `acme/my-awesome-crate` |
| `zircote` | your GitHub owner | `acme` |
| `nsip` | your repository name | `my-awesome-crate` |
| `nsip` | your crate name (underscored) | `my_awesome_crate` |

> The automated **Template Init** workflow is not included in this repository.
> Perform the replacements manually (e.g. with `sed`/`rg`), or restore a
> template-init workflow if you want this automated. After replacing, confirm
> `Cargo.toml`, `README.md`, and documentation links point to your project.

> **What copies and what doesn't?** Files copy; settings don't. See [GitHub Template Features](GITHUB-TEMPLATE-FEATURES.md) for the full breakdown of what transfers when you use a template repository.

---

## 3. Clone and Build

After initialization, pull down your freshly initialized repo:

```bash
git clone https://github.com/<your-org>/<your-repo>.git
cd <your-repo>
```

Build and run the test suite:

```bash
cargo build
cargo test
```

### Requirements

| Requirement | Minimum version |
|---|---|
| Rust toolchain | **1.92** or newer |
| Rust edition | **2024** |
| cargo-deny (optional, for supply chain checks) | latest stable |

Install the latest Rust toolchain with [rustup](https://rustup.rs/):

```bash
rustup default stable
rustup update
```

---

## 4. Explore the Structure

```
.
├── crates/
│   ├── lib.rs              # Library entry point and public API
│   └── main.rs             # Binary entry point (optional)
├── tests/
│   └── integration_test.rs # Integration tests
├── benches/                # Benchmarks (criterion)
├── examples/               # Example programs
├── docs/                   # Documentation
├── .github/
│   └── workflows/          # CI/CD pipelines
├── Cargo.toml              # Package manifest
├── deny.toml               # cargo-deny configuration
├── rustfmt.toml            # Formatting rules
└── clippy.toml             # Linting rules
```

Key points:

- **Source code** lives under `crates/`, not `src/`. The paths are configured in `Cargo.toml` via `[lib]` and `[[bin]]`.
- **Unit tests** go inside `crates/*.rs` files within `#[cfg(test)]` modules.
- **Integration tests** go in the `tests/` directory.
- **CI/CD workflows** are in `.github/workflows/`. The template ships with 30+ workflows covering CI, security, releases, and more.

---

## 5. First CI Pass

Push any change (or wait for the init commit) and verify CI passes:

```bash
git add -A
git commit -m "feat: initial implementation"
git push
```

Open the **Actions** tab in your GitHub repository to watch the pipeline. The core CI workflow runs these checks:

| Check | What it does | Command |
|---|---|---|
| **Format** | Enforces consistent code style | `cargo fmt --all -- --check` |
| **Clippy** | Lints with pedantic + nursery rules | `cargo clippy --all-targets --all-features -- -D warnings` |
| **Test** | Runs tests on Linux, macOS, and Windows | `cargo test --all-features --verbose` |
| **Documentation** | Verifies rustdoc builds cleanly | `cargo doc --no-deps --all-features` |
| **Cargo Deny** | Audits licenses, advisories, and sources | `cargo deny check` |
| **MSRV** | Confirms the crate builds on Rust 1.92 | `cargo check --all-features` |
| **Coverage** | Generates code coverage via `cargo-llvm-cov` | `cargo llvm-cov --all-features` |

All checks (except coverage) must pass for the **"All Checks Pass"** gate to go green.

> For details on every workflow included in this template, see [CI Workflows](CI-WORKFLOWS.md).

---

## 6. Configure Your Project

Open `Cargo.toml` and update the package metadata to match your project:

```toml
[package]
name = "your_crate_name"          # already set by template-init
version = "0.1.0"
edition = "2024"
rust-version = "1.92"
description = "A short description of your crate"  # <-- update
license = "MIT"                                      # <-- update if needed
authors = ["Your Name <you@example.com>"]            # <-- update
repository = "https://github.com/you/your-repo"     # already set by template-init
keywords = ["your", "keywords"]                      # <-- update
categories = ["development-tools"]                   # <-- update
```

Checklist:

- [ ] Set `description` to a one-line summary of your crate.
- [ ] Set `authors` to the correct name and email.
- [ ] Choose the appropriate `license` (MIT, Apache-2.0, or dual).
- [ ] Update `keywords` (up to 5) and `categories` for crates.io discoverability.
- [ ] Update `README.md` with your project's purpose, usage examples, and badges.

> For a comprehensive configuration walkthrough, see [Configuration](CONFIGURATION.md).

---

## 7. Required Secrets

Most workflows use only the automatic `GITHUB_TOKEN`. Optional workflows require additional secrets configured in **Settings > Secrets and variables > Actions**:

| Secret | Required for | How to obtain |
|---|---|---|
| `GITHUB_TOKEN` | All workflows (CI, releases, etc.) | Automatic -- provided by GitHub Actions |
| `CARGO_REGISTRY_TOKEN` | Publishing to crates.io (`publish.yml`) | [crates.io account settings](https://crates.io/settings/tokens) |
| `CODECOV_TOKEN` | Uploading coverage reports (`ci.yml`) | [Codecov dashboard](https://app.codecov.io/) after linking your repo |

> Workflows that reference missing secrets will either skip gracefully or fail with a clear error. You only need to configure a secret when you are ready to use the corresponding feature.

---

## 8. Enable Commit Signing

The template's release workflows already sign artifacts with Sigstore
Cosign and generate SLSA Level 3 provenance. To extend signing to
individual commits:

**Enable in branch protection:**

- [ ] Go to **Settings > Branches > Branch protection rules** for
  `main`.
- [ ] Check **"Require signed commits"**.

This ensures all commits merged into `main` carry a verified
signature.

**Contributor setup:**

- [ ] Point contributors to the
  [Commit Signing](../../CONTRIBUTING.md#commit-signing) section for
  SSH key or gitsign configuration.

**Enable vigilant mode (recommended for maintainers):**

- [ ] Go to **Settings > SSH and GPG keys** on your GitHub profile.
- [ ] Enable **Vigilant mode** so unsigned commits display an
  "Unverified" badge, making it easy to spot gaps.

---

## 9. Next Steps

You have a building, tested, CI-validated Rust project. Here is where to go from here:

- **[Configuration](CONFIGURATION.md)** -- Full guide to `Cargo.toml`, feature flags, profiles, and lints.
- **[CI Workflows](CI-WORKFLOWS.md)** -- Deep dive into every workflow: triggers, secrets, and customization.
- **[Customization](CUSTOMIZATION.md)** -- How to add modules, remove the binary target, enable async, and tailor the template to your needs.
- **[CONTRIBUTING.md](../../CONTRIBUTING.md)** -- Contribution guidelines for your collaborators.
