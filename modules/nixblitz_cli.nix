{
  config,
  pkgs,
  lib,
  ...
}: let
  name = "nixblitz-cli";
  cfg = config.services.${name};

  inherit (lib) mkOption mkIf types literalExpression;
in {
  options = {
    services.${name} = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable the nixblitz-cli package.";
      };

      package = mkOption {
        type = types.package;
        defaultText = literalExpression "pkgs.${name}";
        default = pkgs.${name};
        description = "The ${name} package to use.";
      };
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      nixblitz-cli
    ];
  };
}
