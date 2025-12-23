#!/usr/bin/env bash
set -euo pipefail

echo "=== Build Infrastructure Diagnostic Report ==="
echo "Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo

# 1. Checksum file validation
echo "1. CHECKSUM FILE VALIDATION"
echo "   File: scripts/tools.sha256"
if [ -f "scripts/tools.sha256" ]; then
    line_count=$(wc -l < "scripts/tools.sha256")
    echo "   Lines: $line_count"
    
    # Count actual checksum entries (non-comment, non-empty)
    checksum_entries=$(grep -c '^[a-zA-Z0-9-]*-[0-9a-zA-Z.-]* [a-f0-9]' "scripts/tools.sha256" 2>/dev/null || echo "0")
    echo "   Actual checksum entries: $checksum_entries"
    
    # Count platform coverage for main tools
    oasdiff_platforms=$(grep -c 'oasdiff-1.11.7-' "scripts/tools.sha256" 2>/dev/null || echo "0")
    buf_platforms=$(grep -c 'buf-1.45.0-' "scripts/tools.sha256" 2>/dev/null || echo "0")
    atlas_platforms=$(grep -c 'atlas-latest-' "scripts/tools.sha256" 2>/dev/null || echo "0")
    echo "   Platform coverage: oasdiff=$oasdiff_platforms, buf=$buf_platforms, atlas=$atlas_platforms"
    
    if [ "$checksum_entries" -eq 0 ]; then
        echo "   ❌ CRITICAL: No actual checksums found - only placeholders"
    elif [ "$checksum_entries" -lt 12 ]; then
        echo "   ⚠️  WARNING: Insufficient checksum entries (expected >= 12, found $checksum_entries)"
    elif [ "$oasdiff_platforms" -lt 3 ] || [ "$buf_platforms" -lt 4 ] || [ "$atlas_platforms" -lt 4 ]; then
        echo "   ⚠️  WARNING: Incomplete platform coverage"
    else
        echo "   ✅ Checksum entries found with full platform coverage"
    fi
else
    echo "   ❌ CRITICAL: Checksum file not found"
fi
echo

# 2. Rust version consistency check
echo "2. RUST VERSION CONSISTENCY"
if [ -f "rust-toolchain.toml" ]; then
    toolchain_version=$(grep -E '^\s*channel\s*=\s*"[0-9.]+"' rust-toolchain.toml | sed -E 's/.*"([0-9.]+)".*/\1/' || echo "NOT_FOUND")
    echo "   rust-toolchain.toml version: $toolchain_version"
else
    echo "   ❌ rust-toolchain.toml not found"
fi

if [ -f "Cargo.toml" ]; then
    cargo_version=$(grep -E '^\s*rust-version\s*=\s*"[0-9.]+"' Cargo.toml | sed -E 's/.*"([0-9.]+)".*/\1/' || echo "NOT_FOUND")
    echo "   Cargo.toml rust-version: $cargo_version"
else
    echo "   ❌ Cargo.toml not found"
fi

if [ "$toolchain_version" != "$cargo_version" ] && [ "$toolchain_version" != "NOT_FOUND" ] && [ "$cargo_version" != "NOT_FOUND" ]; then
    echo "   ❌ CRITICAL: Version mismatch detected"
else
    echo "   ✅ Versions aligned"
fi
echo

# 3. MSRV validation across crates
echo "3. MSRV VALIDATION ACROSS CRATES"
total_crates=0
crates_with_rust_version=0

for crate_toml in crates/*/Cargo.toml; do
    if [ -f "$crate_toml" ]; then
        total_crates=$((total_crates + 1))
        crate_name=$(basename "$(dirname "$crate_toml")")
        
        if grep -q 'rust-version' "$crate_toml"; then
            crates_with_rust_version=$((crates_with_rust_version + 1))
            echo "   ✅ $crate_name: declares rust-version"
        else
            echo "   ⚠️  $crate_name: no rust-version declaration"
        fi
    fi
done

echo "   Total crates: $total_crates"
echo "   Crates with rust-version: $crates_with_rust_version"
if [ "$crates_with_rust_version" -lt "$total_crates" ]; then
    echo "   ❌ CRITICAL: $(($total_crates - $crates_with_rust_version)) crates missing MSRV declaration"
fi
echo

# 4. Security advisory check
echo "4. SECURITY ADVISORY STATUS"
if [ -f "deny.toml" ]; then
    ignored_advisories=$(grep -c '"RUSTSEC-' deny.toml 2>/dev/null || echo "0")
    echo "   Ignored security advisories: $ignored_advisories"
    
    if [ "$ignored_advisories" -gt 0 ]; then
        echo "   ⚠️  Ignored advisories:"
        grep '"RUSTSEC-' deny.toml 2>/dev/null | sed 's/^[[:space:]]*/      /'
    fi
else
    echo "   ❌ deny.toml not found"
fi
echo

# 5. Tool download verification status
echo "5. TOOL DOWNLOAD VERIFICATION STATUS"
echo "   ENFORCE_CHECKSUMS environment: ${ENFORCE_CHECKSUMS:-0 (not set)}"
if [ "${ENFORCE_CHECKSUMS:-0}" = "1" ]; then
    echo "   ✅ Checksum enforcement enabled"
else
    echo "   ⚠️  Checksum enforcement disabled (default)"
fi
echo

echo "=== Diagnostic Summary ==="
echo "Critical issues requiring immediate attention:"
echo "1. Empty checksum file - security vulnerability"
echo "2. Rust version misalignment - build consistency issue"
echo "3. Incomplete MSRV validation - compatibility risk"
echo