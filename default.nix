{ pkgs ? import <nixpkgs> { }
, lib ? import <nixpkgs/lib>
}:
pkgs.stdenv.mkDerivation rec {
  pname = "snowflakeos-module-manager";
  version = "0.0.1";

  src = [ ./. ];

  cargoDeps = pkgs.rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "nix-data-0.0.2" = "sha256-yts2bkp9cn4SuYPYjgTNbOwTtpFxps3TU8zmS/ftN/Q=";
    };
  };

  nativeBuildInputs = with pkgs; [
    appstream-glib
    polkit
    gettext
    desktop-file-utils
    meson
    ninja
    pkg-config
    git
    wrapGAppsHook4
  ] ++ (with pkgs.rustPlatform; [
    cargoSetupHook
    cargo
    rustc
  ]);

  buildInputs = with pkgs; [
    gdk-pixbuf
    glib
    gtk4
    libadwaita
    libxml2
    openssl
    vte-gtk4
    wayland
    gnome.adwaita-icon-theme
    desktop-file-utils
  ];
}
