---
diataxis_type: reference
---
# Spell Checking with typos

## Overview

Automated spell checking for documentation, code comments, and string literals using [typos](https://github.com/crate-ci/typos).

**Workflow:** `.github/workflows/spell-check.yml`  
**Configuration:** `.typos.toml`  
**Behavior:** Warns on typos, does not fail CI

## How It Works

The workflow runs on:
- Every push to main/master
- All pull requests
- Manual workflow dispatch

It scans all files (except excluded paths) for common typos and suggests corrections.

## Configuration

Edit `.typos.toml` to customize behavior:

```toml
[default]
# Add regex patterns to ignore
extend-ignore-re = [
    "[0-9a-f]{40}",  # Git SHAs
]

[files]
# Exclude directories/files
extend-exclude = [
    "target/",
    "*.lock",
]

[default.extend-words]
# Project-specific dictionary
# "typo" = "correct"
```

## Usage

### Local Check

```bash
# Install typos
cargo install typos-cli

# Check for typos
typos

# Auto-fix typos
typos --write-changes
```

### CI Integration

The workflow runs automatically. Check the Actions tab for results.

**Warning Output:**
```
warning: `recieve` should be `receive`
  --> src/lib.rs:10
```

## Troubleshooting

### False Positives

Add to `.typos.toml`:

```toml
[default.extend-words]
myword = "myword"  # Accept as correct
```

### Ignore Specific Files

```toml
[files]
extend-exclude = [
    "docs/legacy/",
]
```

### Custom Dictionary

Create a project dictionary:

```toml
[default.extend-identifiers]
# Code identifiers
myvar = "myvar"

[default.extend-words]
# Documentation words
specialterm = "specialterm"
```

## Links

- [typos Documentation](https://github.com/crate-ci/typos)
- [Configuration Reference](https://github.com/crate-ci/typos/blob/master/docs/reference.md)
