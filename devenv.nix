{pkgs, ...}: let
  libspnav-sans-x11 = pkgs.libspnav.overrideAttrs (final: prev: {
    configureFlags = ["--disable-debug" "--disable-x11"];
  });
in {
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

  packages = with pkgs; [
    clang
    libspnav-sans-x11
  ];

  # https://devenv.sh/languages/
  languages = {
    rust.enable = true;

    javascript = {
      enable = true;
      bun.enable = true;
    };
  };

  # See full reference at https://devenv.sh/reference/options/
}
