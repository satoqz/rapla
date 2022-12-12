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
      pkgs = import nixpkgs {
        inherit system;
      };

      buildInputs =
        [pkgs.openssl]
        ++ nixpkgs.lib.optional pkgs.stdenv.isDarwin
        pkgs.darwin.apple_sdk.frameworks.SystemConfiguration;

      nativeBuildInputs = [
        pkgs.cargo
        pkgs.rustc
        pkgs.pkg-config
      ];
    in rec {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        name = "rapla-to-ics";
        version = "main";

        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        inherit buildInputs nativeBuildInputs;
      };

      apps.default = flake-utils.lib.mkApp {
        drv = packages.default;
      };

      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo-watch
          rust-analyzer
          rustfmt
          clippy
          alejandra
        ];

        inherit buildInputs nativeBuildInputs;
      };

      formatter = pkgs.alejandra;
    });
}
