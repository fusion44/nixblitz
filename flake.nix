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
            cargo-deny # Cargo plugin to generate list of all licenses for a crate
            rust-analyzer
            vscode-extensions.vadimcn.vscode-lldb.adapter # for rust debugging
            rustc # rust compiler
            rustfmt
            pre-commit # https://pre-commit.com
            rustPackages.clippy # rust linter
            python3 # to build the xcb Rust library
            nixd # for the flake files
            nodePackages.prettier # for the markdown files
            dbus # needed for an openssl package
            openssl
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
}
