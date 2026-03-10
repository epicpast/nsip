---
diataxis_type: how-to
---
# Dependency Updates

Runbook for managing Cargo and GitHub Actions dependencies in nsip.

---

## Dependabot Configuration

Dependabot is configured in `.github/dependabot.yml` with two ecosystems:

### Cargo Dependencies

| Setting | Value |
|---|---|
| Schedule | Weekly, Mondays at 09:00 America/Chicago |
| PR limit | 10 open PRs |
| Commit prefix | `chore(deps)` |
| Labels | `dependencies`, `rust` |
| Reviewer | `zircote` |

**Grouped updates** (minor + patch combined into single PRs):

| Group | Packages |
|---|---|
| `dev-dependencies` | `proptest*`, `test-*`, `criterion*` |
| `async-runtime` | `tokio*`, `async-*` |
| `serde-ecosystem` | `serde*` |

### GitHub Actions Dependencies

| Setting | Value |
|---|---|
| Schedule | Weekly, Mondays at 09:00 America/Chicago |
| PR limit | 5 open PRs |
| Commit prefix | `chore(deps)` |
| Labels | `dependencies`, `github-actions` |
| Reviewer | `zircote` |

**Grouped updates:** All actions grouped together for minor + patch updates.

---

## Dependabot Auto-Merge Policy

The `dependabot-automerge.yml` workflow automatically squash-merges Dependabot PRs that meet these criteria:

| Update type | Auto-merge? | Rationale |
|---|---|---|
| **Patch** (`0.0.x`) | Yes | Bug fixes, low risk |
| **Minor** (`0.x.0`) | Yes | Backward-compatible features |
| **Major** (`x.0.0`) | **No -- manual review** | Potentially breaking changes |

### What Auto-Merge Requires

Auto-merge is enabled but the PR still must pass all CI checks before merging. The flow is:

1. Dependabot opens a PR
2. `dependabot-automerge.yml` runs and enables auto-merge (squash) for patch/minor
3. CI (`ci.yml`) runs: fmt, clippy, test, doc, deny, msrv
4. Only after all CI checks pass does the PR actually merge

### When to Intervene Manually

- [ ] **Major version bumps** -- always review the changelog for breaking changes
- [ ] **PRs that fail CI** -- investigate the failure; the dependency update may be incompatible
- [ ] **Security advisories** -- expedite the merge; do not wait for the weekly cycle
- [ ] **Grouped PRs with many changes** -- scan the diff for unexpected changes

### Manually Merging a Dependabot PR

```bash
# Review the PR
gh pr view <number>
gh pr diff <number>

# Approve and merge
gh pr review <number> --approve
gh pr merge <number> --squash
```

---

## Manual Dependency Auditing

### cargo-deny (Supply Chain Policy)

`cargo-deny` runs in CI as part of the `deny` job. It checks four categories defined in `deny.toml`:

| Check | What it does | Failure mode |
|---|---|---|
| **advisories** | Known vulnerabilities (RustSec DB) | Deny all advisory types |
| **licenses** | Only allow listed SPDX licenses | Deny anything not in the allow-list |
| **bans** | Block specific crates | `openssl` (use rustls), `atty` (use std) |
| **sources** | Only allow crates.io | Deny unknown registries and git sources |

**Run locally:**

```bash
# Install
cargo install cargo-deny

# Run all checks
cargo deny check

# Run a specific check
cargo deny check advisories
cargo deny check licenses
cargo deny check bans
cargo deny check sources

# Generate a report
cargo deny list
```

### cargo-audit (Security Advisories)

The `security-audit.yml` workflow runs cargo-audit:
- **Daily** at 00:00 UTC (cron schedule)
- On every push that changes `Cargo.toml` or `Cargo.lock`
- Can be triggered manually via `workflow_dispatch`

**Run locally:**

```bash
# Install
cargo install cargo-audit

# Run audit
cargo audit

# Run with deny on warnings
cargo audit --deny warnings

# Generate JSON output for tooling
cargo audit --json
```

### Full Manual Audit

```bash
# 1. Update the advisory database
cargo audit fetch

# 2. Run cargo-audit
cargo audit --deny warnings

# 3. Run cargo-deny (broader checks)
cargo deny check

# 4. Check for outdated dependencies
cargo outdated

# 5. Inspect dependency tree
cargo tree
cargo tree --duplicates  # find duplicate crates
```

---

## Handling Security Advisories

When cargo-audit or Dependabot flags a security advisory:

### 1. Assess Severity

```bash
# Check the advisory details
cargo audit

# Check if the vulnerable code path is reachable
cargo tree -i <affected-crate>
```

### 2. Update the Dependency

```bash
# Update a specific dependency
cargo update -p <crate-name>

# Update all dependencies
cargo update

# Verify the update resolves the advisory
cargo audit
```

### 3. If No Fix Is Available

