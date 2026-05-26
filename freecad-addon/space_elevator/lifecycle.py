import FreeCAD

from .event_pump import EventPump
from . import libspnav, settings, axes, navigator, buttons, lcd_client, lcd_labels

_state = {"active": False, "pump": None, "settings": None, "bindings": None}


def _make_handler(s, b):
    def handle(ev):
        kind = ev[0]
        if kind == "motion":
            _, raw, period = ev
            scales = tuple(s.axis_scale(a) for a in settings.AXES)
            inverts = tuple(s.invert(a) for a in settings.AXES)
            transformed = axes.transform(raw, scales, inverts, s.deadzone())
            navigator.apply(transformed, period)
        elif kind == "button":
            _, bnum, pressed = ev
            if not pressed:
                return
            cmd = b.get(bnum)
            if cmd is None:
                return
            try:
                import FreeCADGui as Gui
                Gui.runCommand(cmd)
            except Exception as exc:
                FreeCAD.Console.PrintError(f"Space Elevator: command {cmd!r} failed: {exc}\n")
    return handle


def _short_name(cmd):
    if "_" in cmd:
        return cmd.split("_", 1)[1]
    return cmd


def _push_labels(s, b):
    if not s.lcd_enabled():
        return
    path = s.daemon_socket_path() or None
    client = lcd_client.LcdClient(path)
    try:
        client.connect()
    except lcd_client.DaemonUnavailable:
        return  # daemon optional
    try:
        labels = [_short_name(b.get(i) or "") for i in range(8)]
        client.display_svg(lcd_labels.render(labels))
    except lcd_client.ProtocolError as exc:
        FreeCAD.Console.PrintWarning(f"Space Elevator LCD: {exc}\n")
    finally:
        client.close()


def on_activate():
    if _state["active"]:
        return
    s = settings.from_freecad()
    b = buttons.Bindings(FreeCAD.ParamGet)
    try:
        pump = EventPump(_make_handler(s, b))
    except libspnav.LibspnavMissing as e:
        FreeCAD.Console.PrintError(f"Space Elevator: {e}\n")
        return
    if pump.start():
        _state.update(active=True, pump=pump, settings=s, bindings=b)
        _push_labels(s, b)


def on_deactivate():
    if not _state["active"]:
        return
    pump = _state.pop("pump", None)
    if pump is not None:
        pump.stop()
    _state["active"] = False
    _state["settings"] = None
    _state["bindings"] = None
