from space_elevator import bindings


class FakeGroup:
    def __init__(self, cmd):
        self._cmd = cmd

    def GetString(self, name, default=""):
        assert name == "Command"
        return self._cmd


def test_read_bindings_returns_twelve_in_index_order():
    table = {
        "User parameter:BaseApp/Spaceball/Buttons/0": FakeGroup("Std_ViewFit"),
        "User parameter:BaseApp/Spaceball/Buttons/6": FakeGroup("Sketcher_CreateLine"),
    }

    def fake_param_get(path):
        return table.get(path, FakeGroup(""))

    result = bindings.read_bindings(param_get=fake_param_get)

    assert len(result) == 12
    assert result[0] == "Std_ViewFit"
    assert result[6] == "Sketcher_CreateLine"
    assert result[1] == ""  # unbound
