---
diataxis_type: explanation
---
# Documentation Index

> All documentation for the nsip project organized using the [Diataxis framework](https://diataxis.fr/).

## Quick Start

New to NSIP? Start here:

| Document | Description |
|----------|-------------|
| [Getting Started Tutorial](tutorials/GETTING-STARTED.md) | 15-minute hands-on introduction to the NSIP CLI and library |
| [Understanding EBVs](explanation/EBV-EXPLAINED.md) | Learn what Estimated Breeding Values are and how to interpret them |
| [CLI Reference](reference/CLI.md) | Complete reference for every CLI subcommand, flag, and option |

---

## Tutorials

Learning-oriented guides that take you through practical exercises step by step.

| Tutorial | Time | Description |
|----------|------|-------------|
| [Getting Started](tutorials/GETTING-STARTED.md) | 15 min | Install nsip, search for animals, retrieve genetic data |
| [Your First API Query](tutorials/FIRST-API-QUERY.md) | 10 min | Use the Rust library to query the NSIP Search API end-to-end |
| [MCP Server Setup](tutorials/MCP-SERVER-SETUP.md) | 10 min | Configure and start the MCP server for AI assistant integration |
| [Interpreting Search Results](tutorials/INTERPRETING-RESULTS.md) | 10 min | Read and understand animal search results, EBV traits, and accuracy |

---

## How-To Guides

Problem-oriented guides for accomplishing specific tasks.

| Guide | Description |
|-------|-------------|
| [Configure Client](how-to/CONFIGURE-CLIENT.md) | Customize timeout, retries, and base URL for the HTTP client |
| [Compare Animals](how-to/COMPARE-ANIMALS.md) | Side-by-side genetic trait comparisons via CLI, library, or MCP |
| [Filter Search Results](how-to/FILTER-SEARCH-RESULTS.md) | Use SearchCriteria to filter by breed, gender, status, date, and trait ranges |
| [Use MCP Tools](how-to/USE-MCP-TOOLS.md) | Invoke the 13 MCP server tools from AI assistants |
| [MCP Tool Sets](how-to/MCP-TOOL-SETS.md) | Enable specific tool categories with `--tools` |
| [OAuth Authentication](how-to/OAUTH-AUTHENTICATION.md) | Secure the MCP HTTP transport with GitHub OAuth or PAT |
| [Telemetry](how-to/TELEMETRY.md) | Enable OpenTelemetry trace context in server logs |
| [Mating Recommendations](how-to/MATING-RECOMMENDATIONS.md) | Use analytics tools for breeding decisions |
| [Export JSON](how-to/EXPORT-JSON.md) | Export data as JSON using the `--json` flag or library serialization |
| [Batch Query Animals](how-to/BATCH-QUERY.md) | Query multiple animals concurrently with Tokio |
| [Scripting Integration](how-to/SCRIPTING-INTEGRATION.md) | Integrate nsip into shell scripts, CI pipelines, and automation workflows |

---

## Explanation

Understanding-oriented discussions of key concepts.

| Document | Description |
|----------|-------------|
| [Understanding EBVs](explanation/EBV-EXPLAINED.md) | What EBVs are, how they're calculated, accuracy, and selection indexes |
| [NSIP Data Model](explanation/NSIP-DATA-MODEL.md) | Program structure: breed groups, breeds, flocks, animals, and their relationships |
| [Genetic Evaluation](explanation/GENETIC-EVALUATION.md) | How BLUP works, pedigree and genomic data, and the evaluation pipeline |
| [Breed Groups and Traits](explanation/BREED-GROUPS-AND-TRAITS.md) | Understanding breed group categories and the 13 EBV trait abbreviations |
| [From Data to Decisions](explanation/DATA-TO-DECISIONS.md) | How NSIP API data connects to real-world breeding decisions |
| [MCP Security](explanation/MCP-SECURITY.md) | OAuth 2.1, PKCE, GitHub PAT authentication, and DNS rebinding protection |
| [Telemetry](explanation/TELEMETRY.md) | W3C trace context, distributed tracing, and the OpenTelemetry integration |
| [Dynamic Instructions](explanation/DYNAMIC-INSTRUCTIONS.md) | How server instructions adapt to enabled tool sets |

---

## Reference

Information-oriented technical descriptions.

| Document | Description |
|----------|-------------|
| [CLI Reference](reference/CLI.md) | Every subcommand, flag, and option for the `nsip` binary |
| [Library API](reference/LIBRARY-API.md) | `NsipClient` methods, `SearchCriteria` builder, and all model types |
| [MCP Tools](reference/MCP-TOOLS.md) | All 13 MCP server tools with parameters, return types, and examples |
| [Error Handling](reference/ERROR-HANDLING.md) | Complete `Error` enum reference with handling patterns |
| [Configuration](reference/CONFIGURATION.md) | Client builder options, defaults, retry behavior, and environment |
| [MCP Server Configuration](reference/MCP-SERVER-CONFIGURATION.md) | Transport, tool sets, OAuth, telemetry, and middleware configuration |
| [MCP Server API](MCP.md) | Full MCP server reference: tools, resources, prompts, and analytics |

---

## Template Adoption Guides

Guides for developers who just created a repository from this template.

| Guide | Description |
|-------|-------------|
| [Getting Started](template/GETTING-STARTED.md) | "Use this template" to first `cargo build` to first CI pass |
| [Configuration](template/CONFIGURATION.md) | Cargo.toml fields, placeholder replacement, feature flags, editor setup |
| [CI Workflows](template/CI-WORKFLOWS.md) | Every included workflow: triggers, secrets, how to enable/disable |
| [Customization](template/CUSTOMIZATION.md) | Add modules, remove examples, adjust lints, modify release targets |
| [GitHub Template Features](template/GITHUB-TEMPLATE-FEATURES.md) | What copies when using a template -- and what doesn't |
| [Copilot Jumpstart](template/COPILOT-JUMPSTART.md) | Prompts for automatic project scaffolding with GitHub Copilot |

## Operational Runbooks

Step-by-step procedures for ongoing project maintenance.

| Runbook | Description |
|---------|-------------|
| [Releasing](runbooks/RELEASING.md) | Version bump, tag, monitor workflows, verify artifacts |
| [Dependency Updates](runbooks/DEPENDENCY-UPDATES.md) | Dependabot policy, manual cargo-deny audit, handling advisories |
| [Security Response](runbooks/SECURITY-RESPONSE.md) | Vulnerability triage, fix, coordinated disclosure |
| [CI Troubleshooting](runbooks/CI-TROUBLESHOOTING.md) | Common CI failure patterns and fixes |

## Additional Reference

Detailed reference material organized by topic.

### Workflows

| Document | Description |
|----------|-------------|
| [CI Pipeline](workflows/CI.md) | Primary quality gate: format, lint, test, docs, MSRV, and coverage |
| [Release](workflows/RELEASE.md) | Multi-platform binary builds, changelog generation, and release creation |
| [Publish](workflows/PUBLISH.md) | Publish the `nsip` crate to crates.io |
| [Security Audit](workflows/SECURITY-AUDIT.md) | Daily `cargo audit` vulnerability scanning against the RustSec database |
| [SBOM](workflows/SBOM.md) | Software Bill of Materials generation |
| [Secrets Scan](workflows/SECRETS-SCAN.md) | Secret scanning with Gitleaks |
| [Container Scan](workflows/CONTAINER-SCAN.md) | Container image vulnerability scanning |
| [Changelog](workflows/CHANGELOG.md) | Auto-generate `CHANGELOG.md` on version tag |
| [Stale](workflows/STALE.md) | Stale issue and PR management |
| [Dependabot Auto-Merge](workflows/DEPENDABOT-AUTOMERGE.md) | Automatically merge patch and minor Dependabot updates |
| [Contributors](workflows/CONTRIBUTORS.md) | Generate `CONTRIBUTORS.md` from commit history |
| [Copilot Setup Steps](workflows/COPILOT-SETUP-STEPS.md) | Bootstrap environment for GitHub Copilot coding agents |
| [Docker](workflows/DOCKER.md) | Build and push Docker image to GHCR |
| [Signed Releases](workflows/SIGNED-RELEASES.md) | Sign release assets with Cosign and generate checksums |
| [SLSA Provenance](workflows/SLSA-PROVENANCE.md) | SLSA Level 3 provenance attestations for release binaries |

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
| [ADR-0003](adr/0003-adopt-diataxis-documentation-framework.md) | Adopt Diataxis Documentation Framework |

See [docs/adr/README.md](adr/README.md) for the full ADR process and workflow.
