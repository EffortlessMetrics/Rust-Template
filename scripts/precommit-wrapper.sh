#!/usr/bin/env bash
set -euo pipefail

# Helper: are we already inside a Nix dev shell?
in_nix() {
  [[ "${IN_NIX_SHELL:-}" == "impure" || "${IN_NIX_SHELL:-}" == "pure" ]]
}

run_precommit_inside_nix() {
  set -euo pipefail

  echo "[pre-commit] Running cargo fmt..."
  # 1) Run rustfmt
  cargo fmt --all

  # 2) Stage only files changed by fmt (to avoid surprise staging of unrelated files)
  fmt_changed=$(git diff --name-only -- '*.rs' '*.rs.in' || true)
  if [[ -n "${fmt_changed}" ]]; then
    echo "[pre-commit] Staging files formatted by cargo fmt:"
    printf '  - %s\n' ${fmt_changed}
    git add ${fmt_changed}
  fi

  # 3) Now run the governed gate (fmt --check will now pass)
  echo "[pre-commit] Running xtask precommit..."
  cargo run -p xtask -- precommit
}

if in_nix; then
  # Already in nix develop (your usual terminal workflow)
  run_precommit_inside_nix
else
  # VS Code Git / plain WSL shell: hop into nix and do the same
  echo "[pre-commit] Not in nix develop; running fmt + precommit inside dev shell..." >&2
  exec nix develop --command bash -lc "
    set -euo pipefail
    echo '[pre-commit] Running cargo fmt...'
    cargo fmt --all

    fmt_changed=\$(git diff --name-only -- '*.rs' '*.rs.in' || true)
    if [[ -n \"\${fmt_changed}\" ]]; then
      echo '[pre-commit] Staging files formatted by cargo fmt:'
      printf '  - %s\n' \${fmt_changed}
      git add \${fmt_changed}
    fi

    echo '[pre-commit] Running xtask precommit...'
    cargo run -p xtask -- precommit
  "
fi
