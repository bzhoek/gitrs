{ nixpkgs ? import <nixpkgs>
, pkgs ? nixpkgs {}
}:
let
  libsass = pkgs.libsass;
in pkgs.mkShell {
  buildInputs = [
    pkgs.libgit2
    pkgs.pkg-config
  ];
}
