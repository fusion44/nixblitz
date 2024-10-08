{
  services.blitz-api = {
    enable = true;
    ln.connectionType = "lnd_grpc";
    # logLevel = "TRACE";
    dotEnvFile = "/var/lib/blitz_api/.env";
    # passwordFile = "/run/keys/login_password";
    rootPath = "/api";
    nginx = {
      enable = true;
      openFirewall = true;
      location = "/api";
    };
  };
}
