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

      config = common:
        let
          inherit (common) pkgs;
        in
        {
          outputs.defaults = {
            app = "rapla";
            package = "rapla";
          };

          shell.packages = [
            pkgs.rust-analyzer
            pkgs.cargo-watch
            pkgs.nil
          ];
        };

      pkgConfig = common:
        let
          inherit (common.pkgs) stdenv darwin;
        in
        {
          rapla.overrides.libraries.buildInputs =
            lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security;

          rapla.build = true;
          rapla.app = true;
        };

      outputs = nci.lib.makeOutputs {
        inherit config pkgConfig;
        root = ./.;
      };

      withDockerImage = (system: packages:
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
        });
    in

    outputs // {
      packages = lib.mapAttrs withDockerImage outputs.packages;
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
