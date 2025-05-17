# default.nix
{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./packages/cli/Cargo.toml).package;
  commitSha = "eb1c9a5e2928982bf25d91cab4842e8fa6608b8b";
  shortSha = builtins.substring 0 7 commitSha;

  src = pkgs.fetchFromGitHub {
    owner = "fusion44";
    repo = "nixblitz";
    rev = commitSha;
    sha256 = "sha256-3gs7+A/B3pWIvmBG9vK1/eKf+3FQGnFL/4whxN3ToTs=";
  };

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
