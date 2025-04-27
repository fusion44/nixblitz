{
  enable = false;
  address = "127.0.0.1";
  port = 9735;
  proxy = null;
  always-use-proxy = false;
  dataDir = /var/lib/clightning;
  wallet = "sqlite3:///var/lib/clightning/bitcoin/lightningd.sqlite3";
  extraConfig = ''

  '';
  user = "admin";
  group = "cfg.user";
  getPublicAddressCmd = "";
}
