#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo xtask k8s-secrets-policy-test "$@"
