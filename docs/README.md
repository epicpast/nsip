# Documentation Index

> All documentation for the nsip project.

## Template Adoption Guides

Guides for developers who just created a repository from this template.

| Guide | Description |
|-------|-------------|
| [Getting Started](template/GETTING-STARTED.md) | "Use this template" → first `cargo build` → first CI pass |
| [Configuration](template/CONFIGURATION.md) | Cargo.toml fields, placeholder replacement, feature flags, editor setup |
| [CI Workflows](template/CI-WORKFLOWS.md) | Every included workflow: triggers, secrets, how to enable/disable |
| [Customization](template/CUSTOMIZATION.md) | Add modules, remove examples, adjust lints, modify release targets |
| [GitHub Template Features](template/GITHUB-TEMPLATE-FEATURES.md) | What copies when using a template — and what doesn't |
| [Copilot Jumpstart](template/COPILOT-JUMPSTART.md) | Prompts for automatic project scaffolding with GitHub Copilot |

## Operational Runbooks

Step-by-step procedures for ongoing project maintenance.

| Runbook | Description |
|---------|-------------|
| [Releasing](runbooks/RELEASING.md) | Version bump → tag → monitor workflows → verify artifacts |
| [Dependency Updates](runbooks/DEPENDENCY-UPDATES.md) | Dependabot policy, manual cargo-deny audit, handling advisories |
| [Security Response](runbooks/SECURITY-RESPONSE.md) | Vulnerability triage, fix, coordinated disclosure |
| [CI Troubleshooting](runbooks/CI-TROUBLESHOOTING.md) | Common CI failure patterns and fixes |

## Reference Documentation

Detailed reference material organized by topic.

### Workflows

| Document | Description |
|----------|-------------|
| [Coverage](workflows/COVERAGE.md) | Code coverage configuration and reporting |
| [Test Matrix](workflows/TEST-MATRIX.md) | Multi-platform and multi-version test matrix |
| [Benchmark Regression](workflows/BENCHMARK-REGRESSION.md) | Performance regression detection |
| [Mutation Testing](workflows/MUTATION-TESTING.md) | Mutation testing with cargo-mutants |
| [Fuzz Testing](workflows/FUZZ-TESTING.md) | Fuzz testing with cargo-fuzz |
| [Code Quality](workflows/CODE-QUALITY.md) | Code quality metrics and analysis |
| [Spell Check](workflows/SPELL-CHECK.md) | Spell checking configuration |
| [SBOM](workflows/SBOM.md) | Software Bill of Materials generation |
| [Secrets Scan](workflows/SECRETS-SCAN.md) | Secret scanning with Gitleaks |
| [Container Scan](workflows/CONTAINER-SCAN.md) | Container image vulnerability scanning |

### Security

| Document | Description |
|----------|-------------|
| [Signed Releases](security/SIGNED-RELEASES.md) | Release signing and verification |

### Distribution

| Document | Description |
|----------|-------------|
| [Package Managers](distribution/PACKAGE-MANAGERS.md) | Homebrew, Snap, and system package publishing |
| [Docker Registries](distribution/DOCKER-REGISTRIES.md) | Docker Hub and GHCR publishing |
| [Alternative Registries](distribution/ALTERNATIVE-REGISTRIES.md) | Alternative Rust package registries |

### Testing

| Document | Description |
|----------|-------------|
| [Property-Based Testing](testing/PROPERTY-BASED-TESTING.md) | proptest setup and patterns |

### UX

| Document | Description |
|----------|-------------|
| [Shell Completions](ux/SHELL-COMPLETIONS.md) | Shell completion generation |
| [Man Pages](ux/MAN-PAGES.md) | Man page generation |

### Observability

| Document | Description |
|----------|-------------|
| [Metrics Dashboard](observability/METRICS-DASHBOARD.md) | Metrics and monitoring setup |

### Deployment

| Document | Description |
|----------|-------------|
| [Deployment Guide](DEPLOYMENT.md) | Comprehensive deployment instructions |

## Architectural Decision Records

| ADR | Description |
|-----|-------------|
| [ADR-0001](adr/0001-use-architectural-decision-records.md) | Use Architectural Decision Records |
| [ADR-0002](adr/0002-documentation-directory-structure.md) | Documentation Directory Structure |

See [docs/adr/README.md](adr/README.md) for the full ADR process and workflow.
