import FreeCAD

from .event_pump import EventPump
from . import libspnav, settings, axes, navigator

_state = {"active": False, "pump": None, "settings": None}


def _make_handler(s):
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
            if pressed:
                FreeCAD.Console.PrintMessage(f"Space Elevator: button {bnum}\n")
    return handle


def on_activate():
    if _state["active"]:
        return
    s = settings.from_freecad()
    try:
        pump = EventPump(_make_handler(s))
    except libspnav.LibspnavMissing as e:
        FreeCAD.Console.PrintError(f"Space Elevator: {e}\n")
        return
    if pump.start():
        _state.update(active=True, pump=pump, settings=s)


def on_deactivate():
    if not _state["active"]:
        return
    pump = _state.pop("pump", None)
    if pump is not None:
        pump.stop()
    _state["active"] = False
    _state["settings"] = None
