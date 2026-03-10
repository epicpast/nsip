---
diataxis_type: reference
---
# Software Bill of Materials (SBOM)

## Overview

Automated generation of Software Bill of Materials in SPDX format for supply chain transparency and compliance.

**Workflow:** `.github/workflows/sbom.yml`  
**Tool:** `cargo-sbom`  
**Format:** SPDX 2.3 JSON  
**Triggers:** Version tags, releases

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

On every release:
1. Generates SBOM from `Cargo.lock`
2. Outputs SPDX 2.3 JSON format
3. Uploads as build artifact (90 days)
4. Attaches to GitHub release

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
# Download from GitHub release
wget https://github.com/zircote/nsip/releases/download/v0.1.0/sbom-spdx.json

# Analyze with SBOM tools
sbom-tool validate sbom-spdx.json
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
      "versionInfo": "0.1.0",
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
# Install SPDX validator
pip install spdx-tools

# Validate
spdx-tools validate sbom-spdx.json
```

## Links

- [cargo-sbom Documentation](https://github.com/psastras/sbom-rs)
- [SPDX Specification](https://spdx.github.io/spdx-spec/)
- [NTIA Minimum Elements](https://www.ntia.gov/files/ntia/publications/sbom_minimum_elements_report.pdf)
- [Executive Order 14028](https://www.whitehouse.gov/briefing-room/presidential-actions/2021/05/12/executive-order-on-improving-the-nations-cybersecurity/)
