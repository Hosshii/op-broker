{
  description = "Local command-line tools";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      crane,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        workspaceToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        workspaceRustVersion = workspaceToml.workspace.package."rust-version";
        rustToolchainVersion =
          if builtins.match "^[0-9]+\\.[0-9]+\\.[0-9]+$" workspaceRustVersion != null then
            workspaceRustVersion
          else
            "${workspaceRustVersion}.0";
        rustToolchain = pkgs.rust-bin.stable.${rustToolchainVersion}.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            (craneLib.filterCargoSources path type) || (pkgs.lib.hasSuffix ".proto" path);
        };
        packageMeta = craneLib.crateNameFromCargoToml {
          cargoToml = ./crates/localtools/Cargo.toml;
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [
            protobuf
            pkg-config
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "localtools-deps";
          inherit (packageMeta) version;
        });

        localtools = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          inherit (packageMeta) pname version;
          cargoExtraArgs = "-p localtools";
          meta.mainProgram = "localtools";
        });
      in
      {
        packages = {
          inherit localtools;
          default = localtools;
        };

        checks = {
          localtools-build = localtools;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            inherit (packageMeta) pname version;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          fmt = craneLib.cargoFmt {
            inherit src;
            inherit (packageMeta) pname version;
          };
          tests = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            inherit (packageMeta) pname version;
            cargoExtraArgs = "--all";
          });
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            protobuf
            pkg-config
          ];
        };
      }
    );
}
