{
  lib,
  pkgs,
  ...
}: let
  user = "nixos";
  isoLabel = "NIXBLITZ";
in {
  boot.loader.grub.enable = false;

  nixpkgs.config.allowUnfree = false;
  time.timeZone = "America/New_York";
  i18n.defaultLocale = "en_US.utf8";

  system.nixos.label = isoLabel;
  isoImage.volumeID = isoLabel;
  boot.kernelParams = ["iso_label=${isoLabel}"];

  nix = {
    distributedBuilds = true;

    settings = {
      builders-use-substitutes = true;
      auto-optimise-store = true;
      experimental-features = "nix-command flakes";
    };
  };

  console = {
    font = "Lat2-Terminus16";
    useXkbConfig = true;
  };

  environment.systemPackages = with pkgs; [
    bat
    fzf
    git
    vim
    just
    disko
    bottom
    neovim
    lazygit
    ripgrep
    superfile
    nixos-anywhere
  ];

  users.users."${user}" = {
    hashedPassword = "$6$rounds=10000$moY2rIPxoNODYRxz$1DESwWYweHNkoB6zBxI3DUJwUfvA6UkZYskLOHQ9ulxItgg/hP5CRn2Fr4iQGO7FE16YpJAPMulrAuYJnRC9B."; # nixblitz
    hashedPasswordFile = lib.mkForce null;
    password = lib.mkForce null;
    initialPassword = lib.mkForce null;
    initialHashedPassword = lib.mkForce null;
    openssh.authorizedKeys.keys = [];
  };

  services = {
    openssh = {
      enable = true;
      ports = [22];
      settings = {
        PasswordAuthentication = true;
        AllowUsers = [user];
        UseDns = true;
        X11Forwarding = false;
        PermitRootLogin = "prohibit-password";
      };
    };

    nixblitz.enable = true;
  };

  systemd = {
    services.make-home-bash_profile = {
      wantedBy = ["multi-user.target"];
      serviceConfig.Type = "oneshot";
      script = ''
        FILE="/home/${user}/.bash_profile"
        cat <<EOL > "$FILE"
        clear
        # https://patorjk.com/software/taag/#p=display&f=Epic&t=NixBlitz
        # https://patorjk.com/software/taag/#p=display&f=Ivrit&t=NixBlitz <-- current
        echo '   _   _ _      ____  _ _ _       '
        echo '  | \ | (_)_  _| __ )| (_) |_ ____'
        echo '  |  \| | \ \/ /  _ \| | | __|_  /'
        echo '  | |\  | |>  <| |_) | | | |_ / / '
        echo '  |_| \_|_/_/\_\____/|_|_|\__/___|'
        echo ""

        alias build_nixblitz="sudo nix build --no-update-lock-file --max-jobs 0 '/home/${user}/config/src#nixosConfigurations.nixblitzx86vm.config.system.build.toplevel'"
        alias inst_nixblitz_vda="sudo disko-install --flake '/home/${user}/config/src#nixblitzx86vm' --disk main /dev/vda"

        if [ ! -d "config" ]; then
          nixblitz init -w config
        fi
        nixblitz install
        EOL

        chown "${user}":"users" "$FILE"
        chmod 644 "$FILE"
      '';
    };
  };

  networking = {
    hostName = "nixblitz-installer";
    firewall.allowedTCPPorts = [22];
  };

  system.stateVersion = "25.05";
}
