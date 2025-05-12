{
  lib,
  pkgs,
  inputs,
  ...
}: let
  user = "nixos";
  isoLabel = "NIXBLITZ";
  targetSystemConfig = inputs.targetSystem.nixosConfigurations.nixblitzx86vm;
in {
  boot.loader.grub.enable = false;

  nixpkgs.config.allowUnfree = false;
  time.timeZone = "Europe/Berlin";
  i18n.defaultLocale = "en_US.utf8";

  system.nixos.label = isoLabel;
  boot.kernelParams = ["iso_label=${isoLabel}"];
  isoImage = {
    volumeID = isoLabel;
    storeContents = with pkgs; [
      targetSystemConfig.config.system.build.toplevel
      bitcoind
      clightning
      lnd
      lndinit
      lightning-loop
      lightning-pool
      lightning-terminal
    ];
  };

  nix = {
    settings = {
      experimental-features = "nix-command flakes";
      # TODO: fix this
      # uncommenting this will force the installer to use the local nix store only,
      # problem at the moment is that some paths are not included in the storeContents
      # builders-use-substitutes = false;
      # substituters = lib.mkForce [];

      # extra-trusted-substituters = [
      #   "http://192.168.8.202/cache"
      # ];
      # extra-trusted-public-keys = [
      #   "localhost:YYTdZJwWoH5/wtNI1gxWkpG0wRz0Kgpeo/fCfGnqlj4="
      # ];
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
    bandwhich
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
        BLITZ_CONFIG_PATH="/home/${user}/config"

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

        alias build_nixblitz="sudo nix build -vvv --no-update-lock-file --max-jobs 0 '$BLITZ_CONFIG_PATH/src#nixosConfigurations.nixblitzx86.config.system.build.toplevel'"
        alias inst_nixblitz_vda="sudo disko-install --flake '/home/${user}/config/src#nixblitzx86' --disk main /dev/vda"
        alias test_remote_build="sudo ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no -i /root/.ssh/remotebuild remotebuild@192.168.8.202"
        alias sync_config="sudo mkdir -p /mnt/data && sudo mount /dev/vda3 /mnt/data && sudo rsync -av --delete /home/${user}/config/ /mnt/data/config && sudo chown -R 1000:100 /mnt/data/config"

        if [ ! -d "config" ]; then
          sudo mkdir -p /mnt/data/lnd
          sudo mkdir -p /mnt/data/clightning
          sudo mkdir -p /mnt/hdd/bitcoind
          nixblitz init -w $BLITZ_CONFIG_PATH
        fi
        nixblitz install -w $BLITZ_CONFIG_PATH
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
