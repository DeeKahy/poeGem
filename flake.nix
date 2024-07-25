{
  description = "Rust project with required dependencies and automatic cargo run";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.fontconfig
            pkgs.libopus
            pkgs.pkg-config
            pkgs.cmake
            pkgs.gnumake
            pkgs.gcc
            pkgs.rustup
          ];

          shellHook = ''
             npm install express axios
             node app.js
          '';
        };
      });
}
