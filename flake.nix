{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs, ... }:
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

      cargoLock.lockFile = ./Cargo.lock;
    in
    {
      packages = withPackages (pkgs:
        let
          rapla-sync = pkgs.rustPlatform.buildRustPackage {
            inherit name version src cargoLock;
            buildInputs = lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
            ];
          };

          rapla-sync-static =
            pkgs.pkgsStatic.rustPlatform.buildRustPackage {
              inherit name version src cargoLock;
            };

          docker-image = pkgs.dockerTools.buildLayeredImage {
            inherit name;
            tag = "latest";

            contents = [
              rapla-sync-static
              pkgs.cacert
            ];

            config.Cmd = [ "rapla" ];
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

          mkCheck = name: nativeBuildInputs: checkPhase:
            rapla-sync.overrideAttrs (final: prev: {
              name = "check-${name}";
              dontBuild = true;

              nativeBuildInputs =
                prev.nativeBuildInputs ++ nativeBuildInputs;

              inherit checkPhase;

              installPhase = ''
                mkdir -p $out
              '';
            });
        in
        {
          rustfmt = mkCheck "rustfmt" [ pkgs.rustfmt ] ''
            cargo fmt --all -- --check
          '';

          clippy = mkCheck "clippy" [ pkgs.clippy ] ''
            cargo clippy
          '';
        });

      formatter = withPackages (pkgs: pkgs.nixpkgs-fmt);

      devShells = withPackages (pkgs:
        let
          inherit (pkgs) system;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ]
              ++ builtins.attrValues self.checks.${system};

            packages = [
              self.formatter.${system}
              pkgs.rust-analyzer
              pkgs.cargo-watch
              pkgs.flyctl
            ];
          };
        });
    };
}
