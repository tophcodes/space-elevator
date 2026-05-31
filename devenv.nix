{pkgs, ...}: let
  libspnav-sans-x11 = pkgs.libspnav.overrideAttrs (final: prev: {
    configureFlags = ["--disable-debug" "--disable-x11"];
  });
in {
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

  packages = with pkgs; [
    # spnav-sys
    clang
    libspnav-sans-x11

    # spnav + space-elevatord
    systemd # contains libudev, which is required for hidapi

    # freecad-addon test runner
    python3Packages.pytest
  ];

  languages = {
    rust.enable = true;
    python.enable = true;
  };

  # See full reference at https://devenv.sh/reference/options/
}
