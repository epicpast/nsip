---
diataxis_type: reference
---
# Signed Releases & SLSA Provenance

## Overview

Cryptographically sign release artifacts and generate SLSA provenance for supply chain security.

**Workflows:**
- `.github/workflows/signed-releases.yml` - Cosign signatures
- `.github/workflows/slsa-provenance.yml` - SLSA Level 3 provenance

## Why Sign Releases?

- **Authenticity**: Verify artifacts come from you
- **Integrity**: Detect tampering or corruption
- **Non-repudiation**: Prove you created the release
- **Compliance**: Meet supply chain security requirements

## Cosign Signatures

### How It Works

1. **Release published** - Workflow triggers
2. **Download assets** - Get all release files
3. **Sign with Cosign** - Keyless signing via Sigstore
4. **Upload signatures** - Attach `.sig` files to release
5. **Generate checksums** - SHA256/SHA512 sums
6. **Sign checksums** - Verify integrity chain

### Verifying Signatures

**Install Cosign:**
```bash
# Homebrew
brew install cosign

# Download binary
wget https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
```

**Verify Asset:**
```bash
# Download release and signature
wget https://github.com/USER/REPO/releases/download/v0.1.0/nsip
wget https://github.com/USER/REPO/releases/download/v0.1.0/nsip.sig

# Verify signature
cosign verify-blob \
  --signature nsip.sig \
  --certificate-identity-regexp=".*" \
  --certificate-oidc-issuer-regexp=".*" \
  nsip
```

**Verify Checksums:**
```bash
# Download checksums
wget https://github.com/USER/REPO/releases/download/v0.1.0/SHA256SUMS
wget https://github.com/USER/REPO/releases/download/v0.1.0/SHA256SUMS.sig

# Verify checksum signature
cosign verify-blob \
  --signature SHA256SUMS.sig \
  --certificate-identity-regexp=".*" \
  --certificate-oidc-issuer-regexp=".*" \
  SHA256SUMS

# Verify file against checksum
sha256sum --check SHA256SUMS
```

### Keyless Signing

Cosign uses **keyless signing** via Sigstore:
- No private keys to manage
- Uses OIDC identity (GitHub Actions)
- Transparency log (Rekor) for auditability
- Certificate from Fulcio CA

**Benefits:**
- No key rotation needed
- No key compromise risk
- Publicly verifiable
- Auditable via transparency log

## SLSA Provenance

### What is SLSA?

**SLSA** (Supply chain Levels for Software Artifacts) is a framework for ensuring software supply chain integrity.

**Levels:**
- **SLSA 1**: Documentation of build process
- **SLSA 2**: Version control + build service
- **SLSA 3**: Hardened builds (non-falsifiable provenance)
- **SLSA 4**: Hermetic, reproducible builds

### Generate Provenance

The workflow automatically generates **SLSA Level 3** provenance:

```json
{
  "_type": "https://in-toto.io/Statement/v0.1",
  "subject": [
    {
      "name": "nsip",
      "digest": {
        "sha256": "abc123..."
      }
    }
  ],
  "predicateType": "https://slsa.dev/provenance/v0.2",
  "predicate": {
    "builder": {
      "id": "https://github.com/slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.0.0"
    },
    "buildType": "https://github.com/slsa-framework/slsa-github-generator/generic@v1",
    "invocation": {
      "configSource": {
        "uri": "git+https://github.com/USER/REPO@refs/tags/v0.1.0",
        "digest": {
          "sha1": "def456..."
        }
      }
    },
    "metadata": {
      "buildStartedOn": "2026-01-01T00:00:00Z",
      "buildFinishedOn": "2026-01-01T00:05:00Z"
    },
    "materials": [
      {
        "uri": "git+https://github.com/USER/REPO@refs/tags/v0.1.0",
        "digest": {
          "sha1": "def456..."
        }
      }
    ]
  }
}
```

### Verify Provenance

**Install SLSA Verifier:**
```bash
wget https://github.com/slsa-framework/slsa-verifier/releases/download/v2.5.1/slsa-verifier-linux-amd64
chmod +x slsa-verifier-linux-amd64
sudo mv slsa-verifier-linux-amd64 /usr/local/bin/slsa-verifier
```

**Verify Artifact:**
```bash
# Download binary and provenance
wget https://github.com/USER/REPO/releases/download/v0.1.0/nsip
wget https://github.com/USER/REPO/releases/download/v0.1.0/nsip.intoto.jsonl

# Verify provenance
slsa-verifier verify-artifact \
  --provenance-path nsip.intoto.jsonl \
  --source-uri github.com/USER/REPO \
  nsip
```

