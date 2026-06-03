from space_elevator import service
from space_elevator.lcd_client import DaemonUnavailable


class FakeClient:
    last = None

    def __init__(self):
        self.connected = False
        self.closed = False

    def connect(self):
        self.connected = True

    def set_state(self, state):
        FakeClient.last = state

    def close(self):
        self.closed = True


def test_push_once_sends_built_state():
    FakeClient.last = None
    built = {"profile": "FreeCAD", "mode": "PartWorkbench", "left": [], "right": []}

    ok = service.push_once(client_factory=FakeClient, builder=lambda: built)

    assert ok is True
    assert FakeClient.last == built


def test_push_once_swallows_daemon_unavailable():
    class DownClient(FakeClient):
        def connect(self):
            raise DaemonUnavailable("no socket")

    ok = service.push_once(client_factory=DownClient, builder=lambda: {})
    assert ok is False
