{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./packages/cli/Cargo.toml).package;
  rev = "1550131b2749e22c5d30d55d8c871b0ae8dedad1";
  src = pkgs.fetchFromGitHub {
    owner = "fusion44";
    repo = "nixblitz";
    inherit rev;
    sha256 = "sha256-uVUzz7Msjfox9SZloyTaDANBrockaeTGCxDnrdx5PT4=";
  };

  crateSource = src + "/packages";
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.name;
    inherit (manifest) version;
    src = crateSource;

    cargoLock.lockFile = crateSource + "/Cargo.lock";
    VERGEN_GIT_SHA = rev;

    meta = {
      description = manifest.description or "Management CLI for the nixblitz project";
      homepage = manifest.homepage or "https://github.com/fusion44/nixblitz";
      license = pkgs.lib.licenses.mit; # Assuming MIT, adjust if different
      maintainers = ["fusion44"]; # Add your Nix maintainer handle if desired
      mainProgram = manifest.name; # The name of the binary produced
    };
  }
