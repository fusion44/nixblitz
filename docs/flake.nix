{
  description = "NixBlitz Docs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }: let
    name = "nixblitz-docs";

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
            nodejs
            nodePackages.pnpm
            yarn
            nodePackages.prettier
            eslint_d
            statix
            alejandra
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
  in
    overlays // systems;
}
