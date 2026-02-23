{
  description = "Nix flake for building macos-remote with crane";

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
        clientMeta = craneLib.crateNameFromCargoToml {
          cargoToml = ./macos-remote-client/Cargo.toml;
        };
        serverMeta = craneLib.crateNameFromCargoToml {
          cargoToml = ./macos-remote-server/Cargo.toml;
        };
        workspaceMeta = {
          pname = "macos-remote-workspace";
          version = clientMeta.version;
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
          pname = "macos-remote-deps";
          version = workspaceMeta.version;
        });

        client = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          inherit (clientMeta) pname version;
          cargoExtraArgs = "-p macos-remote-client";
        });
        server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          inherit (serverMeta) pname version;
          cargoExtraArgs = "-p macos-remote-server";
        });
      in
      {
        packages = {
          inherit client server;
          default = pkgs.symlinkJoin {
            name = "macos-remote-binaries";
            paths = [ client server ];
          };
        };

        checks = {
          client-build = client;
          server-build = server;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            inherit (workspaceMeta) pname version;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          fmt = craneLib.cargoFmt {
            inherit src;
            inherit (workspaceMeta) pname version;
          };
          tests = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            inherit (workspaceMeta) pname version;
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
