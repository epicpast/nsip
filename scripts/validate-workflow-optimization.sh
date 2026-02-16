#!/bin/bash
# Workflow Optimization Validation Script

set -e

echo "╔════════════════════════════════════════════════╗"
echo "║  GitHub Actions Workflow Validation            ║"
echo "╚════════════════════════════════════════════════╝"
echo ""

ERRORS=0
WARNINGS=0

success() { echo "✓ $1"; }
warning() { echo "⚠ WARNING: $1"; WARNINGS=$((WARNINGS + 1)); }
error() { echo "✗ ERROR: $1"; ERRORS=$((ERRORS + 1)); }
info() { echo "ℹ $1"; }

echo "=== Checking Composite Actions ==="
[ -d ".github/actions/setup-rust-cached" ] && success "setup-rust-cached exists" || error "setup-rust-cached missing"
[ -d ".github/actions/install-cargo-tool" ] && success "install-cargo-tool exists" || error "install-cargo-tool missing"
echo ""

echo "=== Checking Action Version Consistency ==="
CHECKOUT_VERSIONS=$(grep -h "uses: actions/checkout@" .github/workflows/*.yml | sort -u | wc -l)
[ "$CHECKOUT_VERSIONS" -eq 1 ] && success "Consistent checkout versions" || warning "Found $CHECKOUT_VERSIONS different checkout versions"
echo ""

echo "=== Checking Concurrency Control ==="
PR_WORKFLOWS=$(grep -l "pull_request:" .github/workflows/*.yml | wc -l)
PR_WITH_CONCURRENCY=$(grep -l "pull_request:" .github/workflows/*.yml | xargs grep -l "concurrency:" | wc -l)
info "PR workflows with concurrency: $PR_WITH_CONCURRENCY/$PR_WORKFLOWS"
[ "$PR_WITH_CONCURRENCY" -ge 5 ] && success "Good concurrency coverage" || warning "Only $PR_WITH_CONCURRENCY/$PR_WORKFLOWS have concurrency"
echo ""

echo "=== Summary ==="
if [ "$ERRORS" -eq 0 ]; then
    echo "✓ All checks passed with $WARNINGS warnings!"
    exit 0
else
    echo "✗ Found $ERRORS errors and $WARNINGS warnings"
    exit 1
fi
