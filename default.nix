{ rustPlatform, pkg-config }:

rustPlatform.buildRustPackage rec {
  pname = "faithful-pkg";
  version = "1.0.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ ];
}
