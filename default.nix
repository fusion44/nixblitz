# default.nix
{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./crates/nixblitz_cli/Cargo.toml).package;
  commitSha = "1013dd3c929096dfc3ff7ade21ac20f1faa1cc43";
  shortSha = builtins.substring 0 7 commitSha;
  # src = ./.;

  src = pkgs.fetchgit {
    url = "https://forge.f44.fyi/f44/nixblitz";
    rev = commitSha;
    sha256 = "sha256-2pjT4PJ2L1BGQNDCVZuBm4Gp81DMc217bR8KPm1PAj0=";
  };

  # src = pkgs.fetchFromGitHub {
  #   owner = "fusion44";
  #   repo = "nixblitz";
  #   rev = commitSha;
  #   sha256 = "";
  # };

  crateSource = src + "/crates";
  vergenGitSha = commitSha;
  vergenGitDescribe = "${shortSha}-nix";
  vergenGitDirty = "false";

  vergenSourceDateEpoch = "0";
in
  pkgs.rustPlatform.buildRustPackage {
    pname = "nixblitz";
    inherit (manifest) version;
    src = crateSource;

    cargoLock.lockFile = crateSource + "/Cargo.lock";
    VERGEN_GIT_SHA = vergenGitSha;
    VERGEN_GIT_DESCRIBE = vergenGitDescribe;
    VERGEN_GIT_DIRTY = vergenGitDirty;
    SOURCE_DATE_EPOCH = vergenSourceDateEpoch;

    meta = {
      description = manifest.description or "Management CLI for the nixblitz project";
      homepage = manifest.homepage or "https://github.com/fusion44/nixblitz";
      license = pkgs.lib.licenses.mit;
      maintainers = ["fusion44"];
      mainProgram = "nixblitz";
    };
  }
