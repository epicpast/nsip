---
diataxis_type: reference
---
# Template Init Workflow

## Overview

A one-time initialisation workflow that replaces all template placeholder
references (`rust_template`, `rust-template`, `zircote`) with the actual
repository name and owner. Runs automatically on the first push to `main`
after creating a repository from the template. Becomes a no-op once
`Cargo.toml` no longer contains `name = "rust_template"`.

**Workflow:** `.github/workflows/template-init.yml`  
**Trigger:** Push to `main`, manual (`workflow_dispatch`)  
**Required secrets:** None (uses `GITHUB_TOKEN`)  
**Permissions:** `contents: write`  
**Guard:** Skipped entirely when `github.repository == 'zircote/rust-template'`
(the template source itself)

> **Note:** This workflow is specific to repositories created from the
> `zircote/rust-template` template. It is a no-op for `zircote/nsip` once
> initialisation has already occurred.

## What It Replaces

| Placeholder | Replaced with |
|-------------|--------------|
| `rust_template` | Snake-cased repository name (e.g., `my_crate`) |
| `rust-template` | Repository name (e.g., `my-crate`) |
| `zircote` | Repository owner (e.g., `myorg`) |

## How It Works

1. **Check if init is needed** — reads `Cargo.toml`; if `name = "rust_template"`
   is absent, skips all remaining steps
2. **Derive names** — splits `$GITHUB_REPOSITORY` into owner and repo parts,
   converts repo name to snake_case for the crate name
3. **Replace references** — runs `sed` over every tracked text file (excluding
   `.git/`, `.github/workflows/`, and binary files) to substitute all three
   placeholder strings
4. **Commit and push** — commits with `[skip ci]` and pushes directly to `main`

Files in `.github/workflows/` are intentionally excluded from replacement to
avoid breaking other workflow files.

## Idempotency

The initialisation check in step 1 makes the workflow safe to re-trigger. If
`Cargo.toml` no longer contains `name = "rust_template"`, the workflow exits
immediately without making any changes.

## After Initialisation

Once the init commit lands on `main`:

1. Review the changed files to ensure all substitutions are correct
2. Update `Cargo.toml` metadata fields that are not auto-replaced (e.g.,
   `description`, `authors`, `homepage`)
3. The workflow will not run again unless `Cargo.toml` is manually reverted

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Workflow runs on template source repo | Missing `if: github.repository != 'zircote/rust-template'` | The guard condition prevents this; do not remove it |
| Wrong owner/repo in replaced files | `$GITHUB_REPOSITORY` parsed incorrectly | Check the `Derive new names` step output in the workflow log |
| Workflow runs repeatedly | `Cargo.toml` reverted to contain `rust_template` | Restore the correct crate name |
| Binary files corrupted | `find` not excluding binary files | Ensure the `file` command filter is present in the replace step |
