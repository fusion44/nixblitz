# default.nix
{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./packages/cli/Cargo.toml).package;
  commitSha = "4f940cecdc0a4eb31e83919b1b7adb03aeed2eb0";
  shortSha = builtins.substring 0 7 commitSha;
  src = pkgs.fetchgit {
    url = "https://forge.f44.fyi/f44/nixblitz";
    rev = commitSha;
    sha256 = "sha256-oRa2TqtxzEhUftLWspdQBbeYSyE3l+MtQn7odGksgJo=";
  };

  # src = pkgs.fetchFromGitHub {
  #   owner = "fusion44";
  #   repo = "nixblitz";
  #   rev = commitSha;
  #   sha256 = "";
  # };

  crateSource = src + "/packages";
  vergenGitSha = commitSha;
  vergenGitDescribe = "${shortSha}-nix";
  vergenGitDirty = "false";

  vergenSourceDateEpoch = "0";
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.name;
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
      mainProgram = manifest.name;
    };
  }
