#!/usr/bin/env bash
# validate-ts-config.sh - Enforce TypeScript configuration standards
#
# This script ensures all tsconfig.json files in the repo follow modern
# TypeScript best practices:
#   - No deprecated moduleResolution values (node10, node)
#   - No ignoreDeprecations flags (masks future breakage)
#   - NodeNext module resolution is required
#
# Exit codes:
#   0 - All checks pass
#   1 - Violations found
#
# Usage:
#   ./scripts/validate-ts-config.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo "Validating TypeScript configuration standards..."
echo ""

violations=0

# Find all tsconfig.json files
while IFS= read -r -d '' tsconfig; do
    rel_path="${tsconfig#$REPO_ROOT/}"

    # Check for deprecated moduleResolution values
    # node10 and node are legacy and should not be used
    if grep -qE '"moduleResolution"\s*:\s*"(node10|node)"' "$tsconfig" 2>/dev/null; then
        echo -e "${RED}✗ $rel_path${NC}"
        echo "  - Uses deprecated moduleResolution (node10 or node)"
        echo "  - Fix: Use \"moduleResolution\": \"NodeNext\""
        violations=$((violations + 1))
    fi

    # Check for ignoreDeprecations
    # This flag masks important migration warnings
    if grep -qE '"ignoreDeprecations"' "$tsconfig" 2>/dev/null; then
        echo -e "${RED}✗ $rel_path${NC}"
        echo "  - Contains ignoreDeprecations flag"
        echo "  - Fix: Remove ignoreDeprecations and address warnings"
        violations=$((violations + 1))
    fi

    # Advisory: Check if moduleResolution is NodeNext (not enforced for now)
    if grep -qE '"moduleResolution"' "$tsconfig" 2>/dev/null; then
        if ! grep -qE '"moduleResolution"\s*:\s*"NodeNext"' "$tsconfig" 2>/dev/null; then
            echo -e "${YELLOW}⚠ $rel_path${NC}"
            echo "  - moduleResolution is not NodeNext (advisory)"
        fi
    fi

done < <(find "$REPO_ROOT" -name 'tsconfig.json' -not -path '*/node_modules/*' -not -path '*/.git/*' -print0)

echo ""

if [ "$violations" -gt 0 ]; then
    echo -e "${RED}Found $violations TypeScript config violation(s)${NC}"
    echo ""
    echo "TypeScript configuration standards for this repo:"
    echo "  - module: \"NodeNext\""
    echo "  - moduleResolution: \"NodeNext\""
    echo "  - No ignoreDeprecations flags"
    echo ""
    echo "See docs/how-to/implement-backstage-plugin.md for details."
    exit 1
fi

echo -e "${GREEN}✓ All TypeScript configurations pass validation${NC}"
exit 0
