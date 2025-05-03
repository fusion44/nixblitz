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
  }: let
    name = "nixblitz";

    module = {
      nixosModules = {
        ${name} = {...}: {
          imports = [./modules/nixblitz.nix];
          nixpkgs.overlays = [self.overlays.default];
        };
        default = self.nixosModules.${name};
      };
    };

    overlays.overlays = {
      default = final: prev: {
        ${name} = self.packages.${prev.stdenv.hostPlatform.system}.${name};
      };
    };

    systems = flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      packages = {
        ${name} = pkgs.callPackage ./default.nix {};
        default = self.packages.${system}.${name};
      };

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
            just # the command runner
            nushell # alternative to Bash
            typos # code spell checker
            statix
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
  in
    overlays // module // systems;
}
