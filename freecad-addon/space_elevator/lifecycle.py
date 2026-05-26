import FreeCAD

_state = {"active": False}


def on_activate():
    if _state["active"]:
        return
    _state["active"] = True
    FreeCAD.Console.PrintMessage("Space Elevator: activated\n")


def on_deactivate():
    if not _state["active"]:
        return
    _state["active"] = False
    FreeCAD.Console.PrintMessage("Space Elevator: deactivated\n")
