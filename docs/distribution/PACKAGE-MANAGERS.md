# Package Manager Distribution

## Overview

Automated package generation for multiple platforms and package managers.

**Workflows:**
- `.github/workflows/package-homebrew.yml` - macOS Homebrew
- `.github/workflows/package-linux.yml` - Debian (.deb) and RPM (.rpm)
- `.github/workflows/package-snap.yml` - Snap packages (Linux)
- `.github/workflows/package-windows.yml` - Windows MSI installer

## Installation Methods

### Homebrew (macOS/Linux)

```bash
# Add tap
brew tap zircote/tap

# Install
brew install rust-template

# Update
brew upgrade rust-template
```

**Setup Requirements:**
1. Create `homebrew-tap` repository: `https://github.com/USER/homebrew-tap`
2. Add secret `HOMEBREW_TAP_TOKEN` with repo access
3. Formula auto-updates on releases

### Debian/Ubuntu (.deb)

```bash
# Download from releases
wget https://github.com/USER/REPO/releases/download/v0.1.0/rust-template_0.1.0_amd64.deb

# Install
sudo dpkg -i rust-template_0.1.0_amd64.deb

# Install dependencies if needed
sudo apt-get install -f
```

**Package Contents:**
- Binary: `/usr/bin/rust-template`
- Man pages: `/usr/share/man/man1/`
- Documentation: `/usr/share/doc/rust-template/`

### RPM (Fedora/RHEL/CentOS)

```bash
# Download from releases
wget https://github.com/USER/REPO/releases/download/v0.1.0/rust-template-0.1.0-1.x86_64.rpm

# Install
sudo rpm -i rust-template-0.1.0-1.x86_64.rpm

# Or with dnf
sudo dnf install ./rust-template-0.1.0-1.x86_64.rpm
```

### Snap (Universal Linux)

```bash
# Install from Snap Store
sudo snap install rust-template

# Or install from file
sudo snap install rust-template_0.1.0_amd64.snap --dangerous
```

**Snap Confinement:** `strict` - Limited system access for security

**Required Permissions:**
- `home` - Access user home directory
- `network` - Network connectivity

### Windows MSI

```powershell
# Download MSI from releases
# https://github.com/USER/REPO/releases/download/v0.1.0/rust-template-0.1.0-x64.msi

# Install via GUI or command line
msiexec /i rust-template-0.1.0-x64.msi

# Silent install
msiexec /i rust-template-0.1.0-x64.msi /quiet
```

