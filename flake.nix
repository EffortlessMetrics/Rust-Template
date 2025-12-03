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
          pkgs.cargo-audit
          pkgs.cargo-deny
          pkgs.cargo-nextest
          pkgs.protobuf
          pkgs.cargo-llvm-cov
          pkgs.zlib  # Required for rustc/sccache on systems without zlib1g
        ];
        buildInputs = [ pkgs.zlib ];  # Also in buildInputs for linker visibility
        shellHook = ''
          export PATH="$PWD/.tools/bin:$PATH"
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.zlib ]}:$LD_LIBRARY_PATH"
          echo "DevShell ready — try: just checks"
        '';
      };
    });
  };
}
