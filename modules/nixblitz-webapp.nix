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
  userConfiguredLocation = cfg.nginx.location;
  servingLocationPath =
    if userConfiguredLocation == "/"
    then "/"
    else if lib.strings.hasSuffix "/" userConfiguredLocation
    then userConfiguredLocation
    else userConfiguredLocation + "/";

  redirectSourcePath =
    if userConfiguredLocation != "/" && !(lib.strings.hasSuffix "/" userConfiguredLocation)
    then userConfiguredLocation
    else null;

  # https://dioxuslabs.com/learn/0.6/CLI/configure/
  dioxusTomlValueForBasePath =
    if servingLocationPath == "/"
    then "" # Dioxus wants a dot or an unset value for root (info from Discord)
    else
      # For servingLocationPath like "/app/" or "/foo/bar/", Dioxus wants "app" or "foo/bar"
      # 1. Remove leading slash: "app/" or "foo/bar/"
      # 2. Remove trailing slash: "app" or "foo/bar"
      let
        noLeadingSlash = lib.strings.removePrefix "/" servingLocationPath;
      in
        lib.strings.removeSuffix "/" noLeadingSlash;

  appPackage = pkgs.callPackage ../packages/web_app/default.nix {
    basePath = dioxusTomlValueForBasePath;
  };
in {
  options = {
    services.${name} = {
      enable = mkEnableOption "${name}";

      package = mkOption {
        type = types.package;
        defaultText = literalExpression "pkgs.${name}";
        default = appPackage;
        description = "The ${name} package to use.";
      };

      dataDir = mkOption {
        type = types.path;
        default = "/mnt/data/config";
        example = "/path/to/config";
        description = "The config that will be edited by default";
      };

      server = {
        port = mkOption {
          type = types.port;
          default = 8080;
          description = "The internal port for the Dioxus server to bind to.";
        };

        host = mkOption {
          type = types.str;
          default = "127.0.0.1";
          description = "The internal host address for the Dioxus server to bind to.";
        };

        environment = mkOption {
          type = types.attrsOf types.str;
          default = {};
          description = "Environment variables to set for the Dioxus server process.";
          example = literalExpression ''
            {
              DATABASE_URL = "your-db-connection-string";
              # ROCKET_ADDRESS = cfg.services.${name}.server.host; # If using Rocket and need to set explicitly
              # ROCKET_PORT = toString cfg.services.${name}.server.port;
            }
          '';
        };

        user = mkOption {
          type = types.str;
          default = defaultUser;
          example = "${defaultUser}";
          description = "The user to run the server service as";
        };

        group = mkOption {
          type = types.str;
          default = defaultGroup;
          description = "Group to run the server service as";
        };

        logLevel = lib.mkOption {
          type = types.enum ["TRACE" "DEBUG" "INFO" "SUCCESS" "WARNING" "ERROR" "CRITICAL"];
          default = "INFO";
          description = "Log level for the server";
          example = "DEBUG";
        };
      };

      nginx = {
        enable = mkEnableOption "Whether to enable nginx server for ${name}.";

        hostName = mkOption {
          type = types.str;
          example = "my.node.net";
          default = "localhost";
          description = "The hostname to use for the nginx virtual host.";
        };

        location = mkOption {
          type = types.str;
          example = "/app";
          default = "/";
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
    users.users = mkIf (cfg.server.user == defaultUser) {
      ${defaultUser} = {
        description = "${name} service user";
        inherit (cfg.server) group;
        isSystemUser = true;
      };
    };

    users.groups = mkIf (cfg.server.group == defaultGroup) {
      ${defaultGroup} = {};
    };

    systemd = {
      services.${name} = {
        wantedBy = ["multi-user.target"];
        description = "${name} server daemon";
        environment = {
          IP = cfg.server.host;
          PORT = toString cfg.server.port;
          NIXBLITZ_WORK_DIR = cfg.dataDir;
        };
        serviceConfig = {
          ExecStart = "${cfg.package}/server";
          User = cfg.server.user;
          Group = cfg.server.group;
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
        locations =
          {
            "${servingLocationPath}" = {
              proxyPass = "http://${cfg.server.host}:${toString cfg.server.port}/";
              proxyWebsockets = true;
              recommendedProxySettings = true;
              extraConfig = ''
                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                proxy_set_header X-Forwarded-Proto $scheme;
              '';
            };
          }
          // (
            if redirectSourcePath != null
            then {
              "${redirectSourcePath}" = {
                return = "301 $scheme://$http_host${servingLocationPath}";
              };
            }
            else {}
          );
      };
    };

    networking.firewall = mkIf cfg.nginx.openFirewall {
      allowedTCPPorts = [80];
    };
  };
}
