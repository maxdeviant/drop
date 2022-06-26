with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "drop";

  buildInputs = [
      stdenv
      pkg-config
      openssl
      sqlx-cli
      flyctl
  ];
}
