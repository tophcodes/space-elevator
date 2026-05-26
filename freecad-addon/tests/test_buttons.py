from space_elevator.buttons import Bindings


class FakeGroup:
    def __init__(self):
        self._strings = {}
    def GetString(self, k, d): return self._strings.get(k, d)
    def SetString(self, k, v): self._strings[k] = v


def fake_param_get():
    g = FakeGroup()
    return lambda path: g


def test_bind_and_lookup():
    b = Bindings(fake_param_get())
    b.set(0, "Std_New")
    assert b.get(0) == "Std_New"


def test_unset_button_returns_none():
    b = Bindings(fake_param_get())
    assert b.get(7) is None


def test_clear_binding():
    b = Bindings(fake_param_get())
    b.set(3, "Std_Save")
    b.set(3, "")
    assert b.get(3) is None
