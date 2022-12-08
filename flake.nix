{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in rec {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        name = "rapla-to-ics";
        version = "main";

        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        buildInputs =
          [pkgs.openssl]
          ++ nixpkgs.lib.optional pkgs.stdenv.isDarwin
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration;

        nativeBuildInputs = [pkgs.pkg-config];
      };

      apps.default = flake-utils.lib.mkApp {drv = packages.default;};
    });
}
