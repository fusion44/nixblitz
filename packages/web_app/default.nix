{
  pkgs,
  rustPlatform,
  basePath ? "",
}: let
  # for local development
  src = ./..;

  # src = pkgs.fetchgit {
  #   url = "https://forge.f44.fyi/f44/nixblitz";
  #   rev = "6243d7d0bd94279418f852d03aac29bf7641bb82";
  #   sha256 = "sha256-C5MgUaetAyhDjcDcmqczN7Pg7tdz2kcs7ZjmpVg0JOI=";
  # };

  # src = fetchFromGitHub {
  #   owner = "fusion44";
  #   repo = "nixblitz";
  #   rev = "1bc9027bdc32a8b7228c9dbcd707acf860163e67";
  #   sha256 = "sha256-ag6wM9C+lj/m6zeEp0W0inRWMgAm5dgbejsqKK9OXVE=";
  # };

  rustWorkspacePath = src; # + "/packages";
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
      wasm-bindgen-cli
      lld
    ];

    buildInputs = [pkgs.openssl];

    preBuild = ''
      local templatePath="web_app/Dioxus.toml.templ"
      local configTargetPath="web_app/Dioxus.toml"
      rm -f "$configTargetPath"

      if [ ! -f "$templatePath" ]; then
        echo "Error: Dioxus.toml.templ not found at $templatePath"
        exit 1
      fi

      cp "$templatePath" "$configTargetPath"

      echo "Working with the given base_path = \"${basePath}\""

      local basePathLineToInject=""
      if [ -n "${basePath}" ]; then
        # string is not empty
        basePathLineToInject="base_path = \"${basePath}\""
      fi

      echo "Updating $configTargetPath: replacing '%%DIOXUS_BASE_PATH_LINE%%' with '$basePathLineToInject'"
      # substituteInPlace "$configTargetPath" \
      #   --replace "%%DIOXUS_BASE_PATH_LINE%%" "$basePathLineToInject"
      substituteInPlace "$configTargetPath" \
        --replace "%%DIOXUS_BASE_PATH_LINE%%" ""

      echo "--- Patched $configTargetPath ---"
      cat "$configTargetPath"
      echo "---------------------------------"
    '';

    buildPhase = ''
      runHook preBuild

      cd web_app
      echo "Current directory for build: $(pwd)"

      echo "Running 'dx bundle --platform web'"
      dx bundle --platform web

      cd ..

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      mkdir -p $out
      local assetsSourceDir="target/dx/web_app/release/web"

      if [ ! -d "$assetsSourceDir" ]; then
        echo "Error: Built Dioxus assets not found at $assetsSourceDir!"
        # echo "Listing contents of web_app/target/dx if it exists:"
        # ls -R target/dx 2>/dev/null || echo "web_app/target/dx does not exist or is empty"
        exit 1
      fi

      echo "Copying assets from $assetsSourceDir to $out"
      cp -R "$assetsSourceDir"/* $out

      runHook postInstall
    '';
  }
