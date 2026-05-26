"""Thin wrapper over FreeCAD's parameter system.

App.ParamGet returns a ParameterGrp; methods are
GetFloat/SetFloat/GetBool/SetBool/GetString/SetString, each with a default arg.

Settings layout (under "User parameter:BaseApp/Preferences/Mod/SpaceElevator"):

  axis_scale_tx, axis_scale_ty, axis_scale_tz   (float, default 1.0)
  axis_scale_rx, axis_scale_ry, axis_scale_rz   (float, default 1.0)
  invert_tx, invert_ty, ... (bool, default False)
  deadzone (float, default 0.02)         # 0..1 fraction of raw range
  lcd_enabled (bool, default True)
  daemon_socket_path (string, default "")  # "" => use $XDG_RUNTIME_DIR
"""

_PATH = "User parameter:BaseApp/Preferences/Mod/SpaceElevator"

_AXES = ("tx", "ty", "tz", "rx", "ry", "rz")


def _grp(param_get):
    return param_get(_PATH)


class Settings:
    def __init__(self, param_get):
        # param_get: callable taking a path string and returning a parameter group.
        self._grp = _grp(param_get)

    def axis_scale(self, axis):
        return self._grp.GetFloat(f"axis_scale_{axis}", 1.0)

    def set_axis_scale(self, axis, value):
        self._grp.SetFloat(f"axis_scale_{axis}", float(value))

    def invert(self, axis):
        return self._grp.GetBool(f"invert_{axis}", False)

    def set_invert(self, axis, value):
        self._grp.SetBool(f"invert_{axis}", bool(value))

    def deadzone(self):
        return self._grp.GetFloat("deadzone", 0.02)

    def set_deadzone(self, value):
        self._grp.SetFloat("deadzone", float(value))

    def lcd_enabled(self):
        return self._grp.GetBool("lcd_enabled", True)

    def set_lcd_enabled(self, value):
        self._grp.SetBool("lcd_enabled", bool(value))

    def daemon_socket_path(self):
        return self._grp.GetString("daemon_socket_path", "")

    def set_daemon_socket_path(self, value):
        self._grp.SetString("daemon_socket_path", str(value))


def from_freecad():
    import FreeCAD
    return Settings(FreeCAD.ParamGet)


AXES = _AXES
