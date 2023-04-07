{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            openssl
            pkg-config
            cargo-watch
            rust-analyzer
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
            })
            postgresql
          ] ++ lib.optionals pkgs.stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
            SystemConfiguration
            Security
          ]);
          shellHook = ''
            ### Environment variables
            export RUST_LOG=debug
            export PATH=$PATH:/Users/v3lix/.cargo/bin
            # Database
            export NIX_SHELL_DIR=$PWD/.nix-shell
            export PGDATA=$NIX_SHELL_DIR/db
            export DATABASE_URL="postgres://spmt:spmt-database-dev@localhost/spmt"
          '';
        };
      }
    );
}
