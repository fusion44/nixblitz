# default.nix
{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
  commitSha = "7f30615497ddad479969f13fe2d52654fdae9236";
  shortSha = builtins.substring 0 7 commitSha;
  # src = ../../.;

  src = pkgs.fetchgit {
    url = "https://forge.f44.fyi/f44/nixblitz";
    rev = commitSha;
    sha256 = "sha256-/1Gwjjun/4L09zPchqTWSojWm40B+9B8d3xcHF7XoUQ=";
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

    buildPhase = ''
      runHook preBuild

      echo "Building the Installer Engine"
      cargo build --release --workspace --exclude nixblitz_norupo

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      cargo install --root $out --path nixblitz_cli
      cargo install --root $out --path nixblitz_installer_engine

      runHook postInstall
    '';

    meta = {
      description = manifest.description or "Management CLI for the nixblitz project";
      homepage = manifest.homepage or "https://github.com/fusion44/nixblitz";
      license = pkgs.lib.licenses.mit;
      maintainers = ["fusion44"];
      mainProgram = "nixblitz";
    };
  }
