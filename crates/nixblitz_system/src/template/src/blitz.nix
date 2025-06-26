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
      nixblitz-cli = {
        # always enabled
        enable = true;
      };
      nixblitz-system-engine = {
        # always enabled
        enable = true;
        # server = {
        #   user = "root";
        #   group = "root";
        # };
        nginx = {
          enable = true;
          openFirewall = true;
          location = "/system";
        };
      };
      nixblitz-norupo = {
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
