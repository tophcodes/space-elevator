import json
import socket
import threading

import pytest

from space_elevator.lcd_client import LcdClient, DaemonUnavailable


def _fake_server(socket_path, responses):
    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(socket_path)
    srv.listen(1)

    def run():
        conn, _ = srv.accept()
        buf = b""
        sent = 0
        while sent < len(responses):
            chunk = conn.recv(4096)
            if not chunk:
                return
            buf += chunk
            while b"\n" in buf and sent < len(responses):
                line, buf = buf.split(b"\n", 1)
                conn.sendall((json.dumps(responses[sent]) + "\n").encode())
                sent += 1
        conn.close()
        srv.close()

    threading.Thread(target=run, daemon=True).start()


def test_ping_round_trip(tmp_path):
    sock = str(tmp_path / "s.sock")
    _fake_server(sock, [{"v": 1, "id": 1, "ok": True}])
    c = LcdClient(sock)
    c.connect()
    assert c.ping() is True


def test_missing_socket_raises():
    c = LcdClient("/no/such/path.sock")
    with pytest.raises(DaemonUnavailable):
        c.connect()


def _capturing_server(socket_path, response):
    """Like _fake_server but captures received messages for inspection."""
    import threading as _threading
    captured = {}
    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(socket_path)
    srv.listen(1)

    def run():
        conn, _ = srv.accept()
        buf = b""
        chunk = conn.recv(4096)
        buf += chunk
        line, _ = buf.split(b"\n", 1)
        captured["msg"] = json.loads(line.decode())
        conn.sendall((json.dumps(response) + "\n").encode())
        conn.close()
        srv.close()

    _threading.Thread(target=run, daemon=True).start()
    return captured


def test_set_state_sends_cmd_and_merges_payload(tmp_path):
    sock = str(tmp_path / "s.sock")
    captured = _capturing_server(sock, {"v": 1, "id": 1, "ok": True})
    c = LcdClient(sock)
    c.connect()
    c.set_state({"profile": "FreeCAD", "mode": "Part", "left": [], "right": []})

    msg = captured["msg"]
    assert msg["cmd"] == "lcd_set_state"
    assert msg["profile"] == "FreeCAD"
    assert msg["mode"] == "Part"
    assert msg["v"] == 1 and "id" in msg
