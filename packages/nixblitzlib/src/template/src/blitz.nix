{
  lib,
  config,
  ...
}: {
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
