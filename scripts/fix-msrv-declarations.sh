#!/usr/bin/env bash
set -euo pipefail
cargo xtask fix-msrv-declarations "$@"
