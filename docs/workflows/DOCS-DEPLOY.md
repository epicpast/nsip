---
diataxis_type: reference
---
# Documentation Deploy Workflow

## Overview

Builds Rustdoc API documentation (and optionally an mdBook user guide) and
deploys the combined output to GitHub Pages. Disabled by default; the push
trigger is commented out to prevent accidental deployments.

**Workflow:** `.github/workflows/docs-deploy.yml`  
**Trigger:** Manual (`workflow_dispatch`) — push and release triggers are
commented out  
**Required secrets:** None  
**Permissions:** `contents: read`, `pages: write`, `id-token: write`  
**Concurrency group:** `pages` (cancel-in-progress: false)

> This workflow is **not active by default**. See [Enabling](#enabling) below.

## Jobs

### `build-docs`

| Step | Description |
|------|-------------|
| Checkout | Clones the repository |
| Rust setup | Stable toolchain with Cargo cache |
| Build Rustdoc | `cargo doc --no-deps --all-features` → `target/doc/` |
| Install mdBook | `mdbook@0.4.40` via `.github/actions/install-cargo-tool` |
| Build mdBook | If `book.toml` exists, copies the built book to `target/doc/book/` |
| Landing page | Generates `target/doc/index.html` with links to API docs and guide |
| Configure Pages | Prepares the Pages deployment |
| Upload artifact | Uploads `target/doc/` as a Pages artifact |

### `deploy`

Deploys the uploaded artifact to the `github-pages` environment. The
deployment URL is available as `${{ steps.deployment.outputs.page_url }}`.

## Enabling

### Manual-only (current default)

Run the workflow from the **Actions** tab → **Deploy Documentation** →
**Run workflow**.

### Automatic deployment on push

Uncomment the `push` trigger in `.github/workflows/docs-deploy.yml`:

```yaml
on:
  push:
    branches: [main, master]
    paths:
      - 'docs/**'
      - 'crates/**/*.rs'
      - 'Cargo.toml'
  workflow_dispatch:
```

### Automatic deployment on release

Uncomment the `release` trigger:

```yaml
on:
  release:
    types: [published]
  workflow_dispatch:
```

## GitHub Pages Configuration

Ensure GitHub Pages is enabled in repository settings:

1. Go to **Settings → Pages**
2. Set **Source** to **GitHub Actions**
3. No branch selection is needed when deploying via Actions

## Adding an mdBook Guide

If you want an mdBook user guide alongside the API docs:

1. Create `book.toml` in the repository root
2. Add Markdown chapters to `src/`
3. The workflow will automatically detect `book.toml`, build the book,
   and place it at `<pages_url>/book/`

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Deployment fails | Pages not enabled | Enable GitHub Pages with Actions source in repository settings |
| Rustdoc warns treated as errors | Broken doc link or missing docs | Fix with `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` |
| mdBook not built | `book.toml` missing | Create `book.toml` or the step is intentionally skipped |
| `index.html` shows wrong crate name | Hardcoded `rust_template` in landing page | Update the landing page template in the workflow |
