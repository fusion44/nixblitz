{...}: {
  imports = [
    ./hardware-configuration.nix
    ../configuration.common.nix
  ];

  boot.loader.generic-extlinux-compatible.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking.hostName = "nixblitzx86";
}
