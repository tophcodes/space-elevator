"""Build the lcd_set_state payload from FreeCAD's current bindings + workbench.

profile is fixed "FreeCAD"; mode is the active workbench. Twelve button
bindings (indices 0-11) become two 6-tile clusters. Unbound buttons become
blank tiles (the daemon renders them as nothing). No category yet -> neutral.
"""

from space_elevator import bindings as _bindings
from space_elevator import commands as _commands

PROFILE = "FreeCAD"


def _default_active_wb():
    import FreeCADGui
    return FreeCADGui.activeWorkbench().MenuText


def _blank_tile():
    return {"label": "", "icon": None, "active": False}


def build_state(read_bindings=None, resolve=None, active_wb=None):
    read_bindings = read_bindings or _bindings.read_bindings
    resolve = resolve or _commands.resolve
    active_wb = active_wb or _default_active_wb

    tiles = []
    for name in read_bindings():
        if not name:
            tiles.append(_blank_tile())
            continue
        info = resolve(name)
        tiles.append({"label": info["label"], "icon": info["icon"], "active": False})

    return {
        "profile": PROFILE,
        "mode": active_wb(),
        "left": tiles[:6],
        "right": tiles[6:12],
    }
