{
  description = "space-elevator — SpaceMouse Enterprise LCD daemon, FreeCAD addon, and spnav Rust bindings";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
    pkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [self.overlays.default];
      };
  in {
    overlays.default = _final: prev: {
      space-elevatord = prev.callPackage ./nix/package.nix {};
    };

    packages = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      inherit (pkgs) space-elevatord;
      default = pkgs.space-elevatord;
    });

    homeManagerModules = {
      space-elevatord = import ./nix/hm-module.nix self;
      default = self.homeManagerModules.space-elevatord;
    };

    nixosModules = {
      space-elevator-udev = import ./nix/nixos-module.nix;
      default = self.nixosModules.space-elevator-udev;
    };
  };
}
