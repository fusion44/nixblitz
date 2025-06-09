{
  sudo = "doas";
  cp = "cp -i";
  ll = "ls -l";
  lla = "ls -la";
  mv = "mv -i";
  nv = "nvim";
  ports = "netstat -tulanp";
  readlink = "readlink -f";
  apijournal = "journalctl -u blitz-api.service";
  apijournalf = "journalctl -u blitz-api.service -f";
  apijournalenvservice = "journalctl -u blitz-api-setup-env.service";
  apijournalenvservicef = "journalctl -u blitz-api-setup-env.service";
  apibatdotenv = "sudo bat /var/lib/blitz_api/.env";
  bitcoindjournal = "journalctl -u bitcoind.service";
  bitcoindjournalf = "journalctl -u bitcoind.service -f";
  bitcoindbatcfg = "sudo bat /var/lib/bitcoind/bitcoin.conf";
  nginxjournal = "journalctl -u nginx.service";
  nginxjournalf = "journalctl -u nginx.service -f -n 50";
}
