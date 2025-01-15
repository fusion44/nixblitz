{
  lib,
  cfg,
  ...
}: {
  nix-bitcoin = {
    generateSecrets = true;
    operator = {
      enable = true;
      name = "admin";
    };
  };
}
