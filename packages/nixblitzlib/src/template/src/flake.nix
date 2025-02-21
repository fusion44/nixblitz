{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-bitcoin.url = "github:fort-nix/nix-bitcoin";
    blitz-api.url = "github:fusion44/blitz_api/nixosify";
    # blitz-api.url = "git+file:../../../../api/nixosify/";
    blitz-web.url = "github:fusion44/raspiblitz-web/nixosify";
    nixblitz.url = "github:fusion44/nixblitz/main";
    home-mgr = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixos-hardware.url = "github:nixos/nixos-hardware";
    raspberry-pi-nix.url = "github:nix-community/raspberry-pi-nix";
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
    nixosConfigurations.nixblitzvm = nixpkgs.lib.nixosSystem {
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

    nixosConfigurations.nixblitzpi4 = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";

      modules = [
        home-mgr.nixosModules.home-manager
        nixos-hardware.nixosModules.raspberry-pi-4
        "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
        nix-bitcoin.nixosModules.default
        blitz-api.nixosModules.default
        blitz-web.nixosModules.default
        nixblitz.nixosModules.default
        ./pi4/configuration.nix
      ];
    };

    nixosConfigurations.nixblitzpi5 = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";

      modules = [
        home-mgr.nixosModules.home-manager
        raspberry-pi-nix.nixosModules.raspberry-pi
        "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
        nix-bitcoin.nixosModules.default
        blitz-api.nixosModules.default
        blitz-web.nixosModules.default
        nixblitz.nixosModules.default
        ./pi5/configuration.nix
      ];
    };
    overlays.overlays = {
      default = final: prev: {
        ${name} = self.packages.${prev.stdenv.hostPlatform.system}.${name};
      };
    };

    images = {
      pi4 = self.nixosConfigurations.nixblitzpi4.config.system.build.sdImage;
      pi5 = self.nixosConfigurations.nixblitzpi5.config.system.build.sdImage;
    };
  };
}
