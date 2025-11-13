#!/usr/bin/env bash
# Run all template tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}  Rust Template Test Suite${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

FAILED=0

# Test 1: Policy tests
echo -e "${BLUE}[1/3] Running policy tests...${NC}"
if bash "$SCRIPT_DIR/test-policies.sh"; then
    echo ""
else
    FAILED=$((FAILED + 1))
    echo ""
fi

# Test 2: AC status tests
echo -e "${BLUE}[2/3] Running AC status tests...${NC}"
if bash "$SCRIPT_DIR/test-ac-status.sh"; then
    echo ""
else
    FAILED=$((FAILED + 1))
    echo ""
fi

# Test 3: Bundler tests
echo -e "${BLUE}[3/3] Running bundler tests...${NC}"
if bash "$SCRIPT_DIR/test-bundler.sh"; then
    echo ""
else
    FAILED=$((FAILED + 1))
    echo ""
fi

# Summary
echo -e "${BLUE}======================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All test suites passed!${NC}"
    echo -e "${BLUE}======================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED test suite(s) failed${NC}"
    echo -e "${BLUE}======================================${NC}"
    exit 1
fi
