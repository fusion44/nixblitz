{
  nix-bitcoin = {
    generateSecrets = true;
    operator = {
      enable = true;
      name = "admin";
    };
  };
  services.lnd = {
    enable = true;
    address = "0.0.0.0";
  };
}