Options in order of preference:

1. **Pin to an unaffected version** in `Cargo.toml`
2. **Add to the ignore list** in `deny.toml` (temporary, with comment explaining why):
   ```toml
   [advisories]
   ignore = [
       "RUSTSEC-2024-XXXX",  # No fix available; unexploitable in our usage. Revisit by YYYY-MM-DD.
   ]
   ```
3. **Replace the dependency** with an alternative crate
4. **Fork and patch** the dependency

### 4. Verify and Push

```bash
cargo deny check advisories
cargo test --all-features
git add Cargo.toml Cargo.lock
git commit -m "fix(deps): address RUSTSEC-XXXX-YYYY in <crate>"
git push
```

---

## Updating Pinned GitHub Actions Versions

All GitHub Actions in this repository are pinned to full commit SHAs for supply chain security (not tags). When updating:

### 1. Find the New SHA

```bash
# Look up the commit SHA for a new release tag
gh api repos/<owner>/<action>/git/refs/tags/<tag> --jq '.object.sha'
```

Or visit the action's releases page and copy the full commit SHA from the tag.

### 2. Update the Workflow File

Replace the SHA and update the comment with the new version:

```yaml
# Before
uses: actions/checkout@OLD_SHA  # v6.0.2

# After
uses: actions/checkout@NEW_SHA  # v6.1.0
```

### 3. Update All Occurrences

The same action may appear in multiple workflow files. Search across all of them:

```bash
grep -r "actions/checkout@" .github/workflows/
```

Update every occurrence to the same SHA.

### 4. Let Dependabot Handle It

Dependabot is configured to update GitHub Actions weekly. For minor and patch updates, these are grouped and auto-merged. For major updates, review manually.

---

## Adding New Dependencies

### Evaluation Criteria

Before adding a new dependency, evaluate:

- [ ] **Necessity** -- Can this be done with `std` or existing dependencies?
- [ ] **Maintenance** -- Is the crate actively maintained? Check last commit date, open issues
- [ ] **License** -- Must be in the `deny.toml` allow-list: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, MPL-2.0, Unicode-DFS-2016, Unicode-3.0, CC0-1.0, BSL-1.0, 0BSD
- [ ] **Security** -- Run `cargo audit` after adding; check RustSec for known advisories
- [ ] **Size** -- Check transitive dependency count with `cargo tree -p <crate>`
- [ ] **Quality** -- Does it have tests? Documentation? Is it widely used?
- [ ] **Banned crates** -- Ensure it does not pull in `openssl` or `atty` (banned in `deny.toml`)
- [ ] **MSRV** -- Does the crate support Rust 1.92 (this project's MSRV)?

### Adding the Dependency

```bash
# Add the dependency
cargo add <crate-name>

# Or for dev-only
cargo add --dev <crate-name>

# Verify it passes all checks
cargo deny check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# Check the dependency tree impact
cargo tree -p <crate-name>
```

### If the License Is Not in the Allow-List

1. Evaluate whether the license is acceptable for your project
2. Add it to `deny.toml`:
   ```toml
   [licenses]
   allow = [
       # ...existing licenses...
       "NEW-LICENSE-SPDX",
   ]
   ```
3. Document why the license was added in the commit message

---

## Removing Unused Dependencies

### Detect Unused Dependencies

```bash
# Install cargo-machete
cargo install cargo-machete

# Find unused dependencies
cargo machete
```

### Remove a Dependency

```bash
# Remove from Cargo.toml
cargo remove <crate-name>

# Update the lock file
cargo update

# Verify nothing broke
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo deny check
```

---

## Dependabot Configuration Changes

### Adding a New Grouped Update

Edit `.github/dependabot.yml`:

```yaml
groups:
  new-group-name:
    patterns:
      - "crate-prefix*"
    update-types:
      - "minor"
      - "patch"
```

### Changing the Schedule

```yaml
schedule:
  interval: "daily"    # or "weekly" or "monthly"
  day: "monday"        # for weekly
  time: "09:00"
  timezone: "America/Chicago"
```

### Ignoring a Dependency

```yaml
ignore:
  - dependency-name: "crate-to-ignore"
    versions: [">=2.0.0"]  # ignore major updates only
```

---

## Quick Reference

| Task | Command |
|---|---|
| Run all supply chain checks | `cargo deny check` |
| Audit for security advisories | `cargo audit --deny warnings` |
| Update all dependencies | `cargo update` |
| Update one dependency | `cargo update -p <crate>` |
| List outdated dependencies | `cargo outdated` |
| Show dependency tree | `cargo tree` |
| Find duplicates | `cargo tree --duplicates` |
| Find unused dependencies | `cargo machete` |
| Add a dependency | `cargo add <crate>` |
| Remove a dependency | `cargo remove <crate>` |
| Check Dependabot PRs | `gh pr list --label dependencies` |
