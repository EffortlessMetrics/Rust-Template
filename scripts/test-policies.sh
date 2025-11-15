#!/usr/bin/env bash
# Legacy wrapper for policy tests - use 'cargo run -p xtask -- policy-test' instead
# This is a thin wrapper around 'cargo run -p xtask -- policy-test'

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"

# Run xtask policy-test
cargo run -p xtask -- policy-test
