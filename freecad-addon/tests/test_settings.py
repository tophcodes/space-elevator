from space_elevator.settings import Settings, AXES


class FakeGroup:
    def __init__(self):
        self._floats = {}
        self._bools = {}
        self._strings = {}

    def GetFloat(self, k, d): return self._floats.get(k, d)
    def SetFloat(self, k, v): self._floats[k] = v
    def GetBool(self, k, d): return self._bools.get(k, d)
    def SetBool(self, k, v): self._bools[k] = v
    def GetString(self, k, d): return self._strings.get(k, d)
    def SetString(self, k, v): self._strings[k] = v


def fake_param_get():
    grp = FakeGroup()
    def f(path):
        return grp
    return f


def test_axis_scale_roundtrip():
    s = Settings(fake_param_get())
    for a in AXES:
        assert s.axis_scale(a) == 1.0
        s.set_axis_scale(a, 2.5)
        assert s.axis_scale(a) == 2.5


def test_invert_roundtrip():
    s = Settings(fake_param_get())
    s.set_invert("tx", True)
    assert s.invert("tx") is True
    assert s.invert("ty") is False


def test_deadzone_default():
    s = Settings(fake_param_get())
    assert s.deadzone() == 0.02
