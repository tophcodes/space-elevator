# Home Manager module for the space-elevatord LCD daemon (systemd --user service).
# `self` is the flake, used to default the package to this flake's build.
self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.space-elevatord;
in {
  options.services.space-elevatord = {
    enable = lib.mkEnableOption "space-elevatord SpaceMouse Enterprise LCD daemon";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.space-elevatord;
      defaultText = lib.literalExpression "space-elevator.packages.\${system}.space-elevatord";
      description = "The space-elevatord package to run.";
    };

    logLevel = lib.mkOption {
      type = lib.types.str;
      default = "info";
      example = "debug";
      description = "tracing log level for the `space_elevatord` target (RUST_LOG).";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.space-elevatord = {
      Unit = {
        Description = "SpaceMouse Enterprise LCD daemon";
        Documentation = "https://github.com/tophcodes/space-elevator";
        # Listens on $XDG_RUNTIME_DIR/space-elevator.sock; needs the graphical session.
        PartOf = ["graphical-session.target"];
        After = ["graphical-session.target"];
      };

      Service = {
        ExecStart = lib.getExe cfg.package;
        Environment = ["RUST_LOG=space_elevatord=${cfg.logLevel}"];
        Restart = "on-failure";
        RestartSec = 2;
      };

      Install.WantedBy = ["graphical-session.target"];
    };
  };
}
