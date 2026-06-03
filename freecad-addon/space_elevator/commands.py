"""Resolve a FreeCAD command name to a tile label + icon.

label  = command's menuText (with & accelerators stripped)
icon   = the command icon as an SVG string, or a data: PNG URI, or None

FreeCAD/Qt access is injected so this unit-tests without FreeCAD. Targets
FreeCAD 1.1+ (PySide6); no PySide2 fallback.
"""

SVG_RESOURCE = ":/icons/%s.svg"


def _default_file(path):
    """Return raw bytes of a Qt resource, or None if it can't be opened."""
    from PySide6 import QtCore
    f = QtCore.QFile(path)
    if not f.open(QtCore.QIODevice.ReadOnly):
        return None
    try:
        return bytes(f.readAll())
    finally:
        f.close()


def _default_icon_b64(path):
    """Rasterize a Qt icon resource to a 48x48 PNG, return base64 str or None."""
    from PySide6 import QtCore, QtGui
    icon = QtGui.QIcon(path)
    if icon.isNull():
        return None
    pm = icon.pixmap(48, 48)
    ba = QtCore.QByteArray()
    buf = QtCore.QBuffer(ba)
    buf.open(QtCore.QIODevice.WriteOnly)
    if not pm.save(buf, "PNG"):
        buf.close()
        return None
    buf.close()
    return bytes(ba.toBase64()).decode()


def read_icon(pixmap, file_factory=None, icon_factory=None):
    """Resolve a pixmap resource name to SVG text, a data: PNG URI, or None."""
    if not pixmap:
        return None
    file_factory = file_factory or _default_file
    icon_factory = icon_factory or _default_icon_b64

    res = SVG_RESOURCE % pixmap
    data = file_factory(res)
    if data:
        return data.decode("utf-8", "replace") if isinstance(data, (bytes, bytearray)) else data

    b64 = icon_factory(res)
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
