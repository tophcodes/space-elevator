"""ctypes wrapper for libspnav.

Exposes only what the addon needs: open/close, the file descriptor, and
non-blocking event polling. Magellan/X11 entry points are intentionally absent
- spacenavd recommends spnav_open() over them.
"""

import ctypes
import ctypes.util

SPNAV_EVENT_MOTION = 1
SPNAV_EVENT_BUTTON = 2


class _MotionEvent(ctypes.Structure):
    _fields_ = [
        ("type", ctypes.c_int),
        ("x", ctypes.c_int),
        ("y", ctypes.c_int),
        ("z", ctypes.c_int),
        ("rx", ctypes.c_int),
        ("ry", ctypes.c_int),
        ("rz", ctypes.c_int),
        ("period", ctypes.c_uint),
        ("data", ctypes.POINTER(ctypes.c_int)),
    ]


class _ButtonEvent(ctypes.Structure):
    _fields_ = [
        ("type", ctypes.c_int),
        ("press", ctypes.c_int),
        ("bnum", ctypes.c_int),
    ]


class _Event(ctypes.Union):
    _fields_ = [
        ("type", ctypes.c_int),
        ("motion", _MotionEvent),
        ("button", _ButtonEvent),
    ]


class LibspnavMissing(RuntimeError):
    pass


class SpacenavdUnavailable(RuntimeError):
    pass


class _Lib:
    def __init__(self):
        path = ctypes.util.find_library("spnav")
        if not path:
            raise LibspnavMissing("libspnav not found; install spacenavd's client lib")
        lib = ctypes.CDLL(path)

        lib.spnav_open.argtypes = []
        lib.spnav_open.restype = ctypes.c_int
        lib.spnav_close.argtypes = []
        lib.spnav_close.restype = ctypes.c_int
        lib.spnav_fd.argtypes = []
        lib.spnav_fd.restype = ctypes.c_int
        lib.spnav_poll_event.argtypes = [ctypes.POINTER(_Event)]
        lib.spnav_poll_event.restype = ctypes.c_int
        self._lib = lib
        self._opened = False

    def open(self):
        if self._opened:
            return
        if self._lib.spnav_open() == -1:
            raise SpacenavdUnavailable("spnav_open failed; is spacenavd running?")
        self._opened = True

    def close(self):
        self._lib.spnav_close()
        self._opened = False

    def fd(self):
        return self._lib.spnav_fd()

    def poll(self):
        ev = _Event()
        if self._lib.spnav_poll_event(ctypes.byref(ev)) == 0:
            return None
        if ev.type == SPNAV_EVENT_MOTION:
            m = ev.motion
            return ("motion", (m.x, m.y, m.z, m.rx, m.ry, m.rz), m.period)
        if ev.type == SPNAV_EVENT_BUTTON:
            b = ev.button
            return ("button", b.bnum, bool(b.press))
        return None


_singleton = None


def get():
    global _singleton
    if _singleton is None:
        _singleton = _Lib()
    return _singleton
