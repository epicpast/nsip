# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-03-09

### Added

- **mcp**: Upgrade to MCP protocol 2025-06-18 with full specification alignment (#143)
  - Dynamic tool sets: `--tools search,breed` to expose only specific tool categories
  - OAuth 2.1 + PKCE authentication with GitHub identity provider (`--auth`)
  - Personal Access Token (PAT) bearer auth for simplified HTTP access
  - Feature-gated OpenTelemetry tracing (`--features telemetry`)
  - Server identity (`nsip-mcp`) and `logging/setLevel` handler
  - CORS headers for `mcp-protocol-version` and `mcp-session-id` (MCP Inspector compatible)
  - Friendly error messages for `trait_ranges` with invalid breed IDs
- **docs**: Adopt Diátaxis documentation framework with 16 new guides
  - Tutorials, how-to guides, explanations, and reference documents
  - Rewritten with NSIP research accuracy and domain expertise
  - Library API reference: `TraitDefinition`, `ebv_glossary`, MCP module docs
  - Comprehensive agentic workflows documentation
- **ci**: Daily full security audit job to surface when ignored advisories get fixes

### Security

- **jsonwebtoken**: Upgrade from v9 to v10 with `rust_crypto` backend (#151)
- **rand**: Upgrade from v0.9 to v0.10, replace deprecated `Rng::random` API
- **ci**: Add `RUSTSEC-2023-0071` ignore with tracking — RSA timing side-channel
  in transitive dep; NSIP only uses HMAC-SHA256, never RSA
- **transport**: Tighten CORS to localhost origins, fix IPv6 bind
- **oauth**: Bearer auth middleware with JWT validation and PAT cache with TTL

### Fixed

- **mcp**: Wire elicitation support, fix protocol stubs — 14 audit findings resolved
- **mcp**: Server declares `nsip-mcp` identity instead of default `rmcp`
- **mcp**: Implement `logging/setLevel` handler (was declared but missing)
- **mcp**: CORS allows `mcp-protocol-version` header for MCP Inspector compatibility
- **mcp**: `trait_ranges` returns friendly message instead of raw API error on HTTP 400
- **ci**: Repair three release-triggered workflow failures (signed-releases race,
  SBOM duplicate asset, changelog branch protection)
- **ci**: Release workflow auto-detects pre-release tags (alpha/beta/rc)
- **ci**: Use `workflow_run` trigger for homebrew packaging and signed releases
- **ci**: Unblock daily-qa network access and add issue input (#115)

### Documentation

- Adopt Diátaxis framework for user documentation
- Add 16 Diátaxis documentation files
- Rewrite existing docs with NSIP research accuracy
- Reference structured-MADR (SMADR) instead of MADR
- Fix broken SearchCriteria reference link (#77)
- Add MCP module reference to LIBRARY-API.md
- Add TraitDefinition and ebv_glossary to LIBRARY-API reference
- Add comprehensive agentic workflows documentation

### Miscellaneous

- **deps**: Bump rmcp from 0.15.0 to 1.1.1
- **deps**: Bump jsonwebtoken from 9.3.1 to 10.3.0
- **deps**: Bump rand from 0.9.2 to 0.10.0
- **deps**: Bump chrono from 0.4.43 to 0.4.44
- **deps**: Bump tokio from 1.49.0 to 1.50.0
- **deps**: Bump clap from 4.5.58 to 4.5.60
- **deps**: Bump tempfile from 3.25.0 to 3.26.0
- **deps**: Bump docker/build-push-action from 6.19.2 to 7.0.0
- **deps**: Bump docker/metadata-action from 5.10.0 to 6.0.0
- **deps**: Bump actions/upload-artifact from 4.6.2 to 7.0.0
- **deps**: Bump actions/download-artifact from 6.0.0 to 8.0.0
- **deps**: Bump actions/attest-build-provenance from 3.2.0 to 4.1.0

[0.4.0]: https://github.com/zircote/nsip/compare/v0.3.3...v0.4.0

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

<!-- generated by git-cliff -->
