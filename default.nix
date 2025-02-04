{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./packages/cli/Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.name;
    inherit (manifest) version;
    cargoLock.lockFile = ./packages/Cargo.lock;
    src = pkgs.lib.cleanSource ./packages;

    meta = {
      description = "Management CLI for the nixblitz project";
      homepage = "https://github.com/fusion44/nixblitz";
      license = pkgs.lib.licenses.mit;
      maintainers = ["fusion44"];
      mainProgram = "nixblitz";
    };
  }
