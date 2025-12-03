{
  description = "Rust spec-as-code template (Nix devShell)";
  # Note: If you see "warning: unknown setting 'lazy-trees'", this is a known cosmetic issue
  # caused by deprecated Nix 2.30+ setting in Determinate Nix installer's managed config.
  # See docs/TROUBLESHOOTING.md for details and fix instructions. Safe to ignore.
  inputs = { nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05"; fenix.url = "github:nix-community/fenix"; };
  outputs = { self, nixpkgs, fenix, ... }:
  let
    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    forAllSystems = f: nixpkgs.lib.genAttrs(systems) (system:
      f rec {
        inherit system;
        pkgs = import nixpkgs { inherit system; overlays = [ fenix.overlays.default ]; };
        rust = pkgs.fenix.stable.withComponents [ "cargo" "clippy" "rustfmt" "rust-src" "rust-analyzer" "llvm-tools-preview" ];
      });
  in {
    devShells = forAllSystems ({ pkgs, rust, ... }: {
      default = pkgs.mkShell {
        packages = [
          rust
          pkgs.just
          pkgs.git
          pkgs.curl
          pkgs.jq
          pkgs.yq-go
          pkgs.nodejs_20
          pkgs.nodePackages_latest.cspell
          pkgs.python3
          pkgs.gitleaks
          pkgs.conftest
          pkgs.kubectl
          pkgs.kustomize
          # cargo-audit and cargo-deny are installed via `cargo install` for lockfile v4 + edition 2024 support
          # Run: cargo install --locked cargo-audit cargo-deny
          pkgs.cargo-nextest
          pkgs.protobuf
          pkgs.zlib  # Required for rustc/sccache on systems without zlib1g
        ]
        # cargo-llvm-cov is marked broken on Darwin in nixpkgs; include only on Linux
        ++ pkgs.lib.optionals (!pkgs.stdenv.isDarwin) [
          pkgs.cargo-llvm-cov
        ];
        buildInputs = [ pkgs.zlib ];  # Also in buildInputs for linker visibility
        shellHook = ''
          # Prefer user cargo-installed tools (cargo-audit, cargo-deny, etc.)
          export PATH="$HOME/.cargo/bin:$PWD/.tools/bin:$PATH"
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.zlib ]}:$LD_LIBRARY_PATH"
          echo "DevShell ready — try: just checks"
        '';
      };
    });
  };
}
