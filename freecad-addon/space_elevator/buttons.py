"""Button -> FreeCAD command bindings.

Stored as ParamGet strings keyed `button_<n>` under the addon's parameter
group. Empty string means unbound.
"""

_PATH = "User parameter:BaseApp/Preferences/Mod/SpaceElevator/Buttons"


class Bindings:
    def __init__(self, param_get):
        self._grp = param_get(_PATH)

    def get(self, bnum):
        s = self._grp.GetString(f"button_{bnum}", "")
        return s if s else None

    def set(self, bnum, command_name):
        self._grp.SetString(f"button_{bnum}", command_name or "")
