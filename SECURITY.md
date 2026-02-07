# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| latest  | Yes                |
| < latest | No                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via [GitHub Security Advisories](https://github.com/zircote/nsip/security/advisories/new).

### What to Include

- A description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours of the report
- **Initial assessment**: Within 1 week
- **Fix and disclosure**: Coordinated with the reporter, typically within 90 days

### Disclosure Policy

We follow responsible disclosure practices:

1. The reporter privately notifies us of the vulnerability.
2. We work together to understand and fix the issue.
3. We release a patched version.
4. The vulnerability is publicly disclosed after users have had time to update.

### Scope

This policy applies to the nsip crate and its published artifacts. Third-party dependencies
are managed via `cargo-deny` and audited regularly through our CI pipeline.

## Security Measures

This project employs several security practices:

- **cargo-deny**: Audits dependencies for known vulnerabilities, license compliance, and banned crates
- **cargo-audit**: Checks for known security advisories in dependencies
- **Dependabot**: Automated dependency updates for security patches
- **No unsafe code**: The crate forbids `unsafe` unless explicitly justified
- **Minimal dependencies**: Only essential dependencies are included
