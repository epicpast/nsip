# Contributing

Thank you for your interest in contributing to nsip!

## Prerequisites

- [Rust](https://rustup.rs/) 1.92 or later
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for supply chain checks

## Getting Started

1. Fork the repository
2. Clone your fork:

   ```bash
   git clone https://github.com/YOUR_USERNAME/nsip.git
   cd nsip
   ```

3. Create a feature branch:

   ```bash
   git checkout -b feat/your-feature
   ```

4. Make your changes and verify:

   ```bash
   cargo fmt -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo doc --no-deps
   cargo deny check
   ```

   > **Tip:** This project uses the [`just`](https://github.com/casey/just) task
   > runner. `just check` runs the full CI-equivalent pipeline (fmt + clippy +
   > test + doc + deny + coverage) in one command. Run `just` to list all
   > available recipes.

## Commit Conventions

This project uses [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation changes
- `refactor:` code changes that neither fix bugs nor add features
- `test:` adding or updating tests
- `chore:` maintenance tasks
- `ci:` CI/CD changes

The changelog is generated from these prefixes via
[git-cliff](https://git-cliff.org/).

## Commit Signing

This project encourages signed commits. Signed commits display a
"Verified" badge on GitHub, confirming the committer's identity.

### SSH key signing (recommended)

Most developers already have SSH keys, making this the simplest
option:

```bash
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
```

Then upload the same public key as a **Signing Key** at
<https://github.com/settings/ssh/new> (select "Signing Key" as the
key type).

### gitsign (keyless, Sigstore OIDC)

For fully keyless signing via your GitHub/Google/Microsoft identity:

```bash
brew install sigstore/tap/gitsign          # macOS
# or: go install github.com/sigstore/gitsign@latest

git config --global gpg.format x509
git config --global gpg.x509.program gitsign
git config --global commit.gpgsign true
```

Each commit opens a browser for OIDC authentication. No keys to
manage.

## Pull Request Checklist

Before submitting a PR, ensure:

- [ ] Code compiles without warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] All tests pass (`cargo test --all-features`)
- [ ] Code is formatted (`cargo fmt -- --check`)
- [ ] Documentation builds (`cargo doc --no-deps`)
- [ ] Supply chain checks pass (`cargo deny check`)
- [ ] Public items have documentation with `# Examples` and `# Errors`
- [ ] No `unwrap()`, `expect()`, or `panic!()` in library code
- [ ] Error types use `thiserror`
- [ ] Commits are signed (see [Commit Signing](#commit-signing))

## Code Style

This project enforces strict linting via clippy with pedantic and nursery
lint groups. Key rules:

- **No panics in library code**: Use `Result` types for fallible operations
- **Document all public items**: Include examples and error conditions
- **Prefer borrowing**: Use `&str` over `String`, `&[T]` over `Vec<T>`
- **Use `const fn`** where possible
- **Line length**: 100 characters maximum

See [CLAUDE.md](CLAUDE.md) for detailed coding patterns and examples.

## Operational Runbooks

For ongoing project operations, see the runbooks in [`docs/runbooks/`](docs/runbooks/):

- [Releasing](docs/runbooks/RELEASING.md) — End-to-end release process
- [Dependency Updates](docs/runbooks/DEPENDENCY-UPDATES.md) — Dependabot policy and manual auditing
- [Security Response](docs/runbooks/SECURITY-RESPONSE.md) — Vulnerability triage and disclosure
- [CI Troubleshooting](docs/runbooks/CI-TROUBLESHOOTING.md) — Common CI failure patterns and fixes

## AI-Assisted Contributions

AI-assisted contributions are welcome. If using AI tools, please ensure
generated code follows the same quality standards as hand-written code.
See `CLAUDE.md` and `.github/copilot-instructions.md` for AI-specific
guidelines.
