{
  lib,
  config,
  ...
}: let
  configPath = "/mnt/data/config";
  webAppPort = 8080;
in {
  options = {
    blitz.enable = lib.mkEnableOption "Enable the blitz module";
  };
  config = lib.mkIf config.blitz.enable {
    services = {
      nixblitz = {
        # always enabled
        enable = true;
      };
      nixblitz-webapp = {
        # always enabled
        enable = true;
        dataDir = configPath;
        nginx = {
          enable = true;
          openFirewall = true;
          location = "/";
        };
      };
      blitz-api = import ./blitz/api.nix;
      blitz-web = import ./blitz/web.nix;
    };
  };
}
