#!/usr/bin/env bash
set -euo pipefail

echo "=== Security Advisory Resolution Plan ==="
echo "Analyzing and providing fixes for ignored advisories"
echo

# Function to check if a dependency is dev-only
is_dev_only() {
    local crate="$1"

    # Check if the crate appears only in [dev-dependencies] across workspace
    if grep -r "^\s*$crate\s*=" crates/*/Cargo.toml | grep -v "dev-dependencies" >/dev/null 2>&1; then
        echo "false"
    else
        echo "true"
    fi
}

echo "1. Analyzing RUSTSEC-2025-0057 (fxhash unmaintained)"
echo "   Path: selectors → scraper → app-http (dev-only)"
echo "   Risk: Unmaintained dependency (not directly exploitable)"
echo "   Dev-only status: $(is_dev_only "scraper")"

echo "   Resolution Options:"
echo "   a) Wait for upstream fix in selectors crate"
echo "   b) Replace scraper with alternative (quick-xml + html5ever)"
echo "   c) Accept risk for dev-only dependency with monitoring"
echo

echo "2. Analyzing RUSTSEC-2025-0134 (rustls-pemfile unmaintained)"
echo "   Path: bollard → testcontainers → adapters-db-sqlx (dev-only)"
echo "   Risk: Unmaintained dependency (not directly exploitable)"
echo "   Dev-only status: $(is_dev_only "testcontainers")"

echo "   Resolution Options:"
echo "   a) Wait for upstream fix in bollard crate"
echo "   b) Replace testcontainers with alternative test setup"
echo "   c) Accept risk for dev-only dependency with monitoring"
echo

echo "3. Recommended Immediate Actions:"
echo "   ✓ Keep advisories ignored but add monitoring and timeline"
echo "   ✓ Add quarterly security review for these dependencies"
echo "   ✓ Document acceptance criteria for dev-only dependencies"
echo "   ✓ Create GitHub issue trackers for upstream fixes"
echo

echo "4. Long-term Security Strategy:"
echo "   - Implement automated dependency update workflow"
echo "   - Add security scanning to CI pipeline"
echo "   - Establish policy for dev-only dependency risks"
echo "   - Create security advisory response process"
echo

echo "5. deny.toml Enhancement:"
echo "   Add comments with:"
echo "   - Review dates"
echo "   - Acceptance rationale"
echo "   - Monitoring plan"
echo "   - Resolution timeline"
