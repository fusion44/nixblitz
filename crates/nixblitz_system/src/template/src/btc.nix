{
  lib,
  config,
  ...
}: {
  options = {
    btc.enable = lib.mkEnableOption "Enable the bitcoin module";
  };

  config = lib.mkIf config.btc.enable {
    services = {
      bitcoind = import ./btc/bitcoind.nix;
      clightning = import ./btc/cln.nix;
      lnd = import ./btc/lnd.nix {inherit config;};
    };
  };
}
