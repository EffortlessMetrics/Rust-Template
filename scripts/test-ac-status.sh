#!/usr/bin/env bash
# Test AC status mapping script

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$WORKSPACE_ROOT"

echo "Testing AC status script..."
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

FAILED=0

# Test 1: Verify script runs on current repo
echo -n "  Test 1: Script runs successfully... "
if python3 scripts/ac_status.py &>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 2: Verify feature_status.md is generated
echo -n "  Test 2: Generates feature_status.md... "
python3 scripts/ac_status.py &>/dev/null || true
if [ -f docs/feature_status.md ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 3: Verify AC-123 appears in output
echo -n "  Test 3: Output contains AC-123... "
if grep -q "AC-123" docs/feature_status.md; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 4: Verify status indicators present
echo -n "  Test 4: Status indicators present... "
if grep -qE "(✅|❌|❓)" docs/feature_status.md; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 5: Verify script fails properly when JUnit missing
echo -n "  Test 5: Fails gracefully without JUnit... "
mv target/junit/acceptance.xml target/junit/acceptance.xml.bak 2>/dev/null || true
if python3 scripts/ac_status.py &>/dev/null; then
    echo -e "${RED}✗${NC} (should have failed)"
    FAILED=$((FAILED + 1))
else
    echo -e "${GREEN}✓${NC}"
fi
mv target/junit/acceptance.xml.bak target/junit/acceptance.xml 2>/dev/null || true

echo ""

# Summary
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All AC status tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED AC status test(s) failed${NC}"
    exit 1
fi
