{
  enable = true;
  ln.connectionType = "none";
  logLevel = "INFO";
  generateDotEnvFile = true;
  dotEnvFile = "/etc/blitz_api/env";
  passwordFile = null;
  rootPath = "/api";
  nginx = {
    enable = false;
    openFirewall = false;
    location = "/";
  };
}
