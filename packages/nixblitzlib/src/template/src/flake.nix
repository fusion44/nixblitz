{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-bitcoin.url = "github:fort-nix/nix-bitcoin";
    blitz-api.url = "github:fusion44/blitz_api/nixosify";
    blitz-web.url = "github:fusion44/raspiblitz-web/nixosify";
    home-mgr = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    nix-bitcoin,
    blitz-api,
    blitz-web,
    home-mgr,
    ...
  }: {
    nixosConfigurations.nixblitzvm = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        home-mgr.nixosModule
        nix-bitcoin.nixosModules.default
        blitz-api.nixosModules.default
        blitz-web.nixosModules.default
        ./vm/configuration.nix
      ];
    };

    nixosConfigurations.nixblitzpi = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";
      modules = [
        home-mgr.nixosModule
        nix-bitcoin.nixosModules.default
        blitz-api.nixosModules.default
        blitz-web.nixosModules.default
        ./pi/configuration.nix
      ];
    };
  };
}
