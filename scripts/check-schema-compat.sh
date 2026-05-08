#!/usr/bin/env bash
set -euo pipefail
cargo xtask check-schema-compat "$@"
