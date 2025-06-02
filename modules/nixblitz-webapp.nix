{
  config,
  pkgs,
  lib,
  ...
}: let
  defaultUser = "nixblitz_webapp";
  defaultGroup = defaultUser;
  name = "nixblitz-webapp";

  cfg = config.services.${name};

  inherit (lib) mkOption mkIf mkEnableOption types literalExpression;
in {
  options = {
    services.${name} = {
      enable = mkEnableOption "${name}";

      package = mkOption {
        type = types.package;
        defaultText = literalExpression "pkgs.${name}";
        default = pkgs.${name};
        description = "The ${name} package to use.";
      };

      host = mkOption {
        type = types.str;
        default = "127.0.0.1";
        example = "127.0.0.1";
        description = "The host to bind to";
      };

      port = mkOption {
        type = types.port;
        default = 2121;
        example = 2121;
        description = "The port the ${name} will be listening on";
      };

      user = mkOption {
        type = types.str;
        default = defaultUser;
        example = "${defaultUser}";
        description = "The user to run the ${name} as";
      };

      group = mkOption {
        type = types.str;
        default = defaultGroup;
        description = "Group to run the ${name} as";
      };

      dataDir = mkOption {
        type = types.path;
        default = "/mnt/hdd/config";
        example = "/mnt/hdd/config";
        description = "The config that will be edited";
      };

      logLevel = lib.mkOption {
        type = types.enum ["TRACE" "DEBUG" "INFO" "SUCCESS" "WARNING" "ERROR" "CRITICAL"];
        default = "INFO";
        description = "Log level for the ${name}";
        example = "DEBUG";
      };

      nginx = {
        enable = mkEnableOption "Whether to enable nginx server for ${name}.";
        description = "This is used to generate the nginx configuration.";

        hostName = mkOption {
          type = types.str;
          example = "my.node.net";
          default = "localhost";
          description = "The hostname to use for the nginx virtual host.";
        };

        location = mkOption {
          type = types.str;
          example = "/app";
          default = "/app";
          description = "The location to serve the ${name} from from.";
        };

        openFirewall = mkOption {
          type = types.bool;
          default = false;
          description = "Whether to open the ports used by ${name} in the firewall for the server";
        };
      };
    };
  };

  config = mkIf cfg.enable {
    users.users = mkIf (cfg.user == defaultUser) {
      ${defaultUser} = {
        description = "${name} service user";
        inherit (cfg) group;
        isSystemUser = true;
      };
    };

    users.groups = mkIf (cfg.group == defaultGroup) {
      ${defaultGroup} = {};
    };

    systemd = {
      services.${name} = {
        wantedBy = ["multi-user.target"];
        description = "${name} server daemon";
        environment = {
          IP = cfg.host;
          PORT = toString cfg.port;
          NIXBLITZ_WORK_DIR = cfg.dataDir;
        };
        serviceConfig = {
          ExecStart = "${cfg.package}/bin/server";
          User = cfg.user;
          Group = cfg.group;
          Restart = "always";
          RestartSec = "5s";
          StartLimitBurst = 5;
          StartLimitIntervalSec = "10min";
          ReadWritePaths = [cfg.dataDir];
        };
      };
    };

    services.nginx = mkIf cfg.nginx.enable {
      enable = true;
      virtualHosts.${cfg.nginx.hostName} = {
        forceSSL = false;
        enableACME = false;

        locations."${cfg.nginx.location}" = {
          extraConfig = ''
            rewrite ${cfg.nginx.location}/(.*) /$1  break;
            proxy_redirect off;
          '';
          proxyPass = "http://${cfg.host}:${toString cfg.port}/";
          recommendedProxySettings = true;
        };
      };
    };

    networking.firewall = mkIf cfg.nginx.openFirewall {
      allowedTCPPorts = [80];
    };
  };
}
