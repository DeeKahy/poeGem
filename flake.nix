{
  description = "poeGem — what a Divine Font gem craft is worth this league";

  # Own pin (Go 1.26), like the other Go tools on the server.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAll (pkgs: rec {
        poegem = pkgs.buildGoModule {
          pname = "poegem";
          version = "2.0.0";
          src = self;
          vendorHash = null; # standard library only
          env.CGO_ENABLED = "0";
          ldflags = [ "-s" "-w" ];
          meta.mainProgram = "poegem";
        };
        default = poegem;
      });

      nixosModules.default = import ./nix/module.nix self;

      devShells = forAll (pkgs: {
        default = pkgs.mkShell { packages = [ pkgs.go pkgs.gopls ]; };
      });
    };
}
