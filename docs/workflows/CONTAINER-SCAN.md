# Container Vulnerability Scanning with Trivy

## Overview

Automated Docker container vulnerability scanning using [Trivy](https://github.com/aquasecurity/trivy).

**Workflow:** `.github/workflows/container-scan.yml`  
**Triggers:** Push, PR, Weekly schedule, Manual  
**Integration:** GitHub Security tab (SARIF upload)

## How It Works

Scans Docker images for:
- OS package vulnerabilities (CVEs)
- Application dependencies vulnerabilities  
- Misconfigurations
- Secrets in image layers

**Severity Levels:**
- CRITICAL
- HIGH
- MEDIUM
- LOW
- UNKNOWN

## Usage

### Local Scanning

```bash
# Install Trivy
brew install trivy
# or
curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin

# Build and scan image
docker build -t nsip:local .
trivy image nsip:local

# Scan specific severity
trivy image --severity HIGH,CRITICAL nsip:local

# Output formats
trivy image --format json nsip:local > scan.json
trivy image --format sarif nsip:local > scan.sarif
```

### CI Scanning

The workflow automatically:
1. Builds Docker image
2. Scans for vulnerabilities
3. Uploads SARIF to GitHub Security
4. Generates human-readable report
5. Uploads report as artifact

**View Results:**
- Security tab → Code scanning alerts
- Actions tab → Artifacts → trivy-scan-report

## Configuration

### Severity Threshold

Edit `.github/workflows/container-scan.yml`:

```yaml
- name: Run Trivy
  with:
    severity: CRITICAL,HIGH  # Only critical and high
```

### Ignore Unfixed

Skip vulnerabilities without fixes:

```yaml
- name: Run Trivy
  with:
    ignore-unfixed: true
```

### Trivy Configuration File

Create `.trivyignore`:

```
# Ignore specific CVEs
CVE-2021-12345

# Ignore by package
pkg:deb/debian/openssl@1.1.1
```

## Understanding Results

### SARIF Output (GitHub Security)

```json
{
  "results": [
    {
      "ruleId": "CVE-2021-12345",
      "level": "error",
      "message": {
        "text": "openssl: buffer overflow vulnerability"
      },
      "locations": [{
        "physicalLocation": {
          "artifactLocation": {
            "uri": "Dockerfile"
          }
        }
      }]
    }
  ]
}
```

### Table Output

```
Library      Vulnerability  Severity  Status  Installed  Fixed
-------      -------------  --------  ------  ---------  -----
openssl      CVE-2021-12345 CRITICAL  fixed   1.1.1k     1.1.1l
```

## Remediation

### Update Base Image

```dockerfile
# Before
FROM rust:1.92-slim

# After (with digest for immutability)
FROM rust:1.92-slim@sha256:abc123...
```

### Update Dependencies

```bash
# Update Cargo dependencies
cargo update
cargo audit

# Rebuild image
docker build -t nsip:patched .
trivy image nsip:patched
```

### Accept Risk

For false positives or accepted risks:

```
# .trivyignore
CVE-2021-12345  # Mitigated by network isolation
```

## Scheduled Scans

Weekly scans run automatically:

```yaml
schedule:
  - cron: "0 0 * * 0"  # Every Sunday at midnight
```

## Troubleshooting

### Scan Failures

```bash
# Update Trivy database
trivy image --download-db-only

# Clear cache
trivy image --clear-cache
```

### False Positives

Check vulnerability details:

```bash
trivy image --format json nsip:local | jq '.Results[].Vulnerabilities[] | select(.VulnerabilityID=="CVE-2021-12345")'
```

Add to `.trivyignore` if confirmed false positive.

### Performance

Scans can be slow. Optimize:

```yaml
# Scan only critical/high
severity: CRITICAL,HIGH

# Skip DB download
skip-db-update: true  # Use cache
```

## Links

- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Configuration Reference](https://aquasecurity.github.io/trivy/latest/docs/configuration/)
- [CVE Database](https://cve.mitre.org/)
- [GitHub Security Advisories](https://github.com/advisories)
