"""Blocking client for space-elevatord's JSON-line Unix socket.

Used from the addon's Qt thread; calls are short (small SVGs, ms-scale device
writes) so blocking is acceptable. If that ever bites, move to a worker thread.
"""

import json
import os
import socket
from itertools import count


class DaemonUnavailable(RuntimeError):
    pass


class ProtocolError(RuntimeError):
    pass


def default_socket_path():
    runtime = os.environ.get("XDG_RUNTIME_DIR")
    if not runtime:
        runtime = f"/tmp/space-elevator-{os.getuid()}"
    return os.path.join(runtime, "space-elevator.sock")


class LcdClient:
    def __init__(self, path=None):
        self._path = path or default_socket_path()
        self._sock = None
        self._buf = b""
        self._id = count(1)

    def connect(self):
        if not os.path.exists(self._path):
            raise DaemonUnavailable(f"socket not found: {self._path}")
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            s.connect(self._path)
        except OSError as e:
            raise DaemonUnavailable(str(e))
        self._sock = s

    def close(self):
        if self._sock is not None:
            self._sock.close()
            self._sock = None

    def _request(self, payload):
        if self._sock is None:
            raise DaemonUnavailable("not connected")
        req = {"v": 1, "id": next(self._id)}
        req.update(payload)
        self._sock.sendall((json.dumps(req) + "\n").encode())
        return self._read_response()

    def _read_response(self):
        while b"\n" not in self._buf:
            chunk = self._sock.recv(4096)
            if not chunk:
                raise ProtocolError("daemon closed connection")
            self._buf += chunk
        line, self._buf = self._buf.split(b"\n", 1)
        return json.loads(line.decode())

    def ping(self):
        r = self._request({"cmd": "ping"})
        return r.get("ok") is True

    def clear(self):
        r = self._request({"cmd": "lcd_clear"})
        if not r.get("ok"):
            raise ProtocolError(r.get("error", "lcd_clear failed"))

    def display_svg(self, svg):
        r = self._request({"cmd": "lcd_display_svg", "svg": svg})
        if not r.get("ok"):
            raise ProtocolError(r.get("error", "lcd_display_svg failed"))
