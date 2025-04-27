{
  enable = false;
  regtest = false;
  txindex = false;
  disablewallet = true;
  listen = false;
  address = "127.0.0.1";
  port = 8333;
  rpc = {
    address = "127.0.0.1";
    port = 8332;
    allowip = [
    ];
    users = {
      "public" = {
        name = "public";
        passwordHMAC = "4ce7bb39c206211e9e601615a2deb379$9fe8f6e710c87d471dee7649dc47f596766892a89c71a18506d616b0111c27ce";
      };
    };
  };

  zmqpubrawblock = "tcp://127.0.0.1:28332";
  zmqpubrawtx = "tcp://127.0.0.1:28333";
}
