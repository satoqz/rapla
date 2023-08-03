{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [
      "aarch64-linux"
      "aarch64-darwin"
      "x86_64-linux"
      "x86_64-darwin"
    ];

    perSystem = { self', inputs', system, lib, pkgs, ... }:
      let
        name = "rapla-sync";

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        inherit (cargoToml.package) version;

        src = builtins.path {
          inherit name;
          path = toString ./.;
        };

        inherit (inputs'.fenix.packages) combine stable targets;

        defaultToolchain = stable.toolchain;

        muslTarget =
          if pkgs.stdenv.isAarch64 then "aarch64-unknown-linux-musl"
          else if pkgs.stdenv.isx86_64 then "x86_64-unknown-linux-musl"
          else throw "unreachable";

        muslToolchain = combine ([
          stable.rustc
          stable.cargo
          targets.${muslTarget}.stable.rust-std
        ]);

        naersk = pkgs.callPackage inputs.naersk;

        builderFor = toolchain: (naersk {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage;

        buildDefault = builderFor defaultToolchain;
        buildMusl = builderFor muslToolchain;

        inherit (self'.packages) rapla-sync;

        mkCheck = checkName: nativeBuildInputs: checkPhase:
          rapla-sync.overrideAttrs (final: prev: {
            name = "${name}-check-${checkName}";
            nativeBuildInputs = nativeBuildInputs ++ prev.nativeBuildInputs;

            dontBuild = true;
            inherit checkPhase;

            installPhase = ''
              mkdir -p $out
            '';
          });
      in
      {
        packages = {
          default = rapla-sync;

          rapla-sync = buildDefault {
            inherit name version src;
            buildInputs = lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
            ];
          };
        } // lib.optionalAttrs pkgs.stdenv.isLinux {
          rapla-sync-static = buildMusl {
            inherit name version src;
            CARGO_BUILD_TARGET = muslTarget;
          };

          docker-image = pkgs.dockerTools.buildLayeredImage {
            inherit name;
            tag = "latest";
            config.ExposedPorts."8080" = { };
            config.Cmd = [
              "${self'.packages.rapla-sync-static}/bin/rapla"
            ];
          };
        };

        checks = {
          rustfmt = mkCheck "rustfmt" [ stable.rustfmt ] ''
            cargo fmt --all -- --check
          '';

          clippy = mkCheck "clippy" [ stable.clippy ] ''
            cargo clippy
          '';
        };

        formatter = pkgs.nixpkgs-fmt;

        devShells.default = pkgs.mkShell {
          inputsFrom = [ rapla-sync ] ++ builtins.attrValues self'.checks;

          packages = [
            self'.formatter
            stable.rust-analyzer
            pkgs.cargo-watch
            pkgs.flyctl
          ];

          RUST_SRC_PATH = "${stable.rust-src}/lib/rustlib/rust/library";
        };
      };
  };
}
