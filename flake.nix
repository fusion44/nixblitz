{
  description = "NixBlitz dev env";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      devShell = with pkgs;
        mkShell {
          buildInputs = [
            alejandra # nix formatter
            cargo # rust package manager
            rust-analyzer
            vscode-extensions.vadimcn.vscode-lldb.adapter # for rust debugging
            rustc # rust compiler
            rustfmt
            pre-commit # https://pre-commit.com
            rustPackages.clippy # rust linter
            python3 # to build the xcb Rust library
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
}
