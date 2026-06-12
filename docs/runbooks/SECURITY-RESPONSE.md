---
diataxis_type: how-to
---
# Security Incident Response

Runbook for handling security vulnerabilities in nsip. Based on the project's [Security Policy](../../SECURITY.md).

---

## Receiving a Vulnerability Report

Vulnerability reports arrive through [GitHub Security Advisories](https://github.com/zircote/nsip/security/advisories).

**Do not** accept security reports through public issues, discussions, or social media. If someone reports a vulnerability publicly, immediately ask them to re-submit privately and consider the issue already disclosed when setting timelines.

### What to Expect in a Report

Per SECURITY.md, reporters are asked to provide:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

---

## Response Timeline

| Milestone | Deadline | Owner |
|---|---|---|
| **Acknowledge receipt** | Within 48 hours | Maintainer |
| **Initial assessment** | Within 1 week | Maintainer |
| **Fix development** | As soon as feasible | Maintainer |
| **Fix and disclosure** | Within 90 days (coordinated with reporter) | Maintainer + reporter |

---

## Triage Process

### 1. Acknowledge the Report (Within 48 Hours)

Respond to the advisory with:

- Confirmation that the report was received
- An estimated timeline for assessment
- Any immediate questions for the reporter

### 2. Severity Assessment

Use the CVSS framework or a simplified severity scale:

| Severity | Criteria | Response time target |
|---|---|---|
| **Critical** | Remote code execution, data exfiltration, supply chain compromise | Fix within 48 hours |
| **High** | Privilege escalation, denial of service, significant data exposure | Fix within 1 week |
| **Medium** | Limited impact, requires uncommon configuration or local access | Fix within 30 days |
| **Low** | Minimal impact, theoretical or defense-in-depth improvement | Fix within 90 days |

### 3. Impact Analysis

Determine the scope of the vulnerability:

- [ ] Is the vulnerability in nsip's own code or a dependency?
- [ ] Which versions are affected?
- [ ] What is the attack vector (network, local, physical)?
- [ ] Is there evidence of exploitation in the wild?
- [ ] What data or systems are at risk?
- [ ] Does this affect the published binary, Docker image, crate, or all of them?

### 4. Document the Assessment

Record findings in the GitHub Security Advisory draft:

- CVSS score (if applicable)
- Affected versions
- Affected components
- Exploitation prerequisites
- Mitigating factors

---

## Fix Development

### 1. Create a Private Fix Branch

Use GitHub's Security Advisory "collaborate on a fix" feature to create a temporary private fork:

1. Go to the advisory draft on GitHub
2. Click **"Start a temporary private fork"**
3. Create a branch for the fix in the private fork

This ensures the fix is not publicly visible before disclosure.

### 2. Develop the Fix

```bash
# Clone the private fork (GitHub provides the URL)
git clone <private-fork-url>
cd nsip

# Create a fix branch
git checkout -b security/fix-<advisory-id>

# Apply the fix
# ...

# Run the full test suite
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo deny check
cargo audit --deny warnings
```

### 3. Review the Fix

- At least one other maintainer should review the fix (in the private fork)
- Verify the fix addresses the root cause, not just the symptom
- Add a regression test for the vulnerability (without revealing exploit details)
- Confirm no new issues are introduced

### 4. Prepare Release Materials

While the fix is in review:

- [ ] Determine the new version number (typically a PATCH bump)
- [ ] Draft release notes that describe the fix without revealing exploit details before coordinated disclosure
- [ ] Prepare a CVE ID request if the severity warrants it
- [ ] Coordinate disclosure timing with the reporter

---

## Coordinated Disclosure

### Timeline

1. **Day 0:** Fix merged to the private fork and verified
2. **Day 0:** Publish the patched release (see [Emergency Release Process](#emergency-release-process) below)
3. **Day 0-3:** Notify the reporter that the fix is published
4. **Day 7-14:** Allow time for users to update
5. **Day 14+:** Publish the GitHub Security Advisory (makes it public)
6. **Day 14+:** CVE published (if requested)

### Publishing the Advisory

1. Go to the advisory draft at https://github.com/zircote/nsip/security/advisories
2. Fill in all required fields:
   - **Affected products:** `zircote/nsip`
   - **Affected versions:** version range
   - **Patched versions:** the new release version
   - **Severity:** based on your assessment
   - **CWE:** applicable weakness type
3. Click **"Publish advisory"**

This will:
- Make the advisory public
- Notify users watching the repository
- Add the advisory to the GitHub Advisory Database
- Trigger Dependabot alerts for affected downstream users

---

## Emergency Release Process

For critical and high severity vulnerabilities, use an expedited release process:

### 1. Merge the Fix

```bash
# Merge the private fork fix into main
# (GitHub provides a merge button in the advisory UI)
```

### 2. Bump Version

```bash
git pull origin main
# Update Cargo.toml version to X.Y.(Z+1)
git add Cargo.toml Cargo.lock
git commit -m "fix: address security vulnerability (GHSA-XXXX-XXXX-XXXX)"
git push origin main
```

### 3. Tag and Release

```bash
git tag -a vX.Y.(Z+1) -m "Security release vX.Y.(Z+1)"
git push origin vX.Y.(Z+1)
```

This triggers the standard release pipeline (release.yml, docker.yml, changelog.yml, publish.yml, package-homebrew.yml).

### 4. Verify Deployment

- [ ] GitHub Release created with binaries and attestations verified
- [ ] Docker image pushed to `ghcr.io/zircote/nsip`
- [ ] crates.io package updated (if enabled)
- [ ] All binaries pass smoke tests

### 5. Yank Affected Versions (If on crates.io)

```bash
# Yank each affected version
cargo yank --version X.Y.Z
```

### 6. Notify Users

- Publish the GitHub Security Advisory
- If the project has a mailing list or announcement channel, post there
- Update the release notes to reference the advisory

---

## Post-Incident Review

After the vulnerability is disclosed and patched, conduct a review:

- [ ] **Root cause:** What introduced the vulnerability?
- [ ] **Detection gap:** Why wasn't this caught by existing tooling?
- [ ] **Process improvement:** What can be improved?
  - Should a new lint rule be added?
  - Should a new cargo-deny ban be added?
  - Should CI checks be expanded?
- [ ] **Documentation:** Update SECURITY.md if the process needs changes
- [ ] **Timeline review:** Were response deadlines met?

---

## Automated Security Tools Overview

This project runs multiple layers of automated security scanning:

### Continuous (Every Push/PR)

| Tool | Workflow | What it checks |
|---|---|---|
| **cargo-deny** | `ci.yml` (deny job) | Advisories, licenses, banned crates, sources |
| **Gitleaks** | `secrets-scan.yml` | Accidentally committed secrets, API keys, tokens |
| **GitHub Secret Scanning** | `.github/secret_scanning.yml` | Provider-specific secret patterns in code |

### Scheduled

| Tool | Workflow | Schedule | What it checks |
|---|---|---|---|
| **cargo-audit** | `security-audit.yml` | Daily at 00:00 UTC | RustSec advisory database |
| **Trivy** | `container-scan.yml` | On-demand (`workflow_dispatch`) only — the `push`/`pull_request`/weekly `schedule` triggers exist in the workflow but are disabled (commented out) | Container image vulnerabilities |

### Dependency Management

| Tool | Configuration | What it does |
|---|---|---|
| **Dependabot** | `.github/dependabot.yml` | Opens PRs for outdated Cargo + Actions dependencies weekly |
| **Dependabot auto-merge** | `dependabot-automerge.yml` | Auto-merges patch and minor dependency updates after CI passes |

### cargo-deny Policy Summary (`deny.toml`)

| Policy | Setting | Details |
|---|---|---|
| Advisories | Deny all | No ignored advisories |
| Licenses | Allow-list only | MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, MPL-2.0, Unicode-DFS-2016, Unicode-3.0, CC0-1.0, BSL-1.0, 0BSD |
| Banned crates | `openssl` (use rustls), `atty` (use std) | Enforced as deny |
| Sources | crates.io only | Unknown registries and git sources denied |
| Multiple versions | Warn | Highlighted in output |
| Wildcards | Deny | No wildcard version requirements |

### What Each Tool Catches

```
Supply Chain Attack ──> cargo-deny (sources), Dependabot, secret scanning
Known Vulnerability ──> cargo-audit (daily), cargo-deny (advisories), Dependabot alerts
License Violation   ──> cargo-deny (licenses)
Container Vuln      ──> Trivy (container-scan, manual)
Leaked Secret       ──> Gitleaks, GitHub Secret Scanning
Unsafe Code         ──> Clippy + #[forbid(unsafe_code)] in crate
```

---

## Supported Versions

Per SECURITY.md:

| Version | Supported |
|---|---|
| Latest release | Yes |
| Older releases | No |

Only the latest release receives security patches. Users on older versions must upgrade.

---

## Quick Reference

| Action | Command / Location |
|---|---|
| View security advisories | https://github.com/zircote/nsip/security/advisories |
| Create new advisory | https://github.com/zircote/nsip/security/advisories/new |
| Run cargo-audit locally | `cargo audit --deny warnings` |
| Run cargo-deny locally | `cargo deny check` |
| Check for leaked secrets | `gitleaks detect` |
| View Dependabot alerts | https://github.com/zircote/nsip/security/dependabot |
| View code scanning alerts | https://github.com/zircote/nsip/security/code-scanning |
| Yank a crate version | `cargo yank --version X.Y.Z` |
