# ./modules/nixblitz-docs.nix
{
  config,
  pkgs,
  lib,
  ...
}: let
  name = "nixblitz-docs";
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

  docsPackage = pkgs.callPackage ../docs/default.nix {
    inherit (cfg) url;
    baseUrl = servingLocationPath;
  };
in {
  options = {
    services.${name} = {
      enable = mkEnableOption "${name}";

      package = mkOption {
        type = types.package;
        default = docsPackage;
        defaultText = literalExpression "Docusaurus site automatically built with baseUrl from nginx.location";
        description = "The ${name} package, built with the appropriate baseUrl matching the Nginx configuration. Do not override unless you know what you are doing.";
        readOnly = true;
      };

      url = mkOption {
        type = types.str;
        example = "https://docs.mynode.net";
        default = "https://docs.f44.fyi";
        description = "The url to use for the Docusaurus site.";
      };

      nginx = {
        enable = mkEnableOption "Whether to enable nginx server for ${name}.";
        description = "This is used to generate the nginx configuration.";

        hostName = mkOption {
          type = types.str;
          example = "docs.mynode.net";
          default = "localhost";
          description = "The hostname to use for the nginx virtual host.";
        };

        location = mkOption {
          type = types.str;
          example = "/docs";
          default = "/docs";
          description = "The base URI path from which ${name} will be served (e.g., /docs). This determines the Docusaurus baseUrl.";
        };

        openFirewall = mkOption {
          type = types.bool;
          default = false;
          description = "Whether to open the Nginx ports in the firewall.";
        };
      };
    };
  };

  config = mkIf cfg.enable {
    services.nginx = mkIf cfg.nginx.enable {
      enable = true;
      virtualHosts.${cfg.nginx.hostName} = {
        forceSSL = false;
        enableACME = false;
        locations =
          {
            "${servingLocationPath}" = {
              alias = "${cfg.package}/";
              index = "index.html";
              tryFiles = "$uri /index.html =404";
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
