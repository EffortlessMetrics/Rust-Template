#!/usr/bin/env bash
# Test LLM context bundler

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$WORKSPACE_ROOT"

echo "Testing LLM context bundler..."
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

FAILED=0

# Test 1: Verify script runs with valid task
echo -n "  Test 1: Bundles implement_ac successfully... "
if bash scripts/make-context.sh implement_ac &>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 2: Verify bundle file is created
echo -n "  Test 2: Bundle file created... "
if [ -f .llm/bundle/implement_ac.md ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 3: Verify bundle is non-empty
echo -n "  Test 3: Bundle is non-empty... "
if [ -s .llm/bundle/implement_ac.md ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 4: Verify bundle contains expected header
echo -n "  Test 4: Bundle has proper header... "
if grep -q "Context Bundle: implement_ac" .llm/bundle/implement_ac.md; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 5: Verify bundle contains file markers
echo -n "  Test 5: Bundle contains FILE markers... "
if grep -q "^# FILE:" .llm/bundle/implement_ac.md; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

# Test 6: Verify script fails with invalid task
echo -n "  Test 6: Fails with invalid task... "
if bash scripts/make-context.sh invalid_task_name &>/dev/null; then
    echo -e "${RED}✗${NC} (should have failed)"
    FAILED=$((FAILED + 1))
else
    echo -e "${GREEN}✓${NC}"
fi

# Test 7: Verify all defined tasks work
echo -n "  Test 7: All defined tasks bundle successfully... "
ALL_TASKS_OK=1
for task in implement_ac implement_feature debug_tests; do
    if ! bash scripts/make-context.sh "$task" &>/dev/null; then
        ALL_TASKS_OK=0
        break
    fi
done
if [ $ALL_TASKS_OK -eq 1 ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    FAILED=$((FAILED + 1))
fi

echo ""

# Summary
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All bundler tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED bundler test(s) failed${NC}"
    exit 1
fi
