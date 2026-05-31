import pytest

from space_elevator.libspnav import _Lib, SpacenavdUnavailable


class _FakeLib:
    def __init__(self, open_result=0):
        self.open_calls = 0
        self.close_calls = 0
        self._open_result = open_result

    def spnav_open(self):
        self.open_calls += 1
        return self._open_result

    def spnav_close(self):
        self.close_calls += 1
        return 0


def _make(fake):
    # Bypass __init__: it loads the real libspnav via ctypes, which isn't
    # available (and isn't what we're testing). Inject a fake instead.
    lib = _Lib.__new__(_Lib)
    lib._lib = fake
    lib._opened = False
    return lib


def test_open_is_idempotent():
    fake = _FakeLib()
    lib = _make(fake)
    lib.open()
    lib.open()
    assert fake.open_calls == 1


def test_close_allows_reopen():
    fake = _FakeLib()
    lib = _make(fake)
    lib.open()
    lib.close()
    lib.open()
    assert fake.open_calls == 2


def test_open_raises_when_spnav_open_fails():
    fake = _FakeLib(open_result=-1)
    lib = _make(fake)
    with pytest.raises(SpacenavdUnavailable):
        lib.open()


def test_failed_open_can_be_retried():
    fake = _FakeLib(open_result=-1)
    lib = _make(fake)
    with pytest.raises(SpacenavdUnavailable):
        lib.open()
    fake._open_result = 0
    lib.open()
    assert fake.open_calls == 2
