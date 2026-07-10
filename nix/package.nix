{
  rustPlatform,
  lib,
}:
let
  fs = lib.fileset;
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = finalAttrs.passthru.cargoToml.package.name;
  inherit (finalAttrs.passthru.cargoToml.package) version;

  src = fs.toSource {
    root = ../.;
    fileset = fs.unions [
      ../src
      ../Cargo.lock
      ../Cargo.toml
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  passthru = {
    cargoToml = lib.importTOML ../Cargo.toml;
  };

  meta = {
    description = "Little helper to migrate/snatch mavens";
    homepage = "https://github.com/macuguita/maven-snatcher";
    mainProgram = finalAttrs.pname;
    license = lib.licenses.eupl12;
    sourceProvenance = with lib.sourceTypes; [
      fromSource
    ];
  };
})
