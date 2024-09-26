{
  config,
  lib,
  pkgs,
  ...
}: {
  imports = [
    ./hardware-configuration.nix
    ./apps/bitcoind.nix
    ./apps/lnd.nix
    ./apps/blitz_api.nix
    ./apps/blitz_web.nix
    ../configuration.common.nix
  ];

  boot.loader.grub.enable = false;
  boot.loader.generic-extlinux-compatible.enable = true;

  networking.hostName = "nixblitzpi"; # Define your hostname.

  system.stateVersion = "24.05"; # Did you read the comment?

  networking.firewall.allowedTCPPorts = [18332 18333 18443 18444 9735];
}