**Install Location:** `C:\Program Files\rust-template\`

## Configuration

### Debian Package Metadata

Add to `Cargo.toml`:

```toml
[package.metadata.deb]
maintainer = "Your Name <email@example.com>"
copyright = "2026, Your Name"
license-file = ["LICENSE", "0"]
extended-description = """\
Detailed description of the package.
Multiple lines supported."""
depends = "$auto"
section = "utility"
priority = "optional"
assets = [
    ["target/release/rust-template", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/rust-template/", "644"],
]
```

### RPM Package Metadata

Add to `Cargo.toml`:

```toml
[package.metadata.generate-rpm]
name = "rust-template"
assets = [
    { source = "target/release/rust-template", dest = "/usr/bin/", mode = "755" },
    { source = "README.md", dest = "/usr/share/doc/rust-template/", mode = "644" },
]

[package.metadata.generate-rpm.requires]
# Add runtime dependencies if needed
```

### Snap Configuration

Edit `snap/snapcraft.yaml`:

```yaml
name: rust-template
base: core22
version: git
summary: One-line summary
description: |
  Multi-line description
  of your application

grade: stable  # or 'devel' for development
confinement: strict  # or 'classic' for full system access

apps:
  rust-template:
    command: bin/rust-template
    plugs:
      - home
      - network
      # Add more as needed:
      # - removable-media
      # - desktop
```

### Windows MSI Configuration

Create `wix/main.wxs` after running `cargo wix init`:

```xml
<?xml version='1.0' encoding='windows-1252'?>
<Wix xmlns='http://schemas.microsoft.com/wix/2006/wi'>
    <Product
        Id='*'
        Name='rust-template'
        UpgradeCode='YOUR-GUID-HERE'
        Manufacturer='Your Company'
        Language='1033'
        Version='$(var.Version)'>

        <Package InstallerVersion='450' Compressed='yes' InstallScope='perMachine' />

        <MajorUpgrade
            DowngradeErrorMessage='A newer version is already installed.' />

        <MediaTemplate EmbeddedCab='yes' />

        <Directory Id='TARGETDIR' Name='SourceDir'>
            <Directory Id='ProgramFiles64Folder'>
                <Directory Id='APPLICATIONFOLDER' Name='rust-template'>
                    <Component Id='MainExecutable'>
                        <File Source='target\release\rust-template.exe' />
                    </Component>
                </Directory>
            </Directory>
        </Directory>

        <Feature Id='Complete'>
            <ComponentRef Id='MainExecutable' />
        </Feature>
    </Product>
</Wix>
```

## CI/CD Integration

### On Release

All packages build automatically on GitHub release:

1. Tag release: `git tag v0.1.0 && git push origin v0.1.0`
2. Create GitHub release
3. Workflows trigger automatically
4. Packages attach to release

### Manual Trigger

```bash
# Trigger workflow manually
gh workflow run package-homebrew.yml -f version=0.1.0 -f dry_run=false
gh workflow run package-linux.yml
gh workflow run package-snap.yml
gh workflow run package-windows.yml
```

## Troubleshooting

### Debian Package Fails

```bash
# Check dependencies
cargo deb --no-build --no-strip --verbose

# Lint package
lintian target/debian/*.deb
```

### RPM Build Fails

```bash
# Check RPM metadata
cargo generate-rpm --auto-req disabled

# Verify spec
rpmlint target/generate-rpm/*.rpm
```

### Snap Build Fails

```bash
# Local snap build
snapcraft clean
snapcraft

# Check confinement issues
snap connections rust-template
```

### MSI Build Fails

```powershell
# Check WiX configuration
cargo wix --nocapture --verbose

# Verify MSI
msiexec /i target/wix/*.msi /l*v install.log
```

## Publishing to Stores

### Homebrew Core (Official)

For official Homebrew inclusion:

1. Formula must be popular and stable
2. Create PR to [homebrew-core](https://github.com/Homebrew/homebrew-core)
3. Follow [Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)

### Snap Store

```bash
# Login to Snap Store
snapcraft login

# Upload snap
snapcraft upload rust-template_0.1.0_amd64.snap --release stable

# Or use workflow automation with SNAPCRAFT_TOKEN secret
```

### Windows Package Manager (winget)

Create manifest in [winget-pkgs](https://github.com/microsoft/winget-pkgs):

```yaml
# manifests/r/rust-template/rust-template/0.1.0/rust-template.rust-template.yaml
PackageIdentifier: rust-template.rust-template
PackageVersion: 0.1.0
PackageLocale: en-US
Publisher: Your Name
PackageName: rust-template
License: MIT
ShortDescription: Modern Rust template
Installers:
  - Architecture: x64
    InstallerType: wix
    InstallerUrl: https://github.com/USER/REPO/releases/download/v0.1.0/rust-template-0.1.0-x64.msi
    InstallerSha256: HASH
ManifestType: singleton
ManifestVersion: 1.0.0
```

## Verification

### Test Installations

```bash
# Debian
docker run -it debian:latest bash -c "apt update && apt install -y ./rust-template.deb && rust-template --version"

# RPM
docker run -it fedora:latest bash -c "dnf install -y ./rust-template.rpm && rust-template --version"

# Snap
sudo snap install rust-template_*_amd64.snap --dangerous && rust-template --version
```

## Links

- [cargo-deb](https://github.com/kornelski/cargo-deb)
- [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm)
- [cargo-wix](https://github.com/volks73/cargo-wix)
- [Snapcraft Documentation](https://snapcraft.io/docs)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
