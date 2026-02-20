#!/usr/bin/env bash
set -euo pipefail

# Determine repo root (script may be run from anywhere)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== Build Infrastructure Testing Suite ==="
echo "Testing all critical build infrastructure fixes"
echo "Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "Repo root: ${REPO_ROOT}"
echo

# Test counter and results tracking
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
test_passed() {
    echo "  ✅ PASS: $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

test_failed() {
    echo "  ❌ FAIL: $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

run_test() {
    local description="$1"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo "🧪 Test $TESTS_TOTAL: $description"
    shift
    if "$@"; then
        test_passed "$description"
    else
        test_failed "$description"
    fi
    echo
}

# 1. Checksum File Validation Tests
echo "1. CHECKSUM FILE VALIDATION TESTS"
echo "==============================="

run_test "Checksum file exists and is readable" "[ -f ${REPO_ROOT}/scripts/tools.sha256 ]"

run_test "Checksum file contains actual checksum entries" "grep -q '^[a-zA-Z0-9-]*-[0-9a-zA-Z.-]* [a-f0-9]' ${REPO_ROOT}/scripts/tools.sha256"

run_test "All required tools have checksum entries" "[ \$(grep -c '^[a-zA-Z0-9-]*-[0-9a-zA-Z.-]* [a-f0-9]' ${REPO_ROOT}/scripts/tools.sha256) -ge 12 ]"

run_test "Checksum format is valid (new format with version and platform)" "grep -q '^oasdiff-1.11.7-.* [a-f0-9]\{64\}\$' ${REPO_ROOT}/scripts/tools.sha256 && grep -q '^buf-1.45.0-.* [a-f0-9]\{64\}\$' ${REPO_ROOT}/scripts/tools.sha256 && grep -q '^atlas-latest-.* [a-f0-9]\{64\}\$' ${REPO_ROOT}/scripts/tools.sha256"

run_test "All platforms are covered for main tools" "[ \$(grep -c 'oasdiff-1.11.7-linux-\|oasdiff-1.11.7-darwin-' ${REPO_ROOT}/scripts/tools.sha256) -eq 3 ] && [ \$(grep -c 'buf-1.45.0-linux-\|buf-1.45.0-darwin-' ${REPO_ROOT}/scripts/tools.sha256) -eq 4 ] && [ \$(grep -c 'atlas-latest-linux-\|atlas-latest-darwin-' ${REPO_ROOT}/scripts/tools.sha256) -eq 4 ]"

# 2. Rust Version Consistency Tests
echo "2. RUST VERSION CONSISTENCY TESTS"
echo "================================="

run_test "rust-toolchain.toml exists" "[ -f rust-toolchain.toml ]"

run_test "Cargo.toml exists" "[ -f Cargo.toml ]"

run_test "Rust versions are aligned" "[ \$(grep -E '^\s*channel\s*=\s*\"[0-9.]+\"' rust-toolchain.toml | sed -E 's/.*\"([0-9.]+)\".*/\1/') = \$(grep -E '^\s*rust-version\s*=\s*\"[0-9.]+\"' Cargo.toml | sed -E 's/.*\"([0-9.]+)\".*/\1/') ]"

run_test "Target Rust version is 1.92.0" "[ \$(grep -E '^\s*rust-version\s*=\s*\"[0-9.]+\"' Cargo.toml | sed -E 's/.*\"([0-9.]+)\".*/\1/') = '1.92.0' ]"

# 3. MSRV Validation Tests
echo "3. MSRV VALIDATION TESTS"
echo "========================"

total_crates=$(find crates -name Cargo.toml | wc -l)
crates_with_rust_version=$(grep -l 'rust-version' crates/*/Cargo.toml | wc -l)

run_test "All crates have MSRV declarations" "[ $crates_with_rust_version -eq $total_crates ]"

run_test "MSRV declarations use workspace reference" "[ \$(grep -c 'rust-version\.workspace = true' crates/*/Cargo.toml) -eq $total_crates ]"

# 4. Security Advisory Tests
echo "4. SECURITY ADVISORY TESTS"
echo "=========================="

run_test "deny.toml exists and is readable" "[ -f deny.toml ]"

run_test "Security advisories are documented with metadata" "grep -q '# RUSTSEC-2025-0057' deny.toml && grep -q '# Last reviewed:' deny.toml"

run_test "Ignored advisories have review dates" "grep -c '# Last reviewed: 2025-' deny.toml"

run_test "Ignored advisories have next review dates" "grep -c '# Next review: 2026-' deny.toml"

# 5. Bootstrap Tools Security Tests
echo "5. BOOTSTRAP TOOLS SECURITY TESTS"
echo "================================="

run_test "bootstrap-tools.sh exists and is executable" "[ -x ${REPO_ROOT}/bootstrap-tools.sh ]"

run_test "Bootstrap script has enhanced security messages" "grep -q '🚨 SECURITY ERROR' ${REPO_ROOT}/bootstrap-tools.sh"

run_test "Bootstrap script provides actionable recommendations" "grep -q 'Action required:' ${REPO_ROOT}/bootstrap-tools.sh"

# 6. Build Process Integration Tests
echo "6. BUILD PROCESS INTEGRATION TESTS"
echo "================================="

run_test "Validation script exists and is executable" "[ -x scripts/validate-build-infrastructure.sh ]"

run_test "MSRV fix script exists and is executable" "[ -x scripts/fix-msrv-declarations.sh ]"

run_test "Security advisory script exists and is executable" "[ -x scripts/fix-security-advisories.sh ]"

# 7. Functional Tests (if tools are available)
echo "7. FUNCTIONAL VALIDATION TESTS"
echo "============================="

# Test checksum validation with ENFORCE_CHECKSUMS=1
run_test "Checksum enforcement works correctly" "ENFORCE_CHECKSUMS=1 ${REPO_ROOT}/bootstrap-tools.sh 2>/dev/null"

# Test MSRV validation
run_test "MSRV validation passes with correct version" "cargo +1.92.0 check --workspace --quiet"

# Test security advisory scanning
run_test "Security advisory scan runs without errors" "cargo deny check advisories --quiet"

# 8. CI Integration Tests
echo "8. CI INTEGRATION TESTS"
echo "======================="

run_test "MSRV CI workflow exists" "[ -f .github/workflows/ci-msrv.yml ]"

run_test "MSRV CI workflow extracts version correctly" "grep -q 'rust-version.*Cargo.toml' .github/workflows/ci-msrv.yml"

run_test "MSRV CI workflow uses extracted version" "grep -q 'toolchain: \${{ steps.msrv.outputs.msrv }}' .github/workflows/ci-msrv.yml"

# Results Summary
echo "=== TEST RESULTS SUMMARY ==="
echo "Total tests run: $TESTS_TOTAL"
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $TESTS_FAILED"
echo "Success rate: $(( TESTS_PASSED * 100 / TESTS_TOTAL ))%"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    echo "🎉 ALL TESTS PASSED - Build infrastructure is secure and functional!"
    exit 0
else
    echo "⚠️  SOME TESTS FAILED - Review and fix remaining issues"
    exit 1
fi
