{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, naersk }:
    let
      inherit (nixpkgs) lib;

      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      withPackages = f: (lib.genAttrs systems)
        (system: f nixpkgs.legacyPackages.${system});

      name = "rapla-sync";
      version = "0.1.1";

      src = builtins.path {
        inherit name;
        path = toString ./.;
      };
    in
    {
      packages = withPackages (pkgs:
        let
          inherit (pkgs) system;
          inherit (fenix.packages.${system}) combine stable targets;

          rapla-sync = (pkgs.callPackage naersk {
            cargo = stable.toolchain;
            rustc = stable.toolchain;
          }).buildPackage {
            inherit name version src;
            buildInputs = lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
            ];
          };

          rapla-sync-static =
            let
              target =
                if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
                else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
                else throw "unreachable";

              toolchain = combine ([
                stable.rustc
                stable.cargo
                targets.${target}.stable.rust-std
              ]);
            in
            (pkgs.callPackage naersk {
              cargo = toolchain;
              rustc = toolchain;
            }).buildPackage {
              inherit name version src;
              CARGO_BUILD_TARGET = target;
            };

          docker-image = pkgs.dockerTools.buildLayeredImage {
            inherit name;
            tag = "latest";
            config.Cmd = [ "${rapla-sync-static}/bin/rapla" ];
            config.ExposedPorts."8080" = { };
          };
        in
        {
          inherit rapla-sync;
          default = rapla-sync;
        } // lib.optionalAttrs pkgs.stdenv.isLinux {
          inherit rapla-sync-static docker-image;
        });

      checks = withPackages (pkgs:
        let
          inherit (pkgs) system;
          inherit (self.packages.${system}) rapla-sync;
          inherit (fenix.packages.${system}) stable;

          mkCheck = name: nativeBuildInputs: checkPhase:
            rapla-sync.overrideAttrs (final: prev: {
              name = "check-${name}";
              dontBuild = true;

              nativeBuildInputs = nativeBuildInputs
                ++ prev.nativeBuildInputs;

              inherit checkPhase;

              installPhase = ''
                mkdir -p $out
              '';
            });
        in
        {
          rustfmt = mkCheck "rustfmt" [ stable.rustfmt ] ''
            cargo fmt --all -- --check
          '';

          clippy = mkCheck "clippy" [ stable.clippy ] ''
            cargo clippy
          '';
        });

      formatter = withPackages (pkgs: pkgs.nixpkgs-fmt);

      devShells = withPackages (pkgs:
        let
          inherit (pkgs) system;
          inherit (fenix.packages.${system}) stable;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ]
              ++ builtins.attrValues self.checks.${system};

            RUST_SRC_PATH = "${stable.rust-src}/lib/rustlib/rust/library";

            packages = [
              self.formatter.${system}
              stable.rust-analyzer
              pkgs.cargo-watch
              pkgs.flyctl
            ];
          };
        });
    };
}
