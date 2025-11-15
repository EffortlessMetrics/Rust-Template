#!/usr/bin/env bash
# Run all template tests via xtask
# This is a thin wrapper around 'cargo run -p xtask -- selftest'

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}  Rust Template Test Suite${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

cd "$ROOT_DIR"

# Run xtask selftest (which includes policy tests, bundler tests, etc.)
echo -e "${BLUE}Running xtask selftest...${NC}"
cargo run -p xtask -- selftest

echo ""
echo -e "${GREEN}✓ Template self-test passed!${NC}"
