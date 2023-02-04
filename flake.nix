{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = {
    nixpkgs,
    nci,
    ...
  }: let
    inherit (nixpkgs) lib;

    name = "rapla-to-ics";

    outputs = nci.lib.makeOutputs {
      root = ./.;

      config = common: {
        outputs.defaults = {
          package = name;
          app = name;
        };

        shell.packages = with common.pkgs; [
          treefmt
          rust-analyzer
          nil
        ];

        cCompiler.package = common.pkgs.clang;
      };

      pkgConfig = common: {
        ${name} = {
          build = true;
          app = true;

          overrides.libraries.nativeBuildInputs = with common.pkgs;
            [libiconv]
            ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
              CoreFoundation
              Security
              SystemConfiguration
            ]);
        };
      };
    };
  in
    outputs
    // {
      packages = lib.mapAttrs (system: packages: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
        packages
        // lib.optionalAttrs pkgs.stdenv.isLinux {
          "${name}-docker" = pkgs.dockerTools.buildLayeredImage {
            inherit name;
            tag = "latest";

            contents = [
              packages.default
              pkgs.cacert
            ];

            config = {
              Cmd = [name];
              ExposedPorts."8080/tcp" = {};
            };
          };
        })
      outputs.packages;
    };

  nixConfig = {
    extra-substitutors = [
      "https://systems.cachix.org"
    ];
    extra-trusted-public-keys = [
      "systems.cachix.org-1:w+BPDlm25/PkSE0uN9uV6u12PNmSsBuR/HW6R/djZIc="
    ];
  };
}
