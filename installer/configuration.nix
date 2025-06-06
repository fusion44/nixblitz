{
  lib,
  pkgs,
  inputs,
  ...
}: let
  user = "nixos";
  isoLabel = "NIXBLITZ";
  targetSystemConfig = inputs.targetSystem.nixosConfigurations.nixblitzx86vm;
  initConfigPath = "/tmp/config";
in {
  boot.loader.grub.enable = false;

  nixpkgs.config.allowUnfree = false;
  time.timeZone = "Europe/Berlin";
  i18n.defaultLocale = "en_US.UTF-8";

  system.nixos.label = isoLabel;
  boot.kernelParams = ["iso_label=${isoLabel}"];
  isoImage = {
    volumeID = isoLabel;
    storeContents = with pkgs; [
      targetSystemConfig.config.system.build.toplevel
      inputs.nixblitz.outputs.packages.x86_64-linux.nixblitz
      inputs.nixblitz.outputs.packages.x86_64-linux.nixblitz-webapp
      inputs.blitz-api.outputs.packages.x86_64-linux.default
      # inputs.blitz-web.outputs.packages.x86_64-linux.default
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

  environment = {
    sessionVariables = {
      EDITOR = "nvim";
      VISUAL = "nvim";
      NIXBLITZ_WORK_DIR = "${initConfigPath}";
    };

    systemPackages = with pkgs; [
      bat
      fzf
      just
      disko
      bottom
      neovim
      nushell
      lazygit
      ripgrep
      bandwhich
      superfile
      dioxus-cli
      nixos-anywhere
    ];

    etc = {
      "nushell/dev_scripts.nu" = {
        source = ./tools/dev_scripts.nu;
      };
    };
  };

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
    nixblitz-webapp = {
      enable = true;
      dataDir = initConfigPath;
      nginx = {
        enable = true;
        openFirewall = true;
        location = "/";
      };
    };
    nixblitz-docs = {
      enable = true;
      url = "https://docs.f44.fyi";
      nginx = {
        enable = true;
        openFirewall = true;
        location = "/docs";
      };
    };
  };

  programs = {
    git = {
      enable = true;
      config = {
        core.editor = "nvim";
        user.name = "nixblitz";
        user.email = "nixblitz";
        init.defaultBranch = "main";
      };
    };
  };

  systemd = {
    services.make-home-bash_profile = {
      wantedBy = ["multi-user.target"];
      serviceConfig.Type = "oneshot";
      script = ''
        FILE="/home/${user}/.bash_profile"
        cat <<EOL > "$FILE"

        if [ ! -d "${initConfigPath}" ]; then
          sudo mkdir -p /mnt/data/lnd
          sudo mkdir -p /mnt/data/clightning
          sudo mkdir -p /mnt/hdd/bitcoind
          nixblitz init -w ${initConfigPath}
          # DIRTY HACK: find out why this is necessary; the lock file in the
          #             template is not properly updated
          cd ${initConfigPath}/src
          sleep 3s
          nix flake update nixblitz
          cd ${initConfigPath} && git init && git add --all && git commit -m "init"
          cd ~
          sudo chmod -R 777 ${initConfigPath}
        fi

        clear

        # https://patorjk.com/software/taag/#p=display&f=Epic&t=NixBlitz
        # https://patorjk.com/software/taag/#p=display&f=Ivrit&t=NixBlitz <-- current
        echo '   _   _ _      ____  _ _ _       '
        echo '  | \ | (_)_  _| __ )| (_) |_ ____'
        echo '  |  \| | \ \/ /  _ \| | | __|_  /'
        echo '  | |\  | |>  <| |_) | | | |_ / / '
        echo '  |_| \_|_/_/\_\____/|_|_|\__/___|'
        echo ""
        echo "Working directory: ${initConfigPath}"

        alias build_nixblitz="sudo nix build -vvv --no-update-lock-file --max-jobs 0 '${initConfigPath}/src#nixosConfigurations.nixblitzx86.config.system.build.toplevel'"
        alias inst_nixblitz_vda="sudo disko-install --flake '/home/${user}/config/src#nixblitzx86' --disk main /dev/vda"
        alias test_remote_build="sudo ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no -i /root/.ssh/remotebuild remotebuild@192.168.8.202"
        alias sync_config="sudo mkdir -p /mnt/data && sudo mount /dev/vda3 /mnt/data && sudo rsync -av --delete ${initConfigPath} /mnt/data/config && sudo chown -R 1000:100 /mnt/data/config"
        alias nu="nu -e 'source /etc/nushell/dev_scripts.nu'"

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
