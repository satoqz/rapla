{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, nci, ... }:

    let
      inherit (nixpkgs) lib;

      buildInputs = pkgs: lib.optionals pkgs.stdenv.isDarwin [
        pkgs.libiconv
        pkgs.darwin.apple_sdk.frameworks.Security
        pkgs.darwin.apple_sdk.frameworks.CoreFoundation
      ];

      shellInputs = pkgs: [
        pkgs.rust-analyzer
        pkgs.cargo-watch
        pkgs.nil
      ];

      outputs = nci.lib.makeOutputs {
        root = ./.;

        pkgConfig = common: {
          rapla.build = true;
          rapla.app = true;
          rapla.overrides.libraries.buildInputs = buildInputs common.pkgs;
          rapla.depsOverrides.libraries.buildInputs = buildInputs common.pkgs;
        };

        config = common: {
          outputs.defaults = {
            app = "rapla";
            package = "rapla";
          };

          shell.packages = shellInputs common.pkgs;
        };
      };
    in

    outputs // {
      packages = lib.mapAttrs
        (system: packages:
          let
            pkgs = import nixpkgs { inherit system; };
          in
          packages // lib.optionalAttrs pkgs.stdenv.isLinux {
            "rapla-docker" = pkgs.dockerTools.buildLayeredImage {
              name = "rapla-to-ics";
              tag = "latest";
              config = {
                Cmd = [ "${packages.rapla}/bin/rapla" "serve-ics" ];
                ExposedPorts."8080/tcp" = { };
              };
            };
          })
        outputs.packages;
    };

  nixConfig = {
    extra-substituters = [
      "https://systems.cachix.org"
    ];
    extra-trusted-public-keys = [
      "systems.cachix.org-1:w+BPDlm25/PkSE0uN9uV6u12PNmSsBuR/HW6R/djZIc="
    ];
  };
}
