{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./packages/cli/Cargo.toml).package;
  rev = "a2dc1b51881efdbb52f6354d51c7a253c3d31fb2";
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
