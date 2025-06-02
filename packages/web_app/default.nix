{
  pkgs,
  pkg-config,
  fetchFromGitHub,
  rustPlatform,
  openssl,
}: let
  src = pkgs.fetchgit {
    url = "https://forge.f44.fyi/f44/nixblitz";
    rev = "82d8b8d426d0a50ca15911641132739df43abc46";
    sha256 = "sha256-gmH0XTWWqKs0WFGqP7ML/A+eZDbpWy0E5f/UXlW/8r4=";
  };

  # src = fetchFromGitHub {
  #   owner = "fusion44";
  #   repo = "nixblitz";
  #   rev = "1bc9027bdc32a8b7228c9dbcd707acf860163e67";
  #   sha256 = "sha256-ag6wM9C+lj/m6zeEp0W0inRWMgAm5dgbejsqKK9OXVE=";
  # };

  rustWorkspacePath = src + "/packages";
in
  rustPlatform.buildRustPackage {
    pname = "web-app";
    version = "0.1.0";

    src = rustWorkspacePath;
    cargoLock.lockFile = "${rustWorkspacePath}/Cargo.lock";

    nativeBuildInputs = with pkgs; [
      rustPlatform.cargoSetupHook
      pkg-config
      cargo
      rustc
      dioxus-cli
      tree
      wasm-bindgen-cli
      lld
    ];

    buildInputs = [openssl];

    buildPhase = ''
      cd web_app
      dx bundle --platform web
      cd ..
    '';

    installPhase = ''
      runHook preInstall
      mkdir -p $out/bin
      echo "Copying binary to $out/bin"
      tree target/dx
      cp -r target/dx/web_app/release/web/* $out/bin
      runHook postInstall
    '';
  }
