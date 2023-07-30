{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      rec
      {
        packages = let
          snowflakeos-module-manager = pkgs.callPackage ./default.nix {};
        in {
          inherit snowflakeos-module-manager;
          default = snowflakeos-module-manager;
        };

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            clippy
            desktop-file-utils
            rust-analyzer
            rustc
            rustfmt
            cairo
            gdk-pixbuf
            gobject-introspection
            graphene
            gtk4
            libadwaita
            libxml2
            meson
            ninja
            openssl
            pkg-config
            polkit
            vte-gtk4
            wrapGAppsHook4
          ];
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      });
}
