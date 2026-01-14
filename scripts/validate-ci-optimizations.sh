#!/usr/bin/env bash
set -euo pipefail

# Validation script for CI optimization changes
# Usage: ./scripts/validate-ci-optimizations.sh

echo "=== CI Optimization Validation ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass() {
    echo -e "${GREEN}✓${NC} $1"
}

fail() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo "ℹ $1"
}

# 1. Check composite actions exist
info "Checking composite actions..."
if [ -f ".github/actions/setup-rust-nix/action.yml" ]; then
    pass "setup-rust-nix composite action exists"
else
    fail "setup-rust-nix composite action missing"
fi

if [ -f ".github/actions/sccache-stats/action.yml" ]; then
    pass "sccache-stats composite action exists"
else
    fail "sccache-stats composite action missing"
fi

# 2. Validate composite action syntax
info "Validating composite action syntax..."
for action in .github/actions/*/action.yml; do
    if grep -q "^name:" "$action" && \
       grep -q "^description:" "$action" && \
       grep -q "^runs:" "$action"; then
        pass "$(basename "$(dirname "$action")") has valid structure"
    else
        fail "$(basename "$(dirname "$action")") has invalid structure"
    fi
done

# 3. Check workflows using composite actions
info "Checking workflows using composite actions..."
count=$(grep -l "uses: ./.github/actions" .github/workflows/*.yml 2>/dev/null | wc -l)
if [ "$count" -ge 5 ]; then
    pass "$count workflows using composite actions"
else
    warn "Only $count workflows using composite actions (expected 5+)"
fi

# 4. Check for concurrency in updated workflows
info "Checking concurrency configuration..."
for workflow in ci-agents.yml tier1-selftest.yml policy-test.yml; do
    if grep -q "concurrency:" ".github/workflows/$workflow" 2>/dev/null; then
        pass "$workflow has concurrency control"
    else
        fail "$workflow missing concurrency control"
    fi
done

# 5. Check artifact naming improvements
info "Checking artifact naming..."
if grep -q 'name: coverage-report-\${{ github.sha }}' .github/workflows/ci-coverage.yml 2>/dev/null; then
    pass "ci-coverage.yml has SHA in artifact name"
else
    warn "ci-coverage.yml artifact naming not updated"
fi

if grep -q 'name: tier1-selftest-artifacts' .github/workflows/tier1-selftest.yml 2>/dev/null; then
    pass "tier1-selftest.yml has unique artifact name"
else
    warn "tier1-selftest.yml artifact naming not updated"
fi

# 6. Check MSRV test execution
info "Checking MSRV workflow..."
if grep -q "Test with MSRV" .github/workflows/ci-msrv.yml 2>/dev/null; then
    pass "ci-msrv.yml runs tests"
else
    fail "ci-msrv.yml missing test execution"
fi

# 7. Check for sccache stats in workflows
info "Checking sccache stats reporting..."
stats_count=$(grep -l "sccache-stats" .github/workflows/*.yml 2>/dev/null | wc -l)
if [ "$stats_count" -ge 5 ]; then
    pass "$stats_count workflows reporting sccache stats"
else
    warn "Only $stats_count workflows reporting sccache stats"
fi

# 8. Verify no hardcoded sccache setup (should use composite action)
info "Checking for hardcoded sccache setup..."
hardcoded=$(grep -l "RUSTC_WRAPPER=.*sccache" .github/workflows/*.yml 2>/dev/null | wc -l || true)
if [ "$hardcoded" -eq 0 ]; then
    pass "No hardcoded sccache setup found (using composite action)"
else
    warn "$hardcoded workflows still have hardcoded sccache setup"
fi

# 9. Check documentation updated
info "Checking documentation..."
if [ -f "docs/CI_OPTIMIZATION_REPORT.md" ]; then
    pass "CI optimization report exists"
else
    fail "CI optimization report missing"
fi

if [ -f "CI_OPTIMIZATION_SUMMARY.md" ]; then
    pass "CI optimization summary exists"
else
    fail "CI optimization summary missing"
fi

# 10. Check for timeouts in all jobs
info "Checking for missing timeouts..."
missing_timeout=0
for workflow in .github/workflows/ci-*.yml .github/workflows/tier1-*.yml .github/workflows/policy-*.yml; do
    if [ -f "$workflow" ]; then
        if ! grep -q "timeout-minutes:" "$workflow"; then
            warn "$(basename "$workflow") missing timeout"
            missing_timeout=$((missing_timeout + 1))
        fi
    fi
done

if [ "$missing_timeout" -eq 0 ]; then
    pass "All workflows have timeouts"
else
    warn "$missing_timeout workflows missing timeouts"
fi

echo ""
echo "=== Validation Summary ==="
echo ""
info "All critical checks passed ✓"
info "Warnings indicate areas for future improvement"
echo ""
info "Next steps:"
echo "  1. Review changed files: git diff --stat"
echo "  2. Test workflows locally: act -j selftest"
echo "  3. Run actionlint: actionlint .github/workflows/*.yml"
echo "  4. Open PR and monitor first workflow runs"
echo ""
