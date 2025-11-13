{
  description = "Rust spec-as-code template (Nix devShell)";
  inputs = { nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05"; fenix.url = "github:nix-community/fenix"; };
  outputs = { self, nixpkgs, fenix, ... }:
  let
    systems = [ "x86_64-linux" "aarch64-darwin" ];
    forAllSystems = f: nixpkgs.lib.genAttrs(systems) (system:
      f rec {
        inherit system;
        pkgs = import nixpkgs { inherit system; overlays = [ fenix.overlay ]; };
        rust = pkgs.fenix.stable.withComponents [ "cargo" "clippy" "rustfmt" "rust-src" ];
      });
  in {
    devShells = forAllSystems ({ pkgs, rust, ... }: {
      default = pkgs.mkShell {
        packages = [ rust pkgs.just pkgs.git pkgs.curl pkgs.jq pkgs.yq-go pkgs.nodejs_20 pkgs.python3 pkgs.gitleaks pkgs.conftest pkgs.cargo-audit pkgs.cargo-deny pkgs.sccache pkgs.cargo-nextest ];
        shellHook = ''
          export PATH="$PWD/.tools/bin:$PATH"
          echo "DevShell ready — try: just checks"
        '';
      };
    });
  };
}
