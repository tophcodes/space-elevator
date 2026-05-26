import FreeCAD

from .event_pump import EventPump
from . import libspnav

_state = {"active": False, "pump": None}


def _debug_handler(ev):
    kind = ev[0]
    if kind == "motion":
        _, axes, period = ev
        FreeCAD.Console.PrintMessage(f"motion {axes} period={period}\n")
    elif kind == "button":
        _, bnum, pressed = ev
        FreeCAD.Console.PrintMessage(f"button {bnum} {'press' if pressed else 'release'}\n")


def on_activate():
    if _state["active"]:
        return
    _state["active"] = True

    try:
        pump = EventPump(_debug_handler)
    except libspnav.LibspnavMissing as e:
        FreeCAD.Console.PrintError(f"Space Elevator: {e}\n")
        _state["active"] = False
        return

    if pump.start():
        _state["pump"] = pump
        FreeCAD.Console.PrintMessage("Space Elevator: event pump started\n")
    else:
        _state["active"] = False


def on_deactivate():
    if not _state["active"]:
        return
    pump = _state.pop("pump", None)
    if pump is not None:
        pump.stop()
    _state["active"] = False
    FreeCAD.Console.PrintMessage("Space Elevator: deactivated\n")
