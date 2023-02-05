{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    utils,
    ...
  }:
    utils.lib.eachDefaultSystem (system: let
      name = "rapla-to-ics";
      version = "0.2.0";

      inherit (nixpkgs) lib;

      pkgs = import nixpkgs {
        inherit system;
      };

      nativeBuildInputs = [
        pkgs.rustc
        pkgs.cargo
        pkgs.pkg-config
      ];

      buildInputs =
        [pkgs.openssl]
        ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
          frameworks.Security
        ]);
    in rec {
      packages.${name} = pkgs.rustPlatform.buildRustPackage {
        inherit name version nativeBuildInputs buildInputs;

        src = builtins.path {
          inherit name;
          path = ./.;
        };

        cargoLock.lockFile = ./Cargo.lock;
      };

      packages.default = packages.${name};

      packages."${name}-docker" = pkgs.dockerTools.buildLayeredImage {
        inherit name;
        tag = "latest";

        contents = [
          packages.default
          pkgs.cacert
        ];

        config = {
          Cmd = [name "serve"];
          ExposedPorts."8080/tcp" = {};
        };
      };

      formatter = pkgs.alejandra;

      devShells.default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;
        packages = [
          # rust
          pkgs.rustfmt
          pkgs.rust-analyzer
          pkgs.clippy
          pkgs.cargo-watch

          # nix
          formatter
          pkgs.nil
        ];
      };
    });

  nixConfig = {
    extra-substituters = [
      "https://systems.cachix.org"
    ];
    extra-trusted-public-keys = [
      "systems.cachix.org-1:w+BPDlm25/PkSE0uN9uV6u12PNmSsBuR/HW6R/djZIc="
    ];
  };
}
