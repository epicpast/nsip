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
| [MCP Server API](MCP.md) | Full MCP server reference: tools, resources, prompts, and analytics |

---

## Template Adoption Guides

Guides for developers who just created a repository from this template.

| Guide | Description |
|-------|-------------|
| [Getting Started](template/GETTING-STARTED.md) | "Use this template" to first `cargo build` to first CI pass |
| [Configuration](template/CONFIGURATION.md) | Cargo.toml fields, placeholder replacement, feature flags, editor setup |
| [CI Workflows](template/CI-WORKFLOWS.md) | Every included workflow: triggers, secrets, how to enable/disable |
| [Agentic Workflows](workflows/AGENTIC-WORKFLOWS.md) | Autonomous AI agents: CI Doctor, Daily QA, Issue Triage, Q optimizer, Update Docs, Daily Documentation Review, Daily Repository Status |
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
| [Agentic Workflows](workflows/AGENTIC-WORKFLOWS.md) | Autonomous AI agents for CI/CD (CI Doctor, Daily QA, Issue Triage, Q optimizer, Update Docs, Daily Documentation Review, Daily Repository Status) |
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
| [ADR-0003](adr/0003-adopt-diataxis-documentation-framework.md) | Adopt Diataxis Documentation Framework |

See [docs/adr/README.md](adr/README.md) for the full ADR process and workflow.
