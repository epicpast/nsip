# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Documentation

- Update CHANGELOG.md for v0.7.1

### Miscellaneous

- Bump version to 0.7.2 (#301)

### Ci

- **docker**: Static musl binary on distroless/static (root-cause CVE fix) (#300)

## [0.7.1] - 2026-06-17

### Documentation

- Update CHANGELOG.md for v0.7.0
- **release skill**: Don't hand-edit CHANGELOG — changelog.yml owns it (#293)

### Miscellaneous

- Bump version to 0.7.1 (#296)

### Ci

- **docker**: Fix container attestation chain (ignore-unfixed + SBOM upload) (#294)
- **docker**: Native per-arch builds + manifest (drop QEMU emulation) (#295)

## [0.7.0] - 2026-06-17

### Documentation

- Update CHANGELOG.md for v0.6.2

### Miscellaneous

- **deps**: Bump uuid from 1.23.2 to 1.23.3 (#278)
- **deps**: Bump chrono from 0.4.44 to 0.4.45 (#279)
- **deps**: Bump tower-http from 0.6.11 to 0.7.0 (#280)
- **deps**: Bump the github-actions group with 2 updates (#282)
- Gh aw upgrade (recompile workflows to v0.79.8)
- **deps**: Bump gitleaks/gitleaks-action (#285)
- **deps**: Bump docker/setup-buildx-action (#284)
- **deps**: Bump docker/login-action (#286)
- **deps**: Bump regex from 1.12.3 to 1.12.4 (#281)
- Add /release orchestration skill adapted for nsip (#288)
- Bump version to 0.7.0 (#289)

### Security

- Add gh-attested merge-time quality-gates caller

### Ci

- Graft gh-attested quality-gate seam into release pipeline
- Make actionlint pass repo-wide
- Add gh-attested container-scan attestation, retire container-scan.yml
- Wire centralized image sign + fail-closed verify (full gh-attested parity)
- Fix OSV + CodeQL gate failures on first PR run

## [0.6.2] - 2026-06-12

### Documentation

- Update CHANGELOG.md for v0.6.0
- Update CHANGELOG.md for v0.6.1
- **claude**: Mandate Release PR workflow for releases; forbid hand-rolled release PRs

### Fixed

- **ci**: Base changelog branch on develop and auto-merge it
- Address PR review feedback

### Miscellaneous

- **deps**: Bump the github-actions group with 3 updates (#267)
- **deps**: Bump docker/setup-buildx-action
- **deps**: Bump gitleaks/gitleaks-action
- **deps**: Bump docker/login-action
- **deps**: Bump codecov/codecov-action from 6.0.1 to 7.0.0
- **deps**: Bump the github-actions group with 2 updates (#271)
- **deps**: Bump docker/login-action
- **deps**: Adopt actions/checkout v6.0.3 in rewritten release workflows
- Bump version to 0.6.2

### Ci

- Align release pipeline with rlm-rs attested delivery

## [0.6.1] - 2026-06-02

### Fixed

- **docker**: Copy build.rs into the build stage

### Miscellaneous

- **release**: Bump version to 0.6.1

## [0.6.0] - 2026-06-02

### Added

- **errors**: RFC 9457 dual-consumer error envelope (CDC remediation)
- **errors**: Per-operation error taxonomy + envelope all MCP paths
- **errors**: Dedicated mcp/invalid-cursor problem type
- Configurable error type-URI base and per-error slugs via Cargo.toml

### Documentation

- Correct workflow trigger accuracy from PR review
- Correct release trigger accuracy in RELEASE.md
- Use annotated tag and dedupe runbook link in RELEASE.md

### Fixed

- **ci**: Resolve MSRV -D warnings failure and RUSTSEC-2026-0037
- Harden error-handling edges found in self-review
- Address Copilot review feedback on PR #259

### Miscellaneous

- Remove editor, devcontainer, snap, certs, and refactor config
- Prune disabled workflows and reconcile docs
- **release**: Bump version to 0.6.0

### Ci

- Automate post-release back-merge and quiet docker attestation

## [0.5.1] - 2026-06-01

### Miscellaneous

- **deps**: Bump clap from 4.6.0 to 4.6.1 (#218)
- **deps**: Bump tokio from 1.52.1 to 1.52.2 in the async-runtime group (#216)
- **deps**: Bump rmcp from 1.5.0 to 1.6.0 (#217)
- **deps**: Bump axum from 0.8.8 to 0.8.9 (#219)
- **deps**: Bump the github-actions group with 2 updates (#220)
- **deps**: Bump tokio from 1.52.2 to 1.52.3 in the async-runtime group (#225)
- **deps**: Bump opentelemetry-otlp from 0.31.1 to 0.32.0 (#227)
- **deps**: Bump assert_cmd from 2.2.1 to 2.2.2 (#230)
- **deps**: Bump tower-http from 0.6.8 to 0.6.10 (#231)
- **deps**: Bump clap_complete from 4.6.3 to 4.6.5 (#226)
- **deps**: Bump jsonwebtoken from 10.3.0 to 10.4.0 (#232)
- **deps**: Bump the github-actions group with 4 updates (#233)
- **deps**: Bump tower-http from 0.6.10 to 0.6.11 (#234)
- **deps**: Bump rmcp from 1.6.0 to 1.7.0 (#235)
- **deps**: Bump the github-actions group with 3 updates (#237)
- **deps**: Bump opentelemetry stack to consistent 0.32 / 0.33 (#239)
- **deps**: Bump softprops/action-gh-release from 2.6.1 to 3.0.0 (#222)
- **deps**: Bump actions/configure-pages from 5.0.0 to 6.0.0 (#224)
- **deps**: Bump actions/github-script from 8.0.0 to 9.0.0 (#221)
- **deps**: Bump codecov/codecov-action from 5.5.3 to 6.0.1 (#238)
- **deps**: Bump serde_json in the serde-ecosystem group (#240)
- **deps**: Bump the github-actions group with 6 updates (#241)
- **deps**: Bump actions/upload-pages-artifact from 4.0.0 to 5.0.0 (#242)
- **deps**: Bump dtolnay/rust-toolchain (#243)
- **deps**: Bump actions/deploy-pages from 4.0.5 to 5.0.0 (#244)
- **deps**: Bump dependabot/fetch-metadata from 2.5.0 to 3.1.0 (#245)
- **deps**: Bump uuid from 1.23.1 to 1.23.2 (#247)
- **deps**: Bump opentelemetry_sdk from 0.32.0 to 0.32.1 (#248)
- **deps**: Bump the github-actions group with 3 updates (#249)
- **deps**: Bump zircote/adrscope (#250)
- Bump version to 0.5.1 (#252)

### Ci

- Adopt develop-based branching model (#246)
- Pin release builds to windows-2022 (#251)

## [0.5.0] - 2026-04-30

### Documentation

- Update CHANGELOG.md for v0.4.0
- Fix NSIP_AUTH_ALLOWED_USERS gap and stale OTLP env var comment
- Fix NSIP_AUTH_ISSUER default value and add optional OAuth vars to CONFIGURATION.md
- Fix incorrect method names in LIBRARY-API.md analytics examples
- Fix serve_stdio signature and add serve_http + with_tool_sets to LIBRARY-API.md
- Fix stale version and MCP protocol references
- Fix MCP-TOOLS protocol version and add Diátaxis frontmatter
- Fix serve_stdio signature and MCP tool names in tutorials
- Add Diátaxis frontmatter to remaining 32 documentation files
- Fix just check description to include coverage step
- Document agentics-maintenance workflow in AGENTIC-WORKFLOWS.md
- Add workflow reference docs for CI, Release, Security Audit, and ADR workflows
- Add workflow reference docs for 18 undocumented workflows

### Fixed

- **ci**: Fix dependabot automerge — use pull_request_target and add approval

### Miscellaneous

- **deps**: Bump tracing-subscriber from 0.3.22 to 0.3.23 (#179)
- **deps**: Bump clap_complete from 4.5.66 to 4.6.0 (#180)
- **deps**: Bump rmcp from 1.1.1 to 1.2.0 (#181)
- **deps**: Bump the github-actions group across 1 directory with 6 updates (#190)
- **deps**: Bump clap_mangen from 0.2.31 to 0.2.33 (#184)
- **deps**: Bump clap from 4.5.60 to 4.6.0 (#182)
- **deps**: Bump actions/download-artifact from 4.2.1 to 8.0.1 (#188)
- **deps**: Bump tempfile from 3.26.0 to 3.27.0 (#185)
- **deps**: Bump zircote/mcp-bundle (#187)
- **security**: Bump rustls-webpki and rand to clear advisories (#214)
- **deps**: Bump proptest (#207)
- **deps**: Bump clap_complete from 4.6.0 to 4.6.3 (#212)
- **deps**: Bump uuid from 1.22.0 to 1.23.1 (#210)
- **deps**: Bump clap_mangen from 0.2.33 to 0.3.0 (#198)
- **deps**: Bump opentelemetry-otlp from 0.31.0 to 0.31.1 (#191)
- **deps**: Bump sha2 from 0.10.9 to 0.11.0 (#195)
- **deps**: Bump assert_cmd from 2.1.2 to 2.2.1 (#209)
- **deps**: Bump the github-actions group across 1 directory with 9 updates (#213)
- **deps**: Bump zircote/mcp-bundle (#193)
- **deps**: Bump tokio from 1.50.0 to 1.52.1 in the async-runtime group (#208)
- **deps**: Bump rmcp from 1.2.0 to 1.5.0 (#211)
- **release**: Prepare v0.5.0 (#215)

### Ci

- **gh-aw**: Recompile all workflows with gh-aw v0.56.2
- **gh-aw**: Add agentics maintenance workflow
- Replace inline mcpb packaging with zircote/mcp-bundle action
- Use full commit SHA for mcp-bundle action pin
- Use date+sha nightly bundle name (nsip-nightly-YYYYMMDD-SHA.mcpb)

## [0.4.0] - 2026-03-09

### Fixed

- **ci**: Repair three release-triggered workflow failures
- Use Rng::fill instead of Rng::random for rand 0.9/0.10 compat
- Enable rust_crypto backend for jsonwebtoken 10, ignore RSA advisory
- **ci**: Ignore RUSTSEC-2023-0071 in cargo audit (RSA not used)
- **ci**: Add daily full audit job to track when ignored advisories get fixes

### Miscellaneous

- **deps**: Bump tokio from 1.49.0 to 1.50.0 in the async-runtime group (#148)
- **deps**: Bump rmcp from 1.1.0 to 1.1.1 (#150)
- **deps**: Bump chrono from 0.4.43 to 0.4.44 (#149)
- **deps**: Bump rand from 0.9.2 to 0.10.0 (#152)
- **deps**: Bump the github-actions group with 4 updates (#153)
- **deps**: Bump jsonwebtoken from 9.3.1 to 10.3.0
- **deps**: Bump actions/upload-artifact from 4.6.2 to 7.0.0
- **deps**: Bump docker/build-push-action from 6.19.2 to 7.0.0
- **deps**: Bump docker/metadata-action from 5.10.0 to 6.0.0
- **release**: Bump version to 0.4.0

## [0.4.0-rc1] - 2026-03-09

### Added

- **mcp**: Upgrade to MCP protocol 2025-06-18 with tool sets, OAuth, and telemetry (#143)

### Documentation

- Update CHANGELOG.md for v0.3.3
- Adopt Diátaxis framework for user documentation
- Add 16 Diátaxis documentation files
- Rewrite existing docs with NSIP research accuracy
- Reference structured-MADR (SMADR) instead of MADR
- Fix broken SearchCriteria reference link
- Fix broken SearchCriteria reference link (#77)
- Add mcp module reference to LIBRARY-API.md
- Add TraitDefinition and ebv_glossary to LIBRARY-API reference
- Add comprehensive agentic workflows documentation
- Apply review feedback to AGENTIC-WORKFLOWS.md and README.md

### Fixed

- **ci**: Enable persist-credentials in update-docs workflow
- **ci**: Use correct adrscope action input names
- **ci**: Add structured MADR frontmatter to ADR files
- **ci**: Use full structured-MADR frontmatter in ADRs
- **ci**: Use workflow_run trigger for homebrew packaging
- **ci**: Apply review feedback to package-homebrew.yml
- **ci**: Unblock daily-qa network access and add issue input (#115)

### Miscellaneous

- **deps**: Bump clap from 4.5.58 to 4.5.60 (#101)
- **deps**: Bump rmcp from 0.15.0 to 0.16.0 (#102)
- **deps**: Bump the github-actions group with 4 updates (#103)
- **deps**: Bump actions/setup-node from 4.2.0 to 6.2.0 (#104)
- **deps**: Bump zircote/adrscope (#107)
- **deps**: Bump peter-evans/dockerhub-description from 4.0.0 to 5.0.0 (#105)
- **deps**: Update dtolnay/rust-toolchain requirement to efa25f7f19611383d5b0ccf2d1c8914531636bf9 (#106)
- **deps**: Bump tempfile from 3.25.0 to 3.26.0 (#123)
- **deps**: Bump rmcp from 0.16.0 to 0.17.0 (#124)
- **deps**: Bump the github-actions group with 3 updates (#125)
- **deps**: Bump actions/attest-build-provenance from 3.2.0 to 4.1.0 (#128)
- **deps**: Bump zircote/adrscope (#127)
- **deps**: Bump actions/download-artifact from 6.0.0 to 8.0.0 (#126)
- **deps**: Bump aquasecurity/trivy-action (#129)
- Ignore .fastembed_cache directory

### Performance

- **.claude**: Sync agents and spec-orchestrator from rust-template

## [0.3.3] - 2026-02-16

### Documentation

- Update CHANGELOG.md for v0.3.3-rc.3

### Fixed

- **ci**: Rename rust-template to nsip in linux and sbom workflows
- **ci**: Correct invalid action SHAs from copilot agent

### Miscellaneous

- **ci**: Remove ci-doctor agentic workflow files
- **ci**: Update agentic workflow lock files
- **ci**: Add engine ID to q workflow lock file
- **deps**: Bump clap_complete from 4.5.65 to 4.5.66 (#19)
- **deps**: Bump clap from 4.5.57 to 4.5.58 (#17)
- **deps**: Bump tempfile from 3.24.0 to 3.25.0 (#18)
- **deps**: Bump rmcp from 0.14.0 to 0.15.0 (#21)
- **deps**: Bump predicates from 3.1.3 to 3.1.4 (#20)
- **deps**: Bump the github-actions group with 3 updates (#22)
- Remove copilot agent working files
- **deps**: Bump actions/attest-build-provenance
- **deps**: Bump actions/download-artifact from 6.0.0 to 7.0.0
- **deps**: Bump actions/upload-pages-artifact from 3.0.1 to 4.0.0
- **ci**: Remove CodeQL workflow (unsupported for Rust)
- **deps**: Bump actions/checkout from 4.2.2 to 6.0.2
- **release**: Bump version to 0.3.3

## [0.3.3-rc.3] - 2026-02-14

### Added

- **homebrew**: Add source formula alongside binary formula
- **release**: Attach attestation bundles to release assets
- **mcpb**: Add MCPB manifest, ignore file, and signing cert
- **release**: Add MCPB bundle packaging to release pipeline

### Documentation

- Update CHANGELOG.md for v0.3.2
- Update CHANGELOG.md for v0.3.3-rc.1
- Update CHANGELOG.md for v0.3.3-rc.2

### Fixed

- **release**: Use PAT for release to trigger downstream workflows
- **homebrew**: Use pre-built binaries with completions and man pages
- **homebrew**: Rename platform binary to nsip during install
- **mcpb**: Correct manifest schema and mcpb CLI usage
- **mcpb**: Regenerate signing cert as RSA-4096
- **mcpb**: Remove signing (mcpb sign v2.1.2 corrupts ZIP)

### Miscellaneous

- Ignore *.local.* files

## [0.3.2] - 2026-02-08

### Documentation

- Update CHANGELOG.md for v0.3.1
- Update CHANGELOG.md for v0.3.2

### Fixed

- Update docs, workflows, and metadata for v0.3.2
- **ci**: Bump to 0.3.2, add environment for secrets, fix yamllint

### Miscellaneous

- **release**: Bump version to 0.3.2

## [0.3.1] - 2026-02-08

### Documentation

- Update CHANGELOG.md for v0.3.0

### Fixed

- Docker build shipping dummy binary, add release provenance

## [0.3.0] - 2026-02-08

### Added

- NSIP Search API client with rmcp MCP server
- Rewrite NSIP client from Python API parity with full endpoint coverage
- Add human-readable formatting module
- Farmer-friendly CLI with compare, completions, and man pages
- Add MCP server with 13 tools, resources, and guided prompts

### Documentation

- Update README examples to match rewritten API
- Add LLM guide templates for MCP server consumers
- Add MCP server pointers to root instruction files

### Fixed

- **ci**: Rename release binary artifacts from rust_template to nsip
- Correct cliff.toml TOML syntax for commit_parsers

### Miscellaneous

- Initialize from rust-template for zircote/nsip
- **deps**: Bump the github-actions group with 7 updates (#2)
- **deps**: Bump sigstore/cosign-installer from 3.7.0 to 4.0.0 (#7)
- **deps**: Bump actions/stale from 9.0.0 to 10.1.1 (#3)
- **deps**: Bump actions/cache from 4.2.0 to 5.0.3 (#5)
- **deps**: Bump github/codeql-action from 3 to 4 (#6)
- **deps**: Bump actions/github-script from 7.0.1 to 8.0.0 (#4)

[unreleased]: https://github.com/zircote/nsip/compare/v0.7.1..HEAD
[0.7.1]: https://github.com/zircote/nsip/compare/v0.7.0..v0.7.1
[0.7.0]: https://github.com/zircote/nsip/compare/v0.6.2..v0.7.0
[0.6.2]: https://github.com/zircote/nsip/compare/v0.6.1..v0.6.2
[0.6.1]: https://github.com/zircote/nsip/compare/v0.6.0..v0.6.1
[0.6.0]: https://github.com/zircote/nsip/compare/v0.5.1..v0.6.0
[0.5.1]: https://github.com/zircote/nsip/compare/v0.5.0..v0.5.1
[0.5.0]: https://github.com/zircote/nsip/compare/v0.4.0..v0.5.0
[0.4.0]: https://github.com/zircote/nsip/compare/v0.4.0-rc1..v0.4.0
[0.4.0-rc1]: https://github.com/zircote/nsip/compare/v0.3.3..v0.4.0-rc1
[0.3.3]: https://github.com/zircote/nsip/compare/v0.3.3-rc.3..v0.3.3
[0.3.3-rc.3]: https://github.com/zircote/nsip/compare/v0.3.2..v0.3.3-rc.3
[0.3.2]: https://github.com/zircote/nsip/compare/v0.3.1..v0.3.2
[0.3.1]: https://github.com/zircote/nsip/compare/v0.3.0..v0.3.1
[0.3.0]: https://github.com/zircote/nsip/tree/v0.3.0

<!-- generated by git-cliff -->
