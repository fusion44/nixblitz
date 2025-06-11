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
    cli_name = "nixblitz-cli";
    docs_name = "nixblitz-docs";
    webapp_name = "nixblitz-norupo";

    module = {
      nixosModules = {
        ${cli_name} = {...}: {
          imports = [./modules/nixblitz_cli.nix];
          nixpkgs.overlays = [self.overlays.default];
        };
        ${docs_name} = {...}: {
          imports = [./modules/nixblitz_docs.nix];
          nixpkgs.overlays = [self.overlays.default];
        };
        ${webapp_name} = {...}: {
          imports = [./modules/nixblitz_norupo.nix];
          nixpkgs.overlays = [self.overlays.default];
        };
        default = self.nixosModules.${cli_name};
      };
    };

    overlays.overlays = {
      default = final: prev: {
        ${cli_name} = self.packages.${prev.stdenv.hostPlatform.system}.${cli_name};
        ${docs_name} = self.packages.${prev.stdenv.hostPlatform.system}.${docs_name};
        ${webapp_name} = self.packages.${prev.stdenv.hostPlatform.system}.${webapp_name};
      };
    };

    systems = flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      packages = {
        ${cli_name} = pkgs.callPackage ./crates/nixblitz_cli/default.nix {};
        ${docs_name} = pkgs.callPackage ./docs/default.nix {};
        ${webapp_name} = pkgs.callPackage ./crates/nixblitz_norupo/default.nix {};
        default = self.packages.${system}.${cli_name};
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
            statix
            nodePackages.prettier # for the markdown files
            dbus # needed for an openssl package
            openssl
            just # the command runner
            nushell # alternative to Bash
            typos # code spell checker
            statix
            fd
            dioxus-cli
            wasm-bindgen-cli
            lld
            nodejs
            tailwindcss_4
            watchman
            websocat

            # for the docs
            nodePackages.pnpm
            yarn
            eslint_d
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
