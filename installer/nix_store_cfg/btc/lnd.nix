{config, ...}: {
  enable = false;
  address = "127.0.0.1";
  port = 9735;
  rpcAddress = "127.0.0.1";
  rpcPort = 10009;
  restAddress = "127.0.0.1";
  restPort = 8080;
  dataDir = "/var/lib/lnd";
  certificate = {
    extraIPs = [
    ];
    extraDomains = [
    ];
  };
  extraConfig = ''

  '';
}
