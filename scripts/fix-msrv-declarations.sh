#!/usr/bin/env bash
set -euo pipefail

echo "=== Fixing MSRV Declarations Across Workspace ==="
echo "Target: Add rust-version.workspace = true to all crates"
echo

# List of crates that need MSRV declaration (from diagnostic)
crates_needing_fix=(
    "acceptance"
    "ac-kernel" 
    "adapters-db-sqlx"
    "adapters-grpc"
    "app-http"
    "business-core"
    "model"
    "rust_iac_xtask_core"
    "spec-runtime"
    "telemetry"
    "xtask"
)

fixed_count=0
total_count=${#crates_needing_fix[@]}

for crate in "${crates_needing_fix[@]}"; do
    cargo_toml="crates/$crate/Cargo.toml"
    
    if [ -f "$cargo_toml" ]; then
        echo "Processing: $crate"
        
        # Check if rust-version already exists
        if grep -q 'rust-version' "$cargo_toml"; then
            echo "  ✅ Already has rust-version declaration"
        else
            # Find the line with edition.workspace = true and add rust-version after it
            if grep -q 'edition.workspace = true' "$cargo_toml"; then
                # Use sed to add rust-version after edition.workspace
                sed -i '/edition\.workspace = true/a rust-version.workspace = true' "$cargo_toml"
                echo "  ✅ Added rust-version.workspace = true"
                fixed_count=$((fixed_count + 1))
            else
                echo "  ❌ No edition.workspace found - manual intervention needed"
            fi
        fi
    else
        echo "  ❌ Cargo.toml not found for $crate"
    fi
done

echo
echo "=== MSRV Fix Summary ==="
echo "Crates processed: $total_count"
echo "Crates fixed: $fixed_count"
echo "Remaining issues: $((total_count - fixed_count))"
echo
echo "Next steps:"
echo "1. Run 'cargo check --workspace' to verify fixes"
echo "2. Run './scripts/validate-build-infrastructure.sh' to confirm"
echo "3. Test MSRV validation with: cargo +1.89.0 check --workspace"