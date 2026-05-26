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

    # spnav
    systemd # contains libudev, which is required for hidapi

    # space-elevator (src-tauri)
    glib
    libsoup_3 # TODO: Is this always needed?
    webkitgtk_4_1
  ];

  languages = {
    rust.enable = true;

    javascript = {
      enable = true;
      bun.enable = true;
    };
  };

  # See full reference at https://devenv.sh/reference/options/
}
