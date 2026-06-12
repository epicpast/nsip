---
diataxis_type: reference
---
# Software Bill of Materials (SBOM)

## Overview

Automated generation of Software Bill of Materials in SPDX format for supply chain transparency and compliance.

**Tools:** `anchore/sbom-action` (Syft — release pipeline), `cargo-sbom` (on-demand)  
**Formats:** CycloneDX JSON (release), SPDX 2.3 JSON (on-demand)  
**Release artifact filename:** `nsip-<VERSION>-sbom.cdx.json` (e.g. `nsip-0.6.0-sbom.cdx.json`)

## SBOM Generation Paths

An SBOM is produced through **two** distinct paths with different purposes:

| Path | Workflow / Job | Trigger | Attaches to release? | Attestation |
|---|---|---|---|---|
| Release pipeline | `release.yml` → `sbom` job | Push of a `v*.*.*` tag (or dispatch dry run) | **Yes** — uploads `nsip-<VERSION>-sbom.cdx.json` | **Yes** — `actions/attest-sbom` binds **every** release artifact to this SBOM; verified fail-closed before publication |
| On-demand | `sbom.yml` | Manual `workflow_dispatch` only | **No** — workflow run artifact only (90-day retention) | No |

The **release pipeline is the single authoritative source** of the SBOM on a
GitHub Release. The standalone `sbom.yml` is for ad-hoc SPDX inspection
(generate an SBOM for the current `main` without cutting a release); it does
**not** attach to releases, so it cannot clobber or invalidate the attested
release SBOM.

## What is an SBOM?

A machine-readable inventory of:
- All dependencies (direct and transitive)
- License information
- Package versions
- Supplier information

**Use cases:**
- Supply chain security (EO 14028 compliance)
- Vulnerability tracking
- License compliance
- Dependency auditing

## How It Works

On every release (the `release.yml` → `sbom` job, on `v*.*.*` tag push):
1. Generates a CycloneDX SBOM from the source tree with Syft
   (`anchore/sbom-action`)
2. Binds every release artifact (binaries, archives, MCPB bundle) to the
   SBOM with `actions/attest-sbom`
3. The `verify` job re-verifies the SBOM attestation on every artifact
   before the release publishes
4. Uploads `nsip-<VERSION>-sbom.cdx.json` to the GitHub release

Consumers verify with:

```bash
gh attestation verify nsip-X.Y.Z-linux-amd64 --repo zircote/nsip \
  --predicate-type https://cyclonedx.org/bom
```

## Usage

### Generate Locally

```bash
# Install cargo-sbom
cargo install cargo-sbom

# Generate SBOM (SPDX format)
cargo sbom --output-format spdx_json_2_3 > sbom.json

# View SBOM
cat sbom.json | jq '.packages[] | {name, version, licenseConcluded}'
```

### Access from Release

```bash
# Download from GitHub release (replace vX.Y.Z with the release tag)
wget https://github.com/zircote/nsip/releases/download/vX.Y.Z/nsip-X.Y.Z-sbom.cdx.json

# Analyze with SBOM tools (CycloneDX)
cyclonedx validate --input-file nsip-X.Y.Z-sbom.cdx.json
```

## Configuration

The workflow uses default configuration. To customize:

```bash
# Different output format
cargo sbom --output-format cyclonedx_json_1_4

# Include build dependencies
cargo sbom --cargo-features all-features
```

## SBOM Contents

The generated SBOM includes:

```json
{
  "SPDXID": "SPDXRef-DOCUMENT",
  "packages": [
    {
      "name": "nsip",
      "versionInfo": "X.Y.Z",
      "licenseConcluded": "MIT",
      "supplier": "Organization: zircote"
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-nsip",
      "relationshipType": "DEPENDS_ON",
      "relatedSpdxElement": "SPDXRef-dependency"
    }
  ]
}
```

## Compliance

### Executive Order 14028

The SBOM meets requirements for:
- Machine-readable format (SPDX)
- Dependency enumeration
- License identification
- Supplier information

### NIST Guidelines

Complies with NIST SP 800-161r1 for supply chain risk management.

## Troubleshooting

### Missing Dependencies

If dependencies are missing:

```bash
# Ensure Cargo.lock is up to date
cargo update
cargo sbom --output-format spdx_json_2_3
```

### License Issues

Unknown licenses appear as `NOASSERTION`. To fix:

```toml
# Cargo.toml
[package]
license = "MIT"

[dependencies]
unlicensed-crate = { version = "1.0", license = "MIT" }
```

### Format Errors

Validate SBOM:

```bash
# Release SBOM (CycloneDX)
cyclonedx validate --input-file nsip-X.Y.Z-sbom.cdx.json

# On-demand SPDX SBOM (sbom.yml artifact)
pip install spdx-tools
spdx-tools validate nsip-X.Y.Z-sbom-spdx.json
```

## Links

- [cargo-sbom Documentation](https://github.com/psastras/sbom-rs)
- [SPDX Specification](https://spdx.github.io/spdx-spec/)
- [NTIA Minimum Elements](https://www.ntia.gov/files/ntia/publications/sbom_minimum_elements_report.pdf)
- [Executive Order 14028](https://www.whitehouse.gov/briefing-room/presidential-actions/2021/05/12/executive-order-on-improving-the-nations-cybersecurity/)
