"""Read FreeCAD's spaceball button -> command bindings.

Bindings live at ``BaseApp/Spaceball/Buttons/<N>/Command`` (flat indices,
0-based). Indices 0-11 are the SpaceMouse Enterprise LCD function keys 1-12.
FreeCAD access is injected so this is unit-testable without FreeCAD.
"""

BUTTONS_PATH = "User parameter:BaseApp/Spaceball/Buttons/%d"
LCD_BUTTON_COUNT = 12


def _default_param_get(path):
    import FreeCAD
    return FreeCAD.ParamGet(path)


def read_bindings(param_get=None, count=LCD_BUTTON_COUNT):
    """Return ``count`` command names by button index; "" for unbound."""
    param_get = param_get or _default_param_get
    out = []
    for i in range(count):
        group = param_get(BUTTONS_PATH % i)
        out.append(group.GetString("Command", ""))
    return out
