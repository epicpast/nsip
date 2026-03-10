---
diataxis_type: reference
---
# Alternative Cargo Registries

## Overview

Publishing Rust crates to alternative registries beyond crates.io for private packages, enterprise distribution, or specialized ecosystems.

## Why Alternative Registries?

- **Private packages** - Keep proprietary code internal
- **Enterprise control** - Host on internal infrastructure
- **Faster builds** - Geographic proximity, caching
- **Compliance** - Meet regulatory requirements
- **Mirrors** - Reduce dependency on crates.io

## Registry Options

### 1. Cloudsmith (Hosted SaaS)

**Best for:** Commercial projects, easy setup

```toml
# .cargo/config.toml
[registries.cloudsmith]
index = "sparse+https://dl.cloudsmith.io/basic/USER/REPO/cargo/index/"
token = "Bearer YOUR_TOKEN"
```

**Publish:**
```bash
cargo publish --registry cloudsmith
```

**Features:**
- Hosted solution (no infrastructure)
- Multiple package formats (npm, Maven, Docker)
- CDN distribution
- Access control and auditing
- Free tier available

**Setup:**
1. Sign up at https://cloudsmith.io/
2. Create Rust/Cargo repository
3. Get API token from account settings
4. Configure `.cargo/config.toml`

### 2. Artifactory (JFrog)

**Best for:** Enterprise with existing JFrog infrastructure

```toml
# .cargo/config.toml
[registries.artifactory]
index = "sparse+https://artifactory.company.com/artifactory/api/cargo/cargo-local/index/"
```

**Publish:**
```bash
# Set credentials
cargo login --registry artifactory

cargo publish --registry artifactory
```

**Features:**
- Enterprise-grade security
- Advanced access control
- Vulnerability scanning
- Build promotion
- High availability

**Setup:**
1. Install/access Artifactory instance
2. Create Cargo repository
3. Configure authentication
4. Set up replication (optional)

### 3. Freight (Self-Hosted)

**Best for:** Complete control, on-premise hosting

```toml
# .cargo/config.toml
[registries.freight]
index = "sparse+https://registry.company.com/index/"
```

**Publish:**
```bash
cargo publish --registry freight
```

**Features:**
- Open-source (MIT)
- Self-hosted
- S3-compatible storage
- PostgreSQL backend
- Docker deployment

**Setup:**
```bash
# Docker Compose
docker-compose up -d

# Configure registry
freight init --storage s3://bucket/path

# Add users
freight user add username --admin
```

**Links:** https://github.com/tantaman/freight

### 4. Kellnr (Self-Hosted)

**Best for:** Simple self-hosted option, small teams

```toml
# .cargo/config.toml
[registries.kellnr]
index = "sparse+https://kellnr.company.com/api/v1/crates"
```

**Publish:**
```bash
cargo publish --registry kellnr
```

**Features:**
- Written in Rust
- Simple deployment (single binary)
- Web UI
- Local cache/mirror
- Lightweight

**Setup:**
```bash
# Download binary
wget https://github.com/kellnr/kellnr/releases/latest/download/kellnr

# Run
./kellnr --port 8080
```

**Links:** https://kellnr.io/

### 5. Romt (Read-Only Mirror)

**Best for:** Offline/air-gapped environments

```bash
# Download full crates.io mirror
romt download --crates --index

# Serve locally
romt serve
```

**Configure:**
```toml
# .cargo/config.toml
[source.crates-io]
replace-with = "romt-mirror"

[source.romt-mirror]
registry = "sparse+http://localhost:8000/index/"
```

**Features:**
- Complete crates.io mirror
- Offline development
- Air-gapped networks
- Bandwidth savings

**Links:** https://github.com/drmikehenry/romt

### 6. Shipyard (Self-Hosted)

**Best for:** Kubernetes-native deployments

```toml
# .cargo/config.toml
[registries.shipyard]
index = "sparse+https://shipyard.k8s.company.com/index/"
```

**Deploy:**
```yaml
# kubernetes/shipyard.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: shipyard
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: shipyard
        image: shipyard/server:latest
        env:
        - name: STORAGE_BACKEND
          value: s3
```

**Features:**
- Kubernetes-native
- Horizontal scaling
- Cloud storage backends
- High availability

## Configuration

### Global Config

**`~/.cargo/config.toml`:**

```toml
# Define registries
[registries.company]
index = "sparse+https://registry.company.com/index/"
token = "Bearer YOUR_TOKEN"

[registries.staging]
index = "sparse+https://staging-registry.company.com/index/"

# Set default registry
[registry]
default = "company"

# Configure crates.io as backup
[source.crates-io]
registry = "https://github.com/rust-lang/crates.io-index"
```

### Project Config

**`.cargo/config.toml`:**

```toml
[registries.internal]
index = "sparse+https://internal.company.com/index/"

# Publish to internal registry by default
[registry]
default = "internal"

# Allow dependencies from multiple registries
[source.crates-io]
registry = "https://github.com/rust-lang/crates.io-index"
```

