"""Wires libspnav's fd into Qt's main loop.

Drains all pending events whenever the fd becomes readable and dispatches each
to a handler callback. The callback is invoked from Qt's thread; it can touch
FreeCAD's GUI safely.
"""

import FreeCAD

try:
    from PySide6 import QtCore
except ImportError:
    from PySide2 import QtCore  # FreeCAD 1.0 on most distros

from . import libspnav


class EventPump(QtCore.QObject):
    def __init__(self, on_event, parent=None):
        super().__init__(parent)
        self._lib = libspnav.get()
        self._on_event = on_event
        self._notifier = None

    def start(self):
        try:
            self._lib.open()
        except libspnav.SpacenavdUnavailable as e:
            FreeCAD.Console.PrintWarning(f"Space Elevator: {e}\n")
            return False
        fd = self._lib.fd()
        if fd < 0:
            FreeCAD.Console.PrintWarning("Space Elevator: no spacenavd fd\n")
            self._lib.close()
            return False
        self._notifier = QtCore.QSocketNotifier(fd, QtCore.QSocketNotifier.Read)
        self._notifier.activated.connect(self._drain)
        self._notifier.setEnabled(True)
        return True

    def stop(self):
        if self._notifier is not None:
            self._notifier.setEnabled(False)
            self._notifier.deleteLater()
            self._notifier = None
        self._lib.close()

    def _drain(self, _fd):
        # Drain everything queued — fd is edge-friendly; multiple events can
        # arrive between activations.
        while True:
            ev = self._lib.poll()
            if ev is None:
                return
            try:
                self._on_event(ev)
            except Exception as exc:
                FreeCAD.Console.PrintError(f"Space Elevator handler error: {exc}\n")
