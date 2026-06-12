---
diataxis_type: reference
---
# Release Attestations & Verification

## Overview

Every release artifact carries GitHub Artifact Attestations — SLSA build
provenance plus a CycloneDX SBOM attestation — created during the build and
**fail-closed verified before the GitHub Release is published**. A tag
publishes nothing unattested.

**Workflow:** `.github/workflows/release.yml` (build → attest → verify →
publish). The published crate is additionally attested by
`.github/workflows/publish.yml`.

## What Is Attested

| Artifact | Provenance | SBOM attestation |
|----------|-----------|------------------|
| `nsip-{version}-{platform}` (5 platform binaries) | Yes | Yes |
| `nsip-{version}-completions.tar.gz` | Yes | Yes |
| `nsip-{version}-man-pages.tar.gz` | Yes | Yes |
| `nsip-{version}.mcpb` (MCPB bundle) | Yes | Yes |
| `nsip-{version}-sbom.cdx.json` (CycloneDX SBOM) | — (it is the SBOM) | — |
| `nsip-{version}.crate` (crates.io, via `publish.yml`) | Yes | No |

A `nsip-{version}-checksums.txt` (SHA-256) file is generated for all assets
at publish time.

## Verifying Release Artifacts

Verification needs only the [GitHub CLI](https://cli.github.com/) (`gh`),
which validates the Sigstore bundle, the certificate identity, and the
subject digest against GitHub's attestation API.

```bash
# Download an asset (replace X.Y.Z with the release version)
gh release download vX.Y.Z --repo zircote/nsip \
  --pattern 'nsip-X.Y.Z-linux-amd64'

# Verify SLSA build provenance
gh attestation verify nsip-X.Y.Z-linux-amd64 --repo zircote/nsip

# Verify the SBOM attestation binding the artifact to its CycloneDX SBOM
gh attestation verify nsip-X.Y.Z-linux-amd64 --repo zircote/nsip \
  --predicate-type https://cyclonedx.org/bom
```

Successful output names the source repository, the workflow that built the
artifact, and the commit it was built from.

### Verify checksums

```bash
gh release download vX.Y.Z --repo zircote/nsip \
  --pattern 'nsip-X.Y.Z-checksums.txt'
sha256sum --check --ignore-missing nsip-X.Y.Z-checksums.txt
```

### Verify the published crate

`publish.yml` downloads the `.crate` file that crates.io actually serves,
asserts it is byte-identical to the locally packaged crate, and attaches
build provenance to it:

```bash
curl -fsSLO https://static.crates.io/crates/nsip/nsip-X.Y.Z.crate
gh attestation verify nsip-X.Y.Z.crate --repo zircote/nsip
```

crates.io publishing uses [Trusted Publishing](https://crates.io/docs/trusted-publishing)
(OIDC) — there is no long-lived registry token to leak.

## How It Works

1. **Build** — each platform binary, the completions/man-page archives, and
   the MCPB bundle are built in isolated jobs and attested with
   [`actions/attest-build-provenance`](https://github.com/actions/attest-build-provenance)
   (keyless Sigstore signing: Fulcio certificate, Rekor transparency log,
   GitHub OIDC identity).
2. **SBOM** — a CycloneDX SBOM is generated from the source tree (Syft) and
   bound to every artifact with
   [`actions/attest-sbom`](https://github.com/actions/attest-sbom).
3. **Verify (fail-closed)** — a dedicated job downloads every artifact and
   runs `gh attestation verify` for both predicates on each one. It also
   asserts the artifact count, so a partial set can never publish.
4. **Publish** — the GitHub Release is created only after verification
   succeeds (`needs: [verify, test, audit]`), and only on a tag ref.
   A `workflow_dispatch` run exercises the same chain as a dry run without
   publishing.

Keyless signing properties:

- No private keys to manage, rotate, or leak
- Identity is the GitHub Actions workflow (OIDC)
- Every signature is logged in the [Rekor](https://github.com/sigstore/rekor)
  transparency log — search at <https://search.sigstore.dev/>

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `gh attestation verify` fails | Asset re-uploaded or modified after attestation | Re-download; if it still fails, treat the asset as untrusted |
| No attestation found | Artifact predates this pipeline (≤ v0.6.x used Cosign `.sig` / `.sigstore.json` bundles) | Verify old releases with the instructions in that release's notes |
| Checksum mismatch | Partial download or tampering | Re-download; verify attestation before use |

## Links

- [GitHub Artifact Attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)
- [SLSA Framework](https://slsa.dev/)
- [Sigstore](https://www.sigstore.dev/)
- [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing)
- [Release workflow reference](../workflows/RELEASE.md)
