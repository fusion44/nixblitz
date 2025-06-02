{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-bitcoin.url = "github:fort-nix/nix-bitcoin";
    nix-bitcoin.inputs.nixpkgs.follows = "nixpkgs";

    blitz-api.url = "github:fusion44/blitz_api/dev";
    blitz-api.inputs.nixpkgs.follows = "nixpkgs";

    blitz-web.url = "github:raspiblitz/raspiblitz-web/master";
    blitz-web.inputs.nixpkgs.follows = "nixpkgs";

    nixblitz.url = "git+https://forge.f44.fyi/f44/nixblitz";
    # nixblitz.url = "github:fusion44/nixblitz/main";
    nixblitz.inputs.nixpkgs.follows = "nixpkgs";

    home-mgr.url = "github:nix-community/home-manager";
    home-mgr.inputs.nixpkgs.follows = "nixpkgs";

    nixos-hardware.url = "github:nixos/nixos-hardware";

    raspberry-pi-nix.url = "github:nix-community/raspberry-pi-nix";
    raspberry-pi-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    nix-bitcoin,
    blitz-api,
    blitz-web,
    nixblitz,
    home-mgr,
    nixos-hardware,
    raspberry-pi-nix,
    ...
  }: let
    name = "nixblitz";
  in {
    nixosConfigurations = {
      nixblitzx86 = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          home-mgr.nixosModules.home-manager
          nix-bitcoin.nixosModules.default
          blitz-api.nixosModules.default
          blitz-web.nixosModules.default
          nixblitz.nixosModules.default
          ./x86/configuration.nix
        ];
      };

      nixblitzx86vm = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          home-mgr.nixosModules.home-manager
          nix-bitcoin.nixosModules.default
          blitz-api.nixosModules.default
          blitz-web.nixosModules.default
          nixblitz.nixosModules.default
          ./vm/configuration.nix
        ];
      };
    };

    overlays.overlays = {
      default = final: prev: {
        ${name} = self.packages.${prev.stdenv.hostPlatform.system}.${name};
      };
    };
  };
}
