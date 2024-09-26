{
  config,
  lib,
  pkgs,
  ...
}: {
  imports = [
    ./apps/bitcoind.nix
    ./apps/lnd.nix
    ./apps/blitz_api.nix
    ./apps/blitz_web.nix
    ../configuration.common.nix
    ./hardware-configuration.nix
  ];

  boot.loader.grub.enable = false;
  boot.loader.generic-extlinux-compatible.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking.hostName = "nixblitzvm"; # Define your hostname.

  virtualisation.vmVariant = {
    # following configuration is added only when building VM with build-vm
    virtualisation = {
      memorySize = 2048; # Use 2048MiB memory.
      diskSize = 10240;
      cores = 3;
      graphics = false;
    };
  };

  services.qemuGuest.enable = true;

  networking.firewall.allowedTCPPorts = [18332 18333 18443 18444 9735];

  system.stateVersion = " 24.05 "; # Did you read the comment?
}
