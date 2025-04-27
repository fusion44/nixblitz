{...}: {
  imports = [
    ./hardware-configuration.nix
    ../configuration.common.nix
  ];

  boot.loader.systemd-boot.enable = true; # Enable systemd-boot
  boot.loader.efi.canTouchEfiVariables = true;

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

  networking.hostName = "nixblitzvm";
}
