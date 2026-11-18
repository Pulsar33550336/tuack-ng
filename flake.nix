{
  description = "重构后的 tuack 项目，旨在提供更加高效和轻量的出题体验。";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";

    templates-src = {
      url = "github:tuackng/templates";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      templates-src,
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.workspace.package.version;

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # 版本与组件由 rust-toolchain.toml 决定，与 CI 使用的版本保持一致。
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        lib = pkgs.lib;

        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.gitTracked ./.;
        };

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;
          pname = "tuack-ng";
          inherit version;
          cargoExtraArgs = "--locked -p tuack-ng -p tuack-xtask";
        };

        tuack-ng = craneLib.buildPackage {
          inherit src cargoArtifacts;
          pname = "tuack-ng";
          inherit version;

          nativeBuildInputs = with pkgs; [
            gcc
            installShellFiles
            testlib
          ];

          buildInputs = with pkgs; [
            testlib
          ];

          TESTLIB_PATH = "${pkgs.testlib}/include/testlib/testlib.h";
          VERGEN_IDEMPOTENT = 1;
          doInstallCargoArtifacts = false;
          doNotPostBuildInstallCargoBinaries = true;

          buildPhaseCargoCommand = ''
            cargo run --release --locked --offline -p tuack-xtask -- dist --platform nix
          '';

          installPhaseCommand = ''
            mkdir -p $out/bin $out/share/tuack-ng
            cp target/dist/tuack-ng $out/bin/
            cp -r target/dist/assets/. $out/share/tuack-ng/

            installShellCompletion --cmd tuack-ng \
              --bash target/dist/completions/bash \
              --fish target/dist/completions/fish \
              --zsh target/dist/completions/zsh

            install -Dm644 target/dist/LICENSE \
              $out/share/licenses/tuack-ng/LICENSE
            install -Dm644 target/dist/NOTICE.md \
              $out/share/licenses/tuack-ng/NOTICE.md

            mkdir -p $out/share/tuack-ng/templates
            cp -r ${templates-src}/* $out/share/tuack-ng/templates/
            chmod -R u+w $out/share/tuack-ng/templates/

            ln -sf ${pkgs.testlib}/include/testlib/testlib.h \
              $out/share/tuack-ng/checkers/testlib.h
          '';

          meta = with pkgs.lib; {
            description = "重构后的 tuack 项目，旨在提供更加高效和轻量的出题体验。";
            homepage = "https://github.com/tuackng/tuack-ng";
            license = with licenses; [
              agpl3Plus
              agpl3Only
              mit
            ];
            platforms = [
              "x86_64-linux"
              "aarch64-linux"
            ];
            mainProgram = "tuack-ng";
          };
        };

      in
      {
        packages = {
          default = tuack-ng;
          tuack-ng = tuack-ng;
        };

        devShells.default = craneLib.devShell {
          inputsFrom = [ tuack-ng ];

          nativeBuildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            gcc
            typst
            testlib
            zig
            cargo-zigbuild
            cargo-deb
            just
          ];

          shellHook = ''
            export TESTLIB_PATH="${pkgs.testlib}/include/testlib/testlib.h"
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"

            echo 'Tuack-NG Dev Shell -- run `just --list` for information'
          '';
        };
      }
    );
}
