with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "drop";

  buildInputs = [
      stdenv
  ];
}
