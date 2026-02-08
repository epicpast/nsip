# justfile — local CI parity for nsip
# Run `just` to list all available recipes.

set shell := ["bash", "-euo", "pipefail", "-c"]

# List available recipes
default:
    @just --list

# === Core Development ===

# Full CI check: fmt, clippy, test, doc, deny, coverage
check: fmt-check lint test doc-build deny coverage-gate

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the binary
run *ARGS:
    cargo run -- {{ ARGS }}

# Run all tests
test:
    cargo test --all-features

# Run tests with stdout visible
test-verbose:
    cargo test --all-features -- --nocapture

# Run a specific test by name
test-single NAME:
    cargo test {{ NAME }}

# Build and open documentation
doc:
    cargo doc --no-deps --all-features --open

# Build documentation without opening
doc-build:
    cargo doc --no-deps --all-features

# Watch for changes and re-run tests
watch:
    cargo watch -x 'test --all-features'

# === Linting & Formatting ===

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run clippy with CI-equivalent flags
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy and auto-fix what it can
lint-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty

# === Security & Audit ===

# Run cargo-deny supply chain checks
deny:
    cargo deny check

# Run cargo-audit advisory database check
audit:
    cargo audit --deny warnings

# Generate SBOM in SPDX format
sbom:
    cargo sbom --output-format spdx_json_2_3

# === Coverage ===

# Fail if line coverage drops below 90%
coverage-gate:
    rustup run stable cargo llvm-cov --all-features --fail-under-lines 90

# Generate LCOV coverage report
coverage:
    rustup run stable cargo llvm-cov --all-features --lcov --output-path lcov.info

# Generate HTML coverage report
coverage-html:
    rustup run stable cargo llvm-cov --all-features --html --output-dir coverage-html

# Print coverage summary to stdout
coverage-summary:
    rustup run stable cargo llvm-cov --all-features --summary-only

# === Advanced Testing ===

# Check against minimum supported Rust version
msrv:
    cargo +1.92 check --all-features

# Run tests under Miri for undefined behavior detection
miri:
    cargo +nightly miri test

# Run benchmarks
bench:
    cargo bench --workspace

# Run a fuzz target for a given duration (seconds)
fuzz TARGET DURATION="60":
    cargo fuzz run {{ TARGET }} -- -max_total_time={{ DURATION }}

# Run mutation testing
mutants:
    cargo mutants --output mutants.out --json

# === Release ===

# Dry-run a crates.io publish
publish-dry:
    cargo publish --dry-run

# Generate changelog for the latest release
changelog:
    git-cliff --latest --strip header

# === Shell Completions & Man Pages ===

# Generate shell completions for all supported shells
completions:
    @mkdir -p completions
    cargo run -- completions bash > completions/nsip.bash
    cargo run -- completions zsh > completions/_nsip
    cargo run -- completions fish > completions/nsip.fish
    @echo "Completions generated in completions/"

# Generate man pages for all commands
man:
    @mkdir -p man
    cargo run -- man-pages --out-dir man/
    @echo "Man pages generated in man/"