**Output:**
```
✓ Verified SLSA provenance
  Source: github.com/USER/REPO
  Builder: https://github.com/slsa-framework/slsa-github-generator
  Build Level: SLSA 3
```

## Integration with Package Managers

### Homebrew

```ruby
def install
  system "cargo", "install", *std_cargo_args

  # Download and verify signature
  signature_url = "#{url}.sig"
  resource("signature").stage do
    system "cosign", "verify-blob",
           "--signature", "nsip.sig",
           "--certificate-identity-regexp", ".*",
           "--certificate-oidc-issuer-regexp", ".*",
           bin/"nsip"
  end
end
```

### Docker

```dockerfile
# Verify binary before adding to image
RUN wget https://github.com/USER/REPO/releases/download/v0.1.0/nsip && \
    wget https://github.com/USER/REPO/releases/download/v0.1.0/nsip.sig && \
    cosign verify-blob \
      --signature nsip.sig \
      --certificate-identity-regexp=".*" \
      --certificate-oidc-issuer-regexp=".*" \
      nsip
```

## Advanced Configuration

### Custom Signing Keys

For organizations with existing PKI:

```yaml
- name: Import GPG key
  run: echo "${{ secrets.GPG_PRIVATE_KEY }}" | gpg --import

- name: Sign with GPG
  run: |
    for file in *; do
      gpg --detach-sign --armor "$file"
    done
```

**Verify GPG:**
```bash
gpg --verify nsip.asc nsip
```

### Multiple Signatures

```yaml
- name: Sign with multiple methods
  run: |
    # Cosign (keyless)
    cosign sign-blob --yes nsip > nsip.cosign.sig

    # GPG (traditional)
    gpg --detach-sign --armor nsip

    # Minisign (simple)
    minisign -Sm nsip
```

## Security Best Practices

### 1. Minimize Attack Surface

- **Use official actions** with commit SHA pinning
- **Limit permissions** to minimum required
- **Avoid secrets** in logs or artifacts

### 2. Verify Everything

- **Verify dependencies** before building
- **Verify build environment** is expected
- **Verify artifacts** match source

### 3. Audit Trail

- **Enable Rekor** transparency log
- **Archive provenance** long-term
- **Monitor certificates** for unexpected issuance

### 4. User Education

- **Document verification** in README
- **Provide examples** of verification
- **Link to tools** (cosign, slsa-verifier)

## Troubleshooting

### Cosign Verification Fails

```bash
# Check certificate details
cosign verify-blob \
  --signature nsip.sig \
  --certificate-identity-regexp=".*" \
  --certificate-oidc-issuer-regexp=".*" \
  --debug \
  nsip
```

**Common issues:**
- Expired certificate (valid for 10 minutes during signing)
- Wrong issuer (should be `https://token.actions.githubusercontent.com`)
- Identity mismatch (should match workflow identity)

### SLSA Verification Fails

```bash
# Verbose verification
slsa-verifier verify-artifact \
  --provenance-path nsip.intoto.jsonl \
  --source-uri github.com/USER/REPO \
  --print-provenance \
  nsip
```

**Common issues:**
- Source URI mismatch
- Builder version mismatch
- Artifact hash mismatch

## Monitoring & Compliance

### Rekor Transparency Log

All Cosign signatures logged to Rekor:

```bash
# Search Rekor for signatures
rekor-cli search --artifact nsip

# Get entry details
rekor-cli get --uuid <uuid>
```

**URL:** https://search.sigstore.dev/

### Provenance Inspection

```bash
# Extract provenance fields
cat nsip.intoto.jsonl | jq '.predicate.builder.id'
cat nsip.intoto.jsonl | jq '.predicate.metadata.buildStartedOn'
cat nsip.intoto.jsonl | jq '.predicate.materials[0].uri'
```

### Compliance Reports

Generate reports for audits:

```bash
# List all signed releases
gh release list --repo USER/REPO | while read line; do
  tag=$(echo $line | awk '{print $1}')
  echo "Release: $tag"
  gh release view $tag --json assets | jq -r '.assets[].name' | grep '\.sig$'
done
```

## Links

- [Sigstore Cosign](https://github.com/sigstore/cosign)
- [SLSA Framework](https://slsa.dev/)
- [SLSA GitHub Generator](https://github.com/slsa-framework/slsa-github-generator)
- [SLSA Verifier](https://github.com/slsa-framework/slsa-verifier)
- [Rekor Transparency Log](https://github.com/sigstore/rekor)
- [Supply Chain Security Guide](https://slsa.dev/spec/v1.0/)
