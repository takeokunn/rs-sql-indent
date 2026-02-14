{
  description = "rs-sql-indent - A CLI tool that formats SQL from stdin";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      imports = [ inputs.treefmt-nix.flakeModule ];

      perSystem =
        {
          config,
          self',
          system,
          lib,
          ...
        }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          };

          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

          src = lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              (craneLib.filterCargoSources path type)
              || (lib.hasSuffix ".sql" path)
              || (lib.hasSuffix ".expected" path);
          };

          commonArgs = {
            inherit src;
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          wasmArgs = commonArgs // {
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            doCheck = false;
          };

          wasmCargoArtifacts = craneLib.buildDepsOnly wasmArgs;

          wasmBuild = craneLib.buildPackage (
            wasmArgs
            // {
              cargoArtifacts = wasmCargoArtifacts;
              installPhaseCommand = ''
                mkdir -p $out/lib
                cp target/wasm32-unknown-unknown/release/rs_sql_indent.wasm $out/lib/
              '';
            }
          );
        in
        {
          packages = {
            default = craneLib.buildPackage (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );

            wasm = pkgs.stdenv.mkDerivation {
              name = "rs-sql-indent-wasm";
              nativeBuildInputs = [ pkgs.wasm-bindgen-cli ];
              dontUnpack = true;
              buildPhase = ''
                wasm-bindgen --target web --out-dir $out ${wasmBuild}/lib/rs_sql_indent.wasm
              '';
              installPhase = "true";
            };

            playground = pkgs.stdenv.mkDerivation (finalAttrs: {
              pname = "rs-sql-indent-playground";
              version = "0.0.0";
              src = lib.cleanSource ./playground;

              nativeBuildInputs = with pkgs; [
                nodejs_22
                pnpm_9
                pnpmConfigHook
              ];

              pnpmDeps = pkgs.fetchPnpmDeps {
                inherit (finalAttrs) pname version src;
                hash = "sha256-mLlehI8Ct28SLsmIgjz0B+NarKnYgu6pSMsabhTFqlQ=";
                fetcherVersion = 3;
                pnpm = pkgs.pnpm_9;
              };

              postConfigure = ''
                mkdir -p pkg
                cp -r ${self'.packages.wasm}/* pkg/
              '';

              buildPhase = ''
                runHook preBuild
                pnpm run build
                runHook postBuild
              '';

              installPhase = ''
                runHook preInstall
                cp -r dist $out
                cp ${./CNAME} $out/CNAME
                runHook postInstall
              '';
            });
          };

          checks = {
            clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "-- -D warnings";
              }
            );

            tests = craneLib.cargoTest (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );
          };

          treefmt = import ./treefmt.nix;
        };
    };
}
