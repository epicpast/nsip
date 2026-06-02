---
diataxis_type: reference
---
# Secrets Scanning with Gitleaks

## Overview

Automated secrets detection to prevent credential leaks using [Gitleaks](https://github.com/gitleaks/gitleaks).

**Workflow:** `.github/workflows/secrets-scan.yml`  
**Configuration:** `.gitleaks.toml`  
**Behavior:** **FAILS CI** if secrets detected  
**Triggers:** Every push, all pull requests

## How It Works

Scans:
- New commits (on push)
- All commits in PR branch
- Files, comments, diffs

Detects:
- API keys
- Passwords
- Tokens (GitHub, AWS, etc.)
- Private keys
- Database connection strings
- Over 100+ secret patterns

## Configuration

Edit `.gitleaks.toml` to customize:

```toml
[extend]
useDefault = true  # Use built-in rules

[allowlist]
paths = [
    '''test/fixtures/secrets.txt''',  # Ignore test files
]

regexes = [
    '''EXAMPLE_.*''',  # Ignore example placeholders
]
```

## Usage

### Local Scanning

```bash
# Install gitleaks
brew install gitleaks
# or download the latest release for your platform from:
#   https://github.com/gitleaks/gitleaks/releases
# then extract the tarball, e.g.:
curl -sSfL <release-tarball-url> | tar -xz

# Scan current changes
gitleaks detect --source . --verbose

# Scan entire history
gitleaks detect --source . --log-opts="--all"

# Scan specific commit
gitleaks detect --source . --log-opts="--since=HEAD~1"
```

### Pre-commit Hook

Prevent secrets from being committed:

```bash
# .git/hooks/pre-commit
#!/bin/sh
gitleaks protect --verbose --redact --staged
```

```bash
chmod +x .git/hooks/pre-commit
```

## What Gets Detected

### Example Patterns

```
# AWS Access Key
AKIAIOSFODNN7EXAMPLE

# GitHub Personal Access Token
ghp_1234567890abcdefghijklmnopqrstuvwxyz

# Private SSH Key
-----BEGIN OPENSSH PRIVATE KEY-----

# Generic API Key
api_key=sk_live_1234567890abcdef

# Database URL
postgres://user:password@localhost:5432/db
```

## Handling Detections

### False Positives

1. **Add to allowlist** (`.gitleaks.toml`):

```toml
[allowlist]
regexes = [
    '''YOUR_PLACEHOLDER_TOKEN''',
]

paths = [
    '''docs/examples/''',
]
```

2. **Inline ignore**:

```rust
// crates/config.rs
let secret = "not-a-real-secret"; // gitleaks:allow
```

### True Positives (Leaked Secrets)

**CRITICAL:** If a secret is leaked:

1. **Rotate immediately** - Assume compromised
2. **Remove from history**:
   ```bash
   # Use BFG Repo-Cleaner
   bfg --replace-text secrets.txt repo.git
   ```
3. **Force push** (destructive):
   ```bash
   git push --force
   ```

## CI Integration

The workflow runs on every push and **fails CI** if secrets are found.

**Failure Example:**
```
Error: gitleaks detected secrets in commits
Finding: api_key="sk_live_..." 
  File: src/config.rs
  Line: 42
  Commit: abc1234
```

## Troubleshooting

### Slow Scans

Gitleaks scans git history. For faster scans:

```bash
# Scan only new commits
gitleaks protect --staged
```

### Ignoring Files

```toml
[allowlist]
paths = [
    '''\.lock$''',       # Lock files
    '''vendor/''',       # Vendored code  
    '''test/fixtures/''', # Test data
]
```

### Custom Rules

Add project-specific secret patterns:

```toml
[[rules]]
id = "custom-api-key"
description = "Project API Key"
regex = '''project_key_[0-9a-f]{32}'''
```

## Links

- [Gitleaks Documentation](https://github.com/gitleaks/gitleaks)
- [Configuration Guide](https://github.com/gitleaks/gitleaks/tree/master#configuration)
- [Rule Patterns](https://github.com/gitleaks/gitleaks/blob/master/config/gitleaks.toml)
