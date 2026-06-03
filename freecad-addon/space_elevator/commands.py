"""Resolve a FreeCAD command name to a tile label + icon.

label  = command's menuText (with & accelerators stripped)
icon   = the command icon rasterized to a data: PNG URI, or None

FreeCAD ships command icons as full SVG *documents* (with an `<?xml?>` prolog),
which the daemon cannot inline into its own SVG (resvg rejects a nested XML
declaration). So we rasterize the icon to a self-contained PNG via Qt and hand
the daemon a `data:` URI it embeds as an `<image>` — robust for any icon and
correctly sized.

FreeCAD/Qt access is injected so this unit-tests without FreeCAD. Targets
FreeCAD 1.1+ (PySide6); no PySide2 fallback.
"""

SVG_RESOURCE = ":/icons/%s.svg"


def _default_icon_b64(path):
    """Rasterize a Qt icon resource to a 64x64 PNG, return base64 str or None."""
    from PySide6 import QtCore, QtGui
    icon = QtGui.QIcon(path)
    if icon.isNull():
        return None
    pm = icon.pixmap(64, 64)
    ba = QtCore.QByteArray()
    buf = QtCore.QBuffer(ba)
    buf.open(QtCore.QIODevice.WriteOnly)
    if not pm.save(buf, "PNG"):
        buf.close()
        return None
    buf.close()
    return bytes(ba.toBase64()).decode()


def read_icon(pixmap, icon_factory=None):
    """Rasterize a command's icon resource to a data: PNG URI, or None."""
    if not pixmap:
        return None
    icon_factory = icon_factory or _default_icon_b64
    b64 = icon_factory(SVG_RESOURCE % pixmap)
    if b64:
        return "data:image/png;base64," + b64
    return None


def _default_command_get(name):
    import FreeCADGui
    return FreeCADGui.Command.get(name)


def resolve(name, command_get=None, icon_resolver=None):
    """Return {"label", "icon"} for a command name. Unknown -> name as label."""
    command_get = command_get or _default_command_get
    icon_resolver = icon_resolver or read_icon

    cmd = command_get(name)
    if cmd is None:
        return {"label": name, "icon": None}
    info = cmd.getInfo()
    label = info.get("menuText", name).replace("&", "")
    icon = icon_resolver(info.get("pixmap", ""))
    return {"label": label, "icon": icon}
