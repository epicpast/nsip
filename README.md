# `nsip`

<!-- Badges -->
[![GitHub Template](https://img.shields.io/badge/template-zircote%2Frust--template-blue?logo=github)](https://github.com/zircote/nsip)
[![CI](https://github.com/zircote/nsip/actions/workflows/ci.yml/badge.svg)](https://github.com/zircote/nsip/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/nsip.svg?logo=rust&logoColor=white)](https://crates.io/crates/nsip)
[![Documentation](https://docs.rs/nsip/badge.svg)](https://docs.rs/nsip)
[![Rust Version](https://img.shields.io/badge/rust-1.92%2B-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/zircote/nsip/blob/main/LICENSE)
[![Clippy](https://img.shields.io/badge/linting-clippy-orange?logo=rust&logoColor=white)](https://github.com/rust-lang/rust-clippy)
[![cargo-deny](https://img.shields.io/badge/security-cargo--deny-blue?logo=rust&logoColor=white)](https://github.com/EmbarkStudios/cargo-deny)
[![Security: gitleaks](https://img.shields.io/badge/security-gitleaks-blue?logo=git&logoColor=white)](https://github.com/gitleaks/gitleaks)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-025e8c?logo=dependabot)](https://docs.github.com/en/code-security/dependabot)

NSIP Search API client for querying livestock data from nsipsearch.nsip.org/api.

## Features

- **Type-safe API client** with comprehensive error handling
- **Search functionality** for animals by breed group, status, and other criteria
- **Detailed animal information** including lineage and progeny
- **MCP (Model Context Protocol) integration** for AI assistant compatibility
- **CLI tool** with multiple subcommands for easy interaction
- **Async/await support** with tokio runtime
- **Full documentation** with examples in all public APIs

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nsip = "0.1"
```

Or use cargo add:

```bash
cargo add nsip
```

## Quick Start

```rust,no_run
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    // Create a new client
    let client = NsipClient::new();

    // List available breed groups
    let breed_groups = client.breed_groups().await?;
    println!("Available breed groups: {}", breed_groups.len());

    // Search for animals
    let criteria = SearchCriteria::new()
        .with_breed_id(640)
        .with_status("CURRENT");

    let results = client
        .search_animals(0, 15, Some(640), None, None, Some(&criteria))
        .await?;
    println!("Found {} animals", results.total_count);

    // Get details for a specific animal
    let animal = client.animal_details("LPN_ID_HERE").await?;
    println!("Animal: {}", animal.lpn_id);

    Ok(())
}
```

## CLI Usage

The `nsip` CLI provides several commands for interacting with the NSIP Search API:

```bash
# Get database last-updated date
nsip date-updated

# List breed groups
nsip breed-groups

# List animal statuses
nsip statuses

# Get trait ranges for a breed
nsip trait-ranges 640

# Search for animals
nsip search --breed-id 640 --status CURRENT --page 0 --page-size 15

# Get animal details
nsip details <lpn-id>

# Get animal lineage
nsip lineage <lpn-id>

# Get animal progeny
nsip progeny <lpn-id>

# Get full profile (details + lineage + progeny)
nsip profile <lpn-id>

# Start MCP server mode
nsip mcp
```

## API Overview

### Client Methods

| Method | Description |
|--------|-------------|
| `date_last_updated()` | Get database last-updated date |
| `breed_groups()` | List available breed groups |
| `statuses()` | List available animal statuses |
| `trait_ranges(breed_id)` | Get trait ranges for a breed |
| `search_animals(page, page_size, breed_id, sorted_trait, reverse, criteria)` | Search for animals |
| `animal_details(search_string)` | Get animal details |
| `lineage(lpn_id)` | Get animal lineage |
| `progeny(lpn_id, page, page_size)` | Get animal progeny |
| `search_by_lpn(lpn_id)` | Get full profile (concurrent) |

### Data Types

| Type | Description |
|------|-------------|
| `NsipClient` | Main API client |
| `SearchCriteria` | Search parameters with builder pattern |
| `AnimalDetails` | Detailed animal record with traits and contact info |
| `AnimalProfile` | Combined details + lineage + progeny |
| `BreedGroup` | Breed group with nested breeds |
| `Lineage` | Animal lineage/ancestry tree |
| `Progeny` | Paginated animal offspring |
| `SearchResults` | Paginated search results |
| `Error` | Error type for operations |
| `Result<T>` | Type alias for `Result<T, Error>` |

## MCP Integration

The library includes MCP (Model Context Protocol) support for integration with AI assistants:

```rust,ignore
// Start the MCP server on stdio
nsip::mcp::serve_stdio().await?;
```

The MCP protocol exposes the following tools when running `nsip mcp`:
- `search` - Search for animals with filters
- `details` - Get detailed animal information
- `lineage` - Get animal lineage/ancestry

## Development

### Prerequisites

- Rust 1.92+ (2024 edition)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for supply chain security

### Setup

```bash
# Clone the repository
git clone https://github.com/zircote/nsip.git
cd nsip

# Build
cargo build

# Run tests
cargo test

# Run linting
cargo clippy --all-targets --all-features

# Format code
cargo fmt

# Check supply chain security
cargo deny check

# Generate documentation
cargo doc --open
```

### Project Structure

```text
crates/
├── lib.rs           # Library entry point
├── main.rs          # Binary entry point
└── ...              # Additional modules

tests/
└── integration_test.rs

Cargo.toml           # Project manifest
clippy.toml          # Clippy configuration
rustfmt.toml         # Formatter configuration
deny.toml            # cargo-deny configuration
CLAUDE.md            # AI assistant instructions
AGENTS.md            # AI coding agent instructions
.editorconfig        # Cross-editor defaults
.devcontainer/       # Codespaces / dev container config
.vscode/             # VS Code settings and extensions
```

### Code Quality

This project maintains high code quality standards:

- **Linting**: clippy with pedantic and nursery lints
- **Formatting**: rustfmt with custom configuration
- **Testing**: Unit tests, integration tests, and property-based tests
- **Documentation**: All public APIs documented with examples
- **Supply Chain**: cargo-deny for dependency auditing
- **CI/CD**: GitHub Actions for automated testing

### Running Checks

```bash
# Run all checks
cargo fmt -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test && \
cargo doc --no-deps && \
cargo deny check

# Run with MIRI for undefined behavior detection
cargo +nightly miri test
```

## CI/CD and Deployment

This template includes production-ready workflows:

### Continuous Integration

- **CI** (`.github/workflows/ci.yml`) - Format, lint, test, docs, supply chain security, MSRV check, coverage
- **Security Audit** (`.github/workflows/security-audit.yml`) - Daily cargo-audit scans
- **`CodeQL` Analysis** (`.github/workflows/codeql-analysis.yml`) - SAST scanning on push/PR and weekly schedule
- **Benchmark** (`.github/workflows/benchmark.yml`) - Performance tracking with criterion
- **ADR Validation** (`.github/workflows/adr-validation.yml`) - Architectural decision records validation

### Release and Deployment

- **Release** (`.github/workflows/release.yml`) - Automated GitHub releases with multi-platform binaries
  - Builds for: Linux (`x86_64`, ARM64), macOS (`x86_64`, ARM64), Windows (`x86_64`)
  - Automatic changelog generation
  - Binary artifacts uploaded to releases

- **Changelog** (`.github/workflows/changelog.yml`) - Automated CHANGELOG.md generation
  - Uses git-cliff with conventional commits
  - Follows Keep a Changelog format
  - Triggered on version tags

- **Docker** (`.github/workflows/docker.yml`) - Multi-platform container builds
  - Platforms: linux/amd64, linux/arm64
  - Distroless base image for security
  - Published to GitHub Container Registry (ghcr.io)
  - Tagged with version and 'latest'

- **Publish** (`.github/workflows/publish.yml`) - Automated crates.io publishing
  - Full pre-publish validation
  - Triggered on version tags
  - Requires `CARGO_REGISTRY_TOKEN` secret

### Creating a Release

1. Update version in `Cargo.toml`
2. Create and push a version tag:
   ```bash
   git tag -a v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```
3. Workflows automatically:
   - Generate changelog
   - Build binaries for all platforms
   - Create GitHub release with artifacts
   - Build and push Docker images
   - Publish to crates.io

### AI Coding Agent

- **Copilot Setup** (`.github/workflows/copilot-setup-steps.yml`) - Environment for GitHub Copilot coding agent
- **Agent Instructions**: `AGENTS.md`, `.github/copilot-instructions.md`, `CLAUDE.md`
- **Path-Specific Instructions**: `.github/instructions/` for Rust code and test patterns
- **Reusable Prompts**: `.github/prompts/` for common development tasks

### Docker Usage

Pull and run the container:

```bash
# Pull latest
docker pull ghcr.io/zircote/nsip:latest

# Run specific version
docker pull ghcr.io/zircote/nsip:v0.1.0
docker run --rm ghcr.io/zircote/nsip:v0.1.0 --version
```

## MSRV Policy

The Minimum Supported Rust Version (MSRV) is **1.92**. Increasing the MSRV is considered a minor breaking change.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, PR checklist, and coding standards.

Please also review:
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community guidelines
- [SECURITY.md](SECURITY.md) - Vulnerability reporting
- [GOVERNANCE.md](GOVERNANCE.md) - Decision-making process

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/zircote/nsip/blob/main/LICENSE) file for details.

## Acknowledgments

- [The Rust Programming Language](https://www.rust-lang.org/)
- [Cargo](https://doc.rust-lang.org/cargo/)
- [clippy](https://github.com/rust-lang/rust-clippy)
