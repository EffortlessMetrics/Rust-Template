# ADR-0002: Nix-First Development Environment

**Status**: Accepted
**Date**: 2025-01-18
**Authors**: Steven Zimmerman
**Related ACs**: AC-TPL-005 (if environment setup AC exists)

---

## Context

Development environment drift is a chronic source of friction:

- "Works on my machine" due to different tool versions
- CI failures that don't reproduce locally
- Onboarding developers requires manual installs (Rust, conftest, yq, etc.)
- Updating tool versions requires coordination across team

Traditional approaches:

1. **Manual installs**: README with "install Rust 1.75, conftest 0.52, yq 4.40"
   - Fragile: versions drift, humans forget steps
2. **Docker devcontainers**: heavyweight, slow on macOS, doesn't solve native tooling
3. **asdf / mise / rtx**: better, but still requires per-tool setup and doesn't pin *everything*

We need:

- Dev environment that matches CI exactly
- Declarative: one file defines all tools + versions
- Reproducible: same environment on every machine, every time
- Fast: no Docker overhead for native compilation
- Escape hatch: still allow native Rust for teams that can't/won't use Nix

---

## Decision

We adopt **Nix-first** development via a `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";  # or aarch64-darwin, etc.
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          (pkgs.rust-bin.stable."1.75.0".default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          })
          pkgs.conftest  # 0.52.0
          pkgs.yq-go     # 4.40.5
          # ... other tools
        ];
      };
    };
}
```

**Core principles:**

1. **Golden path = Nix**
   - README says "run `nix develop`" first
   - CI uses Nix to get the same environment
   - All governance checks (conftest, yq, etc.) are pinned in the flake

2. **Native fallback exists**
   - `docs/dev-environment.md` documents manual install path
   - `xtask selftest` warns if tools are missing but doesn't fail (except in CI)
   - Users can still `cargo build` without Nix

3. **No secret Nix**
   - Flake is in repo root, visible and auditable
   - We don't hide Nix behind wrappers or scripts
   - Developers can inspect `nix flake show` to see what's pinned

4. **Direnv optional**
   - We document `echo "use flake" > .envrc` pattern
   - We don't commit `.envrc` (avoids surprising users who don't want auto-activation)

**Enforcement:**

- CI runs inside `nix develop -c ...` for all checks
- Locally, missing tools show warnings, not failures (except conftest in selftest)

---

## Consequences

### Positive

- **Reproducibility**: Every developer gets Rust 1.75.0, conftest 0.52.0, same as CI
- **Onboarding**: `nix develop` once, done (vs 5+ manual installs)
- **Version upgrades**: Change one line in flake, everyone gets the update via `nix flake update`
- **CI parity**: No more "CI failed but I can't repro" due to tool version skew
- **Cross-platform**: Works on Linux, macOS, WSL without Docker overhead

### Negative

- **Nix learning curve**: Team members unfamiliar with Nix may resist
- **Disk space**: Nix store can grow large (though `/nix/store` is shared across projects)
- **Adoption barrier**: Some orgs ban Nix or require Docker-only
- **Flake instability**: Nix flakes are still "experimental" (though widely used)

### Neutral

- **Build caching**: Nix doesn't change Cargo's build; `target/` behavior is the same
- **IDE integration**: Works fine; devshell provides rust-analyzer, LSP, etc.

---

## Compliance

**Automated:**

- CI job `.github/workflows/ci.yml` runs:
  ```yaml
  - uses: cachix/install-nix-action@v24
  - run: nix develop -c cargo run -p xtask -- selftest
  ```
- `xtask selftest` checks for conftest and warns if missing (but only fails in CI)

**Manual:**

- README and `docs/dev-environment.md` instruct developers to use Nix first
- If developer can't use Nix, they follow the fallback path and accept warnings

**Detection:**

- If a PR fails in CI but passes locally, check:
  1. Is local running in `nix develop`?
  2. Are tool versions the same? (`conftest --version`, `rustc --version`)

---

## Notes

**Why Nix over Docker devcontainers?**

- Nix is native: no VM overhead, no filesystem translation layer
- Nix is faster: incremental, composable, cached per-package
- Nix integrates with host tools (IDEs, shells) seamlessly

**Why not asdf / mise / rtx?**

- Those tools pin languages (Rust, Node), but not system tools (conftest, yq)
- Nix pins *everything*, including non-language tools
- Nix gives us the same environment in CI without extra setup

**Why allow native fallback?**

- Some teams can't/won't install Nix due to org policy
- Some developers want to use system Rust for other projects
- Forcing Nix 100% creates adoption friction

By making Nix the golden path but not mandatory, we get reproducibility for those who opt in, and pragmatism for those who can't.

**Direnv pattern:**

We document but don't commit `.envrc` because:

- Auto-activation can surprise developers who aren't expecting it
- Some teams prefer explicit `nix develop` invocation
- `.envrc` is opt-in, not mandatory

**Installation:**

```bash
# Linux/WSL
sh <(curl -L https://nixos.org/nix/install) --daemon

# macOS
sh <(curl -L https://nixos.org/nix/install)

# Enable flakes (add to ~/.config/nix/nix.conf):
experimental-features = nix-command flakes
```

**Migration from existing project:**

If you're adding Nix to a non-Nix Rust project:

1. Copy `flake.nix` from this template
2. Update tool versions to match your current setup
3. Run `nix develop` and verify `cargo build` works
4. Update CI to use `nix develop -c ...`
5. Document native fallback in your dev-environment.md

**References:**

- [Nix manual](https://nixos.org/manual/nix/stable/)
- [rust-overlay](https://github.com/oxalica/rust-overlay)
- [Cachix (for CI speedup)](https://www.cachix.org/)
