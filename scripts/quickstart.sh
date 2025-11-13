#!/usr/bin/env bash
# Quick validation script for template functionality
# Run this after cloning to verify everything works

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$WORKSPACE_ROOT"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}  Rust Template Quick Start${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

FAILED=0

# Step 1: Check environment
echo -e "${BLUE}[1/5] Checking environment...${NC}"
if command -v cargo &>/dev/null; then
    CARGO_VERSION=$(cargo --version | awk '{print $2}')
    echo -e "  ${GREEN}✓${NC} cargo $CARGO_VERSION"
else
    echo -e "  ${RED}✗${NC} cargo not found"
    FAILED=$((FAILED + 1))
fi

if command -v rustc &>/dev/null; then
    RUSTC_VERSION=$(rustc --version | awk '{print $2}')
    echo -e "  ${GREEN}✓${NC} rustc $RUSTC_VERSION"
else
    echo -e "  ${RED}✗${NC} rustc not found"
    FAILED=$((FAILED + 1))
fi
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Environment check failed. Install Rust or enter 'nix develop' shell.${NC}"
    exit 1
fi

# Step 2: Run checks
echo -e "${BLUE}[2/5] Running xtask check...${NC}"
if cargo run -p xtask -- check &>/tmp/check.log; then
    echo -e "  ${GREEN}✓${NC} Format check passed"
    echo -e "  ${GREEN}✓${NC} Clippy passed"
    echo -e "  ${GREEN}✓${NC} Tests passed"
else
    echo -e "  ${RED}✗${NC} Checks failed. See /tmp/check.log"
    FAILED=$((FAILED + 1))
fi
echo ""

# Step 3: Run BDD tests
echo -e "${BLUE}[3/5] Running BDD acceptance tests...${NC}"
if cargo run -p xtask -- bdd &>/tmp/bdd.log; then
    echo -e "  ${GREEN}✓${NC} BDD scenarios passed"
    if [ -f "target/junit/acceptance.xml" ]; then
        echo -e "  ${GREEN}✓${NC} JUnit output created"
    else
        echo -e "  ${YELLOW}⚠${NC} JUnit output not found (expected at target/junit/acceptance.xml)"
    fi
else
    echo -e "  ${RED}✗${NC} BDD tests failed. See /tmp/bdd.log"
    FAILED=$((FAILED + 1))
fi
echo ""

# Step 4: Test bundler
echo -e "${BLUE}[4/5] Testing LLM context bundler...${NC}"
if cargo run -p xtask -- bundle implement_ac &>/tmp/bundle.log; then
    echo -e "  ${GREEN}✓${NC} Bundle command executed"
    if [ -f ".llm/bundle/implement_ac.md" ]; then
        BUNDLE_SIZE=$(wc -c < .llm/bundle/implement_ac.md)
        echo -e "  ${GREEN}✓${NC} Bundle created ($BUNDLE_SIZE bytes)"
    else
        echo -e "  ${YELLOW}⚠${NC} Bundle file not found"
    fi
else
    echo -e "  ${RED}✗${NC} Bundler failed. See /tmp/bundle.log"
    FAILED=$((FAILED + 1))
fi
echo ""

# Step 5: Test scripts
echo -e "${BLUE}[5/5] Testing helper scripts...${NC}"
if python3 scripts/ac_status.py &>/tmp/ac_status.log; then
    echo -e "  ${GREEN}✓${NC} AC status script works"
    if [ -f "docs/feature_status.md" ]; then
        echo -e "  ${GREEN}✓${NC} Feature status generated"
    fi
else
    # AC status script may fail without complete setup, that's OK
    echo -e "  ${YELLOW}⚠${NC} AC status script needs full setup"
fi

if bash scripts/make-context.sh implement_ac &>/tmp/make_context.log; then
    echo -e "  ${GREEN}✓${NC} Context bundler script works"
else
    echo -e "  ${YELLOW}⚠${NC} Context bundler needs configuration"
fi
echo ""

# Summary
echo -e "${BLUE}======================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ Template validation passed!${NC}"
    echo ""
    echo -e "Next steps:"
    echo -e "  • See ${BLUE}docs/how-to/new-service-from-template.md${NC} for adoption guide"
    echo -e "  • See ${BLUE}TEMPLATE_API.md${NC} for stable interface documentation"
    echo -e "  • See ${BLUE}docs/tutorials/first-ac-change.md${NC} for AC-first development"
    echo -e "${BLUE}======================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED validation step(s) failed${NC}"
    echo ""
    echo -e "Check logs in /tmp/ for details:"
    echo -e "  • /tmp/check.log"
    echo -e "  • /tmp/bdd.log"
    echo -e "  • /tmp/bundle.log"
    echo -e "${BLUE}======================================${NC}"
    exit 1
fi
