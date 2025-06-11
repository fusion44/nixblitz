{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # nixblitz.url = "github:fusion44/nixblitz/main";
    nixblitz.url = "git+https://forge.f44.fyi/f44/nixblitz";
    # nixblitz.url = "..";
    nixblitz.inputs.nixpkgs.follows = "nixpkgs";

    blitz-api.url = "github:fusion44/blitz_api/dev";
    blitz-api.inputs.nixpkgs.follows = "nixpkgs";

    blitz-web.url = "github:fusion44/raspiblitz-web/master";
    blitz-web.inputs.nixpkgs.follows = "nixpkgs";

    nixos-hardware.url = "github:nixos/nixos-hardware";

    disko.url = "github:nix-community/disko/latest";
    disko.inputs.nixpkgs.follows = "nixpkgs";

    targetSystem = {
      url = "path:./nix_store_cfg";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixblitz.follows = "nixblitz";
    };
  };

  outputs = {
    nixpkgs,
    nixblitz,
    ...
  } @ inputs: {
    nixosConfigurations = {
      nixblitzx86installer = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        specialArgs = {inherit inputs;};
        modules = [
          ({modulesPath, ...}: {
            imports = [(modulesPath + "/installer/cd-dvd/installation-cd-minimal.nix")];
          })
          nixblitz.nixosModules.nixblitz-cli
          nixblitz.nixosModules.nixblitz-install-engine
          nixblitz.nixosModules.nixblitz-norupo
          nixblitz.nixosModules.nixblitz-docs
          ./configuration.nix
        ];
      };
    };
  };
}
