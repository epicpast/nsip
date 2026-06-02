---
diataxis_type: reference
---
# Signed Releases Workflow

## Overview

Signs every asset in a GitHub Release using [Cosign](https://github.com/sigstore/cosign)
and generates SHA-256/SHA-512 checksum files — all signed with Cosign as well.
Runs automatically after the Release workflow completes.

**Workflow:** `.github/workflows/signed-releases.yml`  
**Trigger:** Successful completion of the Release workflow for a `v*.*.*` tag
ref (read from `workflow_run.head_branch`)  
**Required secrets:** None (keyless signing via Sigstore OIDC)  
**Permissions:** `contents: write`, `id-token: write`

## How It Works

1. **Detect tag** — reads the tag name from `workflow_run.head_branch`
2. **Install Cosign** — `sigstore/cosign-installer@v4`
3. **Download release assets** — fetches all assets from the GitHub Release
4. **Sign each asset** — `cosign sign-blob --yes <file> > <file>.sig` for every
   non-`.sig` file
5. **Generate checksums** — creates `SHA256SUMS` and `SHA512SUMS`
6. **Sign checksums** — signs both checksum files with Cosign
7. **Upload signatures** — attaches `*.sig`, `SHA256SUMS`, and `SHA512SUMS`
   to the release

## Keyless Signing

Cosign uses [Sigstore's keyless signing](https://docs.sigstore.dev/cosign/signing/overview/)
with GitHub OIDC as the identity provider. No private key is stored in the
repository. The signer identity is tied to the workflow's OIDC token, making
it verifiable and auditable.

## Verifying Signatures

```bash
# Download the release assets
gh release download v0.6.0 --repo zircote/nsip --pattern '*'

# Verify a specific binary (assets are bare, versioned binaries — no .tar.gz)
cosign verify-blob \
  --signature nsip-0.6.0-linux-amd64.sig \
  --certificate-identity \
    "https://github.com/zircote/nsip/.github/workflows/signed-releases.yml@refs/tags/v0.6.0" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  nsip-0.6.0-linux-amd64

# Verify checksums
sha256sum --check SHA256SUMS
```

## Checksum Files

| File | Algorithm | Contents |
|------|-----------|---------|
| `SHA256SUMS` | SHA-256 | Hash of every release asset (excluding `.sig` files) |
| `SHA512SUMS` | SHA-512 | Hash of every release asset (excluding `.sig` files) |
| `SHA256SUMS.sig` | Cosign | Cosign signature of `SHA256SUMS` |
| `SHA512SUMS.sig` | Cosign | Cosign signature of `SHA512SUMS` |

## Relationship to `release.yml`

The [Release workflow](RELEASE.md) builds the binaries and creates the GitHub
Release using `actions/attest-build-provenance` for SBOM and binary provenance.
This workflow **adds an additional signing layer** using Cosign detached
signatures, providing a complementary verification path.

See also: [Signed Releases security guide](../security/SIGNED-RELEASES.md).

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Workflow not triggered | Release workflow failed or tag is not semver | Ensure the Release workflow completes successfully |
| `cosign sign-blob` fails | `id-token: write` permission missing | Verify the permissions block in the workflow |
| Signature verification fails | Wrong certificate identity URL | Use the exact workflow path and tag ref in `--certificate-identity` |
| Duplicate `.sig` uploads | Workflow run twice for the same tag | Remove stale signatures; `--clobber` flag overwrites existing assets |