### Dependency Sources

```toml
# Cargo.toml
[dependencies]
# From crates.io (default)
serde = "1.0"

# From alternative registry
internal-lib = { version = "0.1", registry = "company" }

# From Git
custom = { git = "https://github.com/user/custom.git" }

# From local path
dev-tool = { path = "../dev-tool" }
```

## Publishing Workflow

### Manual Publishing

```bash
# Login to registry
cargo login --registry company

# Publish
cargo publish --registry company

# Verify
cargo search --registry company my-crate
```

### CI/CD Publishing

**GitHub Actions:**

```yaml
- name: Publish to internal registry
  env:
    CARGO_REGISTRIES_COMPANY_TOKEN: ${{ secrets.REGISTRY_TOKEN }}
  run: |
    cargo publish --registry company --token $CARGO_REGISTRIES_COMPANY_TOKEN
```

**Environment Variables:**
```bash
export CARGO_REGISTRIES_COMPANY_TOKEN="your-token"
cargo publish --registry company
```

## Registry Mirror/Cache

### Use Case: Reduce crates.io Load

```toml
# .cargo/config.toml
[source.crates-io]
replace-with = "company-mirror"

[source.company-mirror]
registry = "sparse+https://mirror.company.com/crates.io/"
```

**Benefits:**
- Faster builds (geographic proximity)
- Reduced external bandwidth
- Continued access during crates.io outages
- Compliance with network policies

### Implement Mirror

**With Romt:**
```bash
# Sync crates.io daily
romt download --crates --index --update

# Serve via nginx/apache
romt serve --host 0.0.0.0 --port 8080
```

**With Cloudsmith:**
- Configure upstream proxy
- Automatic caching
- CDN distribution

## Security Considerations

### 1. Authentication

```toml
# Token-based
[registries.secure]
index = "sparse+https://registry.company.com/index/"
token = "Bearer YOUR_TOKEN"

# Credential provider
[registries.secure]
credential-provider = "cargo:token"
```

### 2. TLS/HTTPS

```toml
# Enforce HTTPS
[registries.company]
index = "sparse+https://registry.company.com/index/"  # ✅

# Never use HTTP for sensitive data
index = "sparse+http://registry.company.com/index/"   # ❌
```

### 3. Access Control

Most registries support:
- User authentication
- Team-based permissions
- Read/write separation
- IP allowlists
- Audit logging

### 4. Package Signing

```bash
# Sign package
cargo package --sign

# Verify signature
cargo verify my-crate-0.1.0.crate
```

## Migration Strategies

### From crates.io to Private Registry

**Phase 1: Parallel Publishing**
```bash
# Publish to both
cargo publish  # crates.io
cargo publish --registry company  # private
```

**Phase 2: Update Dependencies**
```toml
[dependencies]
my-crate = { version = "0.2", registry = "company" }
```

**Phase 3: Deprecate Public**
- Archive crates.io package
- Update README with migration notice

### From Private to Public

**Checklist:**
- [ ] Remove proprietary code
- [ ] Add proper licensing
- [ ] Security review
- [ ] Documentation
- [ ] CI/CD for crates.io

## Troubleshooting

### Authentication Fails

```bash
# Check token
cargo login --registry company

# Debug
CARGO_LOG=cargo::ops::registry=trace cargo publish --registry company
```

### Index Not Found

```bash
# Verify index URL
curl https://registry.company.com/index/

# Check network
ping registry.company.com
```

### Slow Publish

```bash
# Check package size
cargo package --list | wc -l

# Exclude unnecessary files
# .cargo/config.toml
[package]
exclude = [
    "tests/fixtures/*",
    "*.tmp",
]
```

## Cost Comparison

| Registry | Hosting | Cost | Best For |
|----------|---------|------|----------|
| crates.io | Public | Free | Open source |
| Cloudsmith | SaaS | $50+/mo | Commercial |
| Artifactory | Self/Cloud | $$$$ | Enterprise |
| Kellnr | Self-hosted | Free | Small teams |
| Freight | Self-hosted | Free | Control freaks |
| Romt | Self-hosted | Free | Air-gapped |

## Best Practices

1. **Use sparse index** - Faster than git index
2. **Mirror crates.io** - Reduce external dependencies
3. **Automate publishing** - CI/CD integration
4. **Version carefully** - Follow semver strictly
5. **Document privately** - Internal registry docs
6. **Test thoroughly** - Before publishing
7. **Backup regularly** - Registry data

## Links

- [Alternative Registries RFC](https://rust-lang.github.io/rfcs/2141-alternative-registries.html)
- [Cargo Registry Documentation](https://doc.rust-lang.org/cargo/reference/registries.html)
- [Cloudsmith Cargo Guide](https://help.cloudsmith.io/docs/cargo-registry)
- [Artifactory Cargo Repository](https://www.jfrog.com/confluence/display/JFROG/Cargo+Repositories)
- [Kellnr Documentation](https://kellnr.io/documentation)
- [Freight GitHub](https://github.com/tantaman/freight)
