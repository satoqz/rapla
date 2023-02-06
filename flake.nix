{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    nci,
    ...
  }: let
    inherit (nixpkgs) lib;

    name = "rapla-to-ics";

    deps = {
      buildInputs = pkgs:
        [
          pkgs.openssl
          pkgs.pkg-config
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
        ];

      nativeBuildInputs = pkgs: [
        pkgs.pkg-config
      ];

      shell = pkgs: [
        pkgs.rust-analyzer
        pkgs.cargo-watch
        pkgs.treefmt
        pkgs.nil
      ];
    };

    outputs = nci.lib.makeOutputs {
      root = ./.;

      pkgConfig = common: {
        ${name} = rec {
          overrides.libraries = {
            buildInputs = deps.buildInputs common.pkgs;
            nativeBuildInputs = deps.nativeBuildInputs common.pkgs;
          };

          depsOverrides.libraries = overrides.libraries;

          build = true;
          app = true;
        };
      };

      config = common: {
        outputs.defaults = {
          app = name;
          package = name;
        };

        shell.packages = deps.shell common.pkgs;
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
    extra-substituters = [
      "https://systems.cachix.org"
    ];
    extra-trusted-public-keys = [
      "systems.cachix.org-1:w+BPDlm25/PkSE0uN9uV6u12PNmSsBuR/HW6R/djZIc="
    ];
  };
}
