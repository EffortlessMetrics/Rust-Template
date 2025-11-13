#!/usr/bin/env bash
# Test all Rego policies against test fixtures

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TESTDATA_DIR="$WORKSPACE_ROOT/policy/testdata"

cd "$WORKSPACE_ROOT"

echo "Testing Rego policies..."
echo ""

FAILED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper to run a test
run_test() {
    local name="$1"
    local policy="$2"
    local fixture="$3"
    local should_pass="$4"  # "pass" or "fail"

    echo -n "  Testing $name... "

    if conftest test -p "$policy" "$fixture" &>/dev/null; then
        result="passed"
    else
        result="failed"
    fi

    if [ "$should_pass" = "pass" ] && [ "$result" = "passed" ]; then
        echo -e "${GREEN}✓${NC} (correctly passed)"
    elif [ "$should_pass" = "fail" ] && [ "$result" = "failed" ]; then
        echo -e "${GREEN}✓${NC} (correctly failed)"
    else
        echo -e "${RED}✗${NC} (expected $should_pass, got $result)"
        FAILED=$((FAILED + 1))
    fi
}

# Check conftest is available
if ! command -v conftest &>/dev/null; then
    echo -e "${RED}Error: conftest not found${NC}" >&2
    echo "Install with: brew install conftest" >&2
    echo "Or: docker run --rm -v \$(pwd):/project openpolicyagent/conftest test ..." >&2
    exit 1
fi

# Test ledger policy
echo "Ledger Policy (policy/ledger.rego):"
run_test "valid ledger" "policy/ledger.rego" "$TESTDATA_DIR/ledger_valid.json" "pass"
run_test "ledger with missing tests" "policy/ledger.rego" "$TESTDATA_DIR/ledger_missing_tests.json" "fail"
echo ""

# Test features policy
echo "Features Policy (policy/features.rego):"
run_test "valid features" "policy/features.rego" "$TESTDATA_DIR/features_valid.json" "pass"
run_test "features with unknown ACs" "policy/features.rego" "$TESTDATA_DIR/features_unknown_ac.json" "fail"
echo ""

# Test flags policy
echo "Flags Policy (policy/flags.rego):"
run_test "valid flags" "policy/flags.rego" "$TESTDATA_DIR/flags_valid.json" "pass"
run_test "invalid flags" "policy/flags.rego" "$TESTDATA_DIR/flags_invalid.json" "fail"
echo ""

# Test privacy policy
echo "Privacy Policy (policy/privacy.rego):"
run_test "valid privacy" "policy/privacy.rego" "$TESTDATA_DIR/privacy_valid.json" "pass"
run_test "invalid privacy" "policy/privacy.rego" "$TESTDATA_DIR/privacy_invalid.json" "fail"
echo ""

# Summary
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All policy tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED policy test(s) failed${NC}"
    exit 1
fi
