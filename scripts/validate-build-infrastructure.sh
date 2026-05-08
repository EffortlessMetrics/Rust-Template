#!/usr/bin/env bash
set -euo pipefail
cargo xtask validate-build-infrastructure "$@"
