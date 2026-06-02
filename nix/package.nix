{
  lib,
  rustPlatform,
  pkg-config,
  libspnav,
  systemd,
}: let
  # The daemon never touches X11; drop it to keep the closure small (mirrors devenv.nix).
  libspnavHeadless = libspnav.overrideAttrs (_: {
    configureFlags = ["--disable-debug" "--disable-x11"];
  });
in
  rustPlatform.buildRustPackage {
    pname = "space-elevatord";
    version = "0.1.0";

    src = lib.cleanSourceWith {
      src = ../.;
      # Keep the build input small and pure: drop the build dir, result symlinks and VCS noise.
      filter = path: type:
        (lib.cleanSourceFilter path type)
        && (let
          base = baseNameOf path;
        in
          base != "target" && base != "result" && !(lib.hasPrefix "result-" base));
    };

    cargoLock.lockFile = ../Cargo.lock;

    # Build only the daemon out of the workspace.
    cargoBuildFlags = ["--package" "space-elevatord"];
    cargoTestFlags = ["--package" "space-elevatord"];

    # bindgenHook wires up libclang for spnav-sys' bindgen build.
    nativeBuildInputs = [pkg-config rustPlatform.bindgenHook];

    # libspnav -> spnav-sys (pkg-config + link); systemd -> libudev for hidapi.
    # libusb is vendored by libusb1-sys, resvg/usvg/image are pure Rust.
    buildInputs = [libspnavHeadless systemd];

    meta = {
      description = "Daemon for SpaceMouse Enterprise LCD output, paired with the space-elevator FreeCAD addon";
      homepage = "https://github.com/tophcodes/space-elevator";
      license = with lib.licenses; [mit asl20];
      mainProgram = "space-elevatord";
      platforms = lib.platforms.linux;
    };
  }
