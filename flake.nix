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
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        name = "rapla-to-ics";
        version = "main";

        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        buildInputs =
          nixpkgs.lib.optional pkgs.stdenv.isDarwin
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration;
      };
    });
}
