"""Lifecycle glue: on startup + workbench change, push LCD state to the daemon.

LCD output is best-effort: if the daemon is down, push_once returns False and
the addon stays silent (no exceptions bubble into FreeCAD's UI).
"""

from space_elevator.lcd_client import LcdClient, DaemonUnavailable, ProtocolError
from space_elevator.state import build_state


def push_once(client_factory=LcdClient, builder=build_state):
    """Build current state and send it. Returns True on success, False if the
    daemon is unavailable or errors."""
    state = builder()
    client = client_factory()
    try:
        client.connect()
        client.set_state(state)
        return True
    except (DaemonUnavailable, ProtocolError):
        return False
    finally:
        client.close()


def start():
    """Connect the workbench-activated signal and push the initial state.
    Called once from InitGui at GUI startup."""
    import FreeCADGui
    mw = FreeCADGui.getMainWindow()
    if mw is not None:
        # workbenchActivated emits the workbench name (str); we rebuild from scratch.
        mw.workbenchActivated.connect(lambda _name=None: push_once())
    push_once()
