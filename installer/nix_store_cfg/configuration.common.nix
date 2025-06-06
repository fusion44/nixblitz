{
  lib,
  pkgs,
  ...
}: let
in {
  imports = [
    ./nix-bitcoin/secure-node.nix
    "${builtins.fetchTarball {
      url = "https://github.com/nix-community/disko/archive/refs/tags/v1.11.0.tar.gz";
      sha256 = "sha256:13brimg7z7k9y36n4jc1pssqyw94nd8qvgfjv53z66lv4xkhin92";
    }}/module.nix"
    ./disko-config-single-disk.nix
  ];

  boot.loader.grub.enable = false;

  nixpkgs.config.allowUnfree = false;
  time.timeZone = "America/New_York";
  i18n.defaultLocale = "en_US.UTF-8";

  nix = {
    settings = {
      experimental-features = "nix-command flakes";
      auto-optimise-store = true;

      extra-trusted-substituters = [
        "http://192.168.8.202/cache"
      ];
      extra-trusted-public-keys = [
        "localhost:YYTdZJwWoH5/wtNI1gxWkpG0wRz0Kgpeo/fCfGnqlj4="
      ];
    };
  };

  console = {
    font = "Lat2-Terminus16";
    useXkbConfig = true;
  };

  users = {
    defaultUserShell = pkgs.nushell;
    users."admin" = {
      extraGroups = ["wheel"];
      hashedPassword = "$6$rounds=10000$moY2rIPxoNODYRxz$1DESwWYweHNkoB6zBxI3DUJwUfvA6UkZYskLOHQ9ulxItgg/hP5CRn2Fr4iQGO7FE16YpJAPMulrAuYJnRC9B.";
      openssh.authorizedKeys.keys = [];
    };
  };

  home-manager.users."admin" = {pkgs, ...}: {
    programs.nushell = {
      enable = true;
      configFile.source = ./configs/nushell/config.nu;
      envFile.source = ./configs/nushell/env.nu;
    };

    home = {
      packages = [];
      stateVersion = "25.05";
    };
  };

  environment = {
    sessionVariables = {
      EDITOR = "nvim";
      VISUAL = "nvim";
      NIXBLITZ_WORK_DIR = "/mnt/data/config";
    };

    systemPackages = with pkgs; [
      bat
      bottom
      fzf
      git
      neovim
      ripgrep
      bandwhich
      just
      superfile
    ];
  };

  nix-bitcoin = {
    generateSecrets = true;
    operator = {
      enable = true;
      name = "admin";
    };
  };

  services = {
    blitz-api = {
      enable = true;
      nginx = {
        enable = true;
      };
    };

    blitz-web = {
      enable = true;
      nginx = {
        enable = false;
      };
    };

    nixblitz.enable = true;
    bitcoind = {
      enable = true;
      regtest = true;
    };
    clightning.enable = true;
    lnd = {
      enable = true;
      port = 9999;
    };

    openssh = {
      enable = true;
      ports = [22];
      settings = {
        PasswordAuthentication = false;
        AllowUsers = ["admin"];
        UseDns = true;
        X11Forwarding = false;
        PermitRootLogin = "prohibit-password";
      };
    };

    redis.servers."".enable = true;
  };

  networking.firewall.allowedTCPPorts = [
    22
  ];
  system.stateVersion = "25.05";
}
