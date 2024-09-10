{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    nix-bitcoin.url = "github:fort-nix/nix-bitcoin";
    blitz-api.url = "github:fusion44/blitz_api/nixosify";
    blitz-web.url = "github:fusion44/raspiblitz-web/nixosify";
  };

  outputs = inputs @ {
    nixpkgs,
    nix-bitcoin,
    blitz-api,
    blitz-web,
    ...
  }: {
    nixosConfigurations.nixblitz = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";
      modules = [
        nix-bitcoin.nixosModules.default
        blitz-api.nixosModules.default
        blitz-web.nixosModules.default
        ./configuration.nix
      ];
    };
  };
}
