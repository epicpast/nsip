## What's Changed

<!-- Automatically filled by git-cliff -->

## Installation

### Binary Releases

Download pre-built binaries for your platform:

- **Linux (x86_64)**: `nsip-VERSION-linux-amd64`
- **Linux (ARM64)**: `nsip-VERSION-linux-arm64`
- **macOS (x86_64)**: `nsip-VERSION-macos-amd64`
- **macOS (ARM64)**: `nsip-VERSION-macos-arm64`
- **Windows (x86_64)**: `nsip-VERSION-windows-amd64.exe`

### Homebrew

```bash
brew install zircote/tap/nsip
```

### Cargo

```bash
cargo install nsip@VERSION
```

### Docker

```bash
docker pull ghcr.io/zircote/nsip:VERSION
```

## Verification

### Attestations

Every asset carries SLSA build provenance and a CycloneDX SBOM attestation,
verified fail-closed before this release was published. Re-verify any
download with the GitHub CLI:

```bash
gh attestation verify nsip-VERSION-linux-amd64 --repo zircote/nsip
gh attestation verify nsip-VERSION-linux-amd64 --repo zircote/nsip \
  --predicate-type https://cyclonedx.org/bom
```

### Binary Checksums

```bash
sha256sum --check --ignore-missing nsip-VERSION-checksums.txt
```

### Docker Image

```bash
docker pull ghcr.io/zircote/nsip:VERSION
docker run --rm ghcr.io/zircote/nsip:VERSION --version
```

## Full Changelog

See [CHANGELOG.md](https://github.com/zircote/nsip/blob/main/CHANGELOG.md) for complete details.
