## What's Changed

<!-- Automatically filled by git-cliff -->

## Installation

### Binary Releases

Download pre-built binaries for your platform:

- **Linux (x86_64)**: `nsip-linux-amd64`
- **Linux (ARM64)**: `nsip-linux-arm64`
- **macOS (x86_64)**: `nsip-macos-amd64`
- **macOS (ARM64)**: `nsip-macos-arm64`
- **Windows (x86_64)**: `nsip-windows-amd64.exe`

### Cargo

```bash
cargo install nsip@VERSION
```

### Docker

```bash
docker pull ghcr.io/zircote/nsip:VERSION
```

## Verification

### Binary Checksums

Checksums (`SHA256SUMS`, `SHA512SUMS`) and Cosign signatures are generated and
attached automatically by the `signed-releases.yml` workflow after the release
publishes. Once attached, verify a downloaded asset against them:

```bash
sha256sum --check SHA256SUMS
```

### Docker Image

```bash
docker pull ghcr.io/zircote/nsip:VERSION
docker run --rm ghcr.io/zircote/nsip:VERSION --version
```

## Full Changelog

See [CHANGELOG.md](https://github.com/zircote/nsip/blob/main/CHANGELOG.md) for complete details.
