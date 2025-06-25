{
  pkgs ? import <nixpkgs> {},
  url ? "https://docs.f44.fyi",
  baseUrl ? "/",
}:
pkgs.buildNpmPackage {
  pname = "nixblitz-docs";
  version = "0.1.0";
  src = ./.;
  npmDepsHash = "sha256-+k1Eix8B7k0eD/GqQwgD1+P8ldYgM0WAjyXFi2gubHU=";

  installPhase = ''
    runHook preInstall

    # Set the URL environment variable before running the build script
    export URL="${url}"

    # Set the BASE_URL environment variable before running the build script
    export BASE_URL="${baseUrl}"

    echo "Building Docusaurus with BASE_URL (from env): $BASE_URL"

    npm run build

    if [ ! -d "build" ]; then
      echo "Error: Docusaurus 'build' directory not found after 'npm run build'!"
      echo "Check that docusaurus.config.js is correctly reading URL/BASE_URL and the build script is successful."
      exit 1
    fi

    cp -R build $out

    runHook postInstall
  '';

  meta = {
    description = "NixBlitz Documentation Site";
    homepage = "https://docs.f44.fyi";
    license = "MIT";
  };
}
