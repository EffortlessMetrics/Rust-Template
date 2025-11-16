#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo hakari generate
cargo hakari verify