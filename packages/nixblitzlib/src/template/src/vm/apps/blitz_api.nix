{
  services.blitz-api = {
    enable = true;
    ln.connectionType = "lnd_grpc";
    # logLevel = "TRACE";
    dotEnvFile = "/var/lib/blitz_api/.env";
    # passwordFile = "/run/keys/login_password";
    rootPath = "/api";

    bitcoind = {
      rpc.address = "192.168.8.242";
    };

    nginx = {
      enable = true;
      openFirewall = true;
      location = "/api";
    };
  };
}
