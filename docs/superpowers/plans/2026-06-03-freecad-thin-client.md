# FreeCAD thin LCD-client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A thin FreeCAD addon (no workbench) that reads FreeCAD's spaceball button bindings + active workbench, resolves each command's label/icon, and pushes an `lcd_set_state` to `space-elevatord` so the SpaceMouse Enterprise LCD mirrors the 12 function keys.

**Architecture:** No spnav, no driver. On GUI startup and on every workbench change, the addon builds an `LcdState` dict (profile=`FreeCAD`, mode=active workbench, 12 tiles from buttons 0–11) and sends it over the existing unix socket via `LcdClient.set_state`. The old driver addon (event pump, navigator, libspnav, settings dialog) is deleted. FreeCAD-touching code is dependency-injected so it unit-tests without FreeCAD installed. One small daemon change: render blank tiles for empty/unbound buttons.

**Tech Stack:** Python 3 + pytest (addon, mocks FreeCAD/PySide6 via injection), Rust + cargo (one daemon render tweak), FreeCAD 1.1.1 (PySide6), `space-elevatord` JSON-line unix socket.

---

## Spike facts this plan relies on (from `freecad-spaceball-discovery` memory)

- Button bindings: `User parameter:BaseApp/Spaceball/Buttons/<N>/Command` (String), flat indices 0–30. Indices **0–11** are LCD function keys 1–12 (`enterprise_layout.json`: `index` 0–11, `name` "1".."12"). Empty string = unbound.
- Command info: `FreeCADGui.Command.get(name).getInfo()` → `menuText` (label, may contain `&` accelerators), `pixmap` (icon resource name, e.g. `"zoom-all"`).
- Icon SVG: `QtCore.QFile(":/icons/{pixmap}.svg")` opens and reads real SVG bytes. Fallback: `QtGui.QIcon(":/icons/{pixmap}.svg").pixmap(48,48)` → PNG → base64 (daemon accepts `data:` URIs).
- Mode: `FreeCADGui.activeWorkbench().name()`.
- Workbench-change signal: `FreeCADGui.getMainWindow().workbenchActivated` (Qt signal, emits the workbench name as a str).

## File structure

**Daemon (Rust):**
- Modify: `space-elevatord/src/lcd_template.rs` — `render_tile` skips fully-empty tiles.

**Addon (Python) — new modules:**
- Create: `freecad-addon/space_elevator/bindings.py` — read `Buttons/0..11/Command`.
- Create: `freecad-addon/space_elevator/commands.py` — resolve a command name → `{label, icon}` (label + svg/png icon).
- Create: `freecad-addon/space_elevator/state.py` — assemble the `lcd_set_state` dict.
- Create: `freecad-addon/space_elevator/service.py` — wire the workbench signal → build → send.
- Modify: `freecad-addon/space_elevator/lcd_client.py` — add `set_state`.
- Rewrite: `freecad-addon/InitGui.py` — thin client bootstrap, no workbench.
- Rewrite: `freecad-addon/Init.py` — minimal/no-op.
- Modify: `freecad-addon/package.xml` — drop the dead workbench content block, bump version.

**Addon — delete (obsolete driver layer):**
- `space_elevator/event_pump.py`, `navigator.py`, `libspnav.py`, `axes.py`, `buttons.py`, `button_dialog.py`, `settings.py`, `preferences.py`, `lifecycle.py`
- `Resources/ui/preferences.ui`
- `tests/test_axes.py`, `tests/test_buttons.py`, `tests/test_libspnav_open.py`, `tests/test_libspnav_types.py`, `tests/test_settings.py`

**Keep:** `space_elevator/lcd_client.py`, `lcd_labels.py`, `enterprise_layout.json`, `Resources/icons/space-elevator.svg`, `tests/test_lcd_client.py`, `pyproject.toml`.

**Tests — new:**
- `freecad-addon/tests/test_bindings.py`, `tests/test_commands.py`, `tests/test_state.py`, `tests/test_service.py`.

---

## Task 1: Daemon renders blank tiles for empty/unbound buttons

A FreeCAD keypad usually has free buttons. An empty tile (no label, no icon, not active) must render as nothing, not a stray fallback glyph.

**Files:**
- Modify: `space-elevatord/src/lcd_template.rs` (function `render_tile`, and the `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `space-elevatord/src/lcd_template.rs`:

```rust
    #[test]
    fn empty_tile_renders_nothing() {
        // Signal theme draws a tile bg rect (#15171C) for every non-empty tile.
        let blank = LcdState {
            theme: Theme::Signal,
            profile: "FreeCAD".into(),
            mode: "Part".into(),
            left: vec![Tile::default()], // label "", icon None, cat None, active false
            right: vec![],
        };
        let svg = render(&blank);
        assert!(!svg.contains("#15171C"), "blank tile must not draw a tile background");

        // sanity: a real tile DOES draw the bg
        let filled = LcdState {
            left: vec![Tile { label: "Line".into(), icon: Some("line".into()), cat: Some("draw".into()), active: false }],
            ..LcdState::default()
        };
        assert!(render(&filled).contains("#15171C"));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p space-elevatord --lib empty_tile_renders_nothing`
Expected: FAIL — blank tile currently still emits the icon group + label (and, for Signal, the bg rect).

- [ ] **Step 3: Add the early-return to `render_tile`**

In `render_tile`, immediately after `let mut out = String::new();`, insert:

```rust
    // Fully-empty tile (unbound button): render nothing.
    if it.label.is_empty() && it.icon.is_none() && !it.active {
        return out;
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p space-elevatord --lib`
Expected: PASS (all lcd_template tests, including `empty_tile_renders_nothing`).

- [ ] **Step 5: Commit**

```bash
git add space-elevatord/src/lcd_template.rs
git commit -m "feat(daemon): render blank tiles for empty/unbound buttons"
```

---

## Task 2: Add `set_state` to the LCD client

**Files:**
- Modify: `freecad-addon/space_elevator/lcd_client.py`
- Test: `freecad-addon/tests/test_lcd_client.py`

- [ ] **Step 1: Write the failing test**

Append to `freecad-addon/tests/test_lcd_client.py`. It uses the same in-process fake-socket pattern as the existing tests in that file — if the existing tests use a helper to stub `socket.socket`, reuse it; otherwise this self-contained fake works:

```python
def test_set_state_sends_cmd_and_merges_payload(monkeypatch):
    import space_elevator.lcd_client as lc

    sent = {}

    class FakeSock:
        def connect(self, path): pass
        def sendall(self, data): sent["raw"] = data
        def recv(self, n): return b'{"v":1,"id":1,"ok":true}\n'
        def close(self): pass

    monkeypatch.setattr(lc.os.path, "exists", lambda p: True)
    monkeypatch.setattr(lc.socket, "socket", lambda *a, **k: FakeSock())

    client = lc.LcdClient(path="/tmp/x.sock")
    client.connect()
    client.set_state({"profile": "FreeCAD", "mode": "Part", "left": [], "right": []})

    import json
    msg = json.loads(sent["raw"].decode())
    assert msg["cmd"] == "lcd_set_state"
    assert msg["profile"] == "FreeCAD"
    assert msg["mode"] == "Part"
    assert msg["v"] == 1 and "id" in msg
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd freecad-addon && python -m pytest tests/test_lcd_client.py::test_set_state_sends_cmd_and_merges_payload -v`
Expected: FAIL with `AttributeError: 'LcdClient' object has no attribute 'set_state'`

- [ ] **Step 3: Implement `set_state`**

Add to `LcdClient` in `freecad-addon/space_elevator/lcd_client.py`, after `display_svg`:

```python
    def set_state(self, state):
        payload = {"cmd": "lcd_set_state"}
        payload.update(state)
        r = self._request(payload)
        if not r.get("ok"):
            raise ProtocolError(r.get("error", "lcd_set_state failed"))
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd freecad-addon && python -m pytest tests/test_lcd_client.py -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/lcd_client.py freecad-addon/tests/test_lcd_client.py
git commit -m "feat(addon): LcdClient.set_state for lcd_set_state command"
```

---

## Task 3: Read spaceball button bindings

`bindings.read_bindings()` returns 12 command names (button indices 0–11), `""` for unbound. FreeCAD access is dependency-injected via `param_get` so tests need no FreeCAD.

**Files:**
- Create: `freecad-addon/space_elevator/bindings.py`
- Test: `freecad-addon/tests/test_bindings.py`

- [ ] **Step 1: Write the failing test**

Create `freecad-addon/tests/test_bindings.py`:

```python
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd freecad-addon && python -m pytest tests/test_bindings.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'space_elevator.bindings'`

- [ ] **Step 3: Implement `bindings.py`**

Create `freecad-addon/space_elevator/bindings.py`:

```python
"""Read FreeCAD's spaceball button -> command bindings.

Bindings live at ``BaseApp/Spaceball/Buttons/<N>/Command`` (flat indices,
0-based). Indices 0-11 are the SpaceMouse Enterprise LCD function keys 1-12.
FreeCAD access is injected so this is unit-testable without FreeCAD.
"""

BUTTONS_PATH = "User parameter:BaseApp/Spaceball/Buttons/%d"
LCD_BUTTON_COUNT = 12


def _default_param_get(path):
    import FreeCAD
    return FreeCAD.ParamGet(path)


def read_bindings(param_get=None, count=LCD_BUTTON_COUNT):
    """Return ``count`` command names by button index; "" for unbound."""
    param_get = param_get or _default_param_get
    out = []
    for i in range(count):
        group = param_get(BUTTONS_PATH % i)
        out.append(group.GetString("Command", ""))
    return out
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd freecad-addon && python -m pytest tests/test_bindings.py -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/bindings.py freecad-addon/tests/test_bindings.py
git commit -m "feat(addon): read spaceball button bindings from params"
```

---

## Task 4: Resolve a command's label and icon

`commands.resolve(name)` → `{"label", "icon"}`. `commands.read_icon(pixmap)` → SVG string, `data:` PNG URI, or `None`. Both inject FreeCAD/Qt so tests need neither.

**Files:**
- Create: `freecad-addon/space_elevator/commands.py`
- Test: `freecad-addon/tests/test_commands.py`

- [ ] **Step 1: Write the failing test**

Create `freecad-addon/tests/test_commands.py`:

```python
from space_elevator import commands


def test_read_icon_prefers_svg_resource():
    def file_factory(path):
        assert path == ":/icons/zoom-all.svg"
        return b"<svg>vector</svg>"

    icon = commands.read_icon("zoom-all", file_factory=file_factory, icon_factory=None)
    assert icon == "<svg>vector</svg>"


def test_read_icon_falls_back_to_png_data_uri():
    def file_factory(path):
        return None  # no svg resource

    def icon_factory(path):
        return "QUJD"  # fake base64

    icon = commands.read_icon("weird-cmd", file_factory=file_factory, icon_factory=icon_factory)
    assert icon == "data:image/png;base64,QUJD"


def test_read_icon_none_when_no_pixmap():
    assert commands.read_icon("", file_factory=lambda p: None, icon_factory=lambda p: None) is None


def test_resolve_strips_accelerator_and_attaches_icon():
    class FakeCmd:
        def getInfo(self):
            return {"menuText": "&Fit all", "pixmap": "zoom-all"}

    def command_get(name):
        assert name == "Std_ViewFitAll"
        return FakeCmd()

    info = commands.resolve(
        "Std_ViewFitAll",
        command_get=command_get,
        icon_resolver=lambda px: "<svg/>",
    )
    assert info == {"label": "Fit all", "icon": "<svg/>"}


def test_resolve_unknown_command_uses_name_as_label():
    info = commands.resolve(
        "Bogus_Cmd",
        command_get=lambda name: None,
        icon_resolver=lambda px: None,
    )
    assert info == {"label": "Bogus_Cmd", "icon": None}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd freecad-addon && python -m pytest tests/test_commands.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'space_elevator.commands'`

- [ ] **Step 3: Implement `commands.py`**

Create `freecad-addon/space_elevator/commands.py`:

```python
"""Resolve a FreeCAD command name to a tile label + icon.

label  = command's menuText (with & accelerators stripped)
icon   = the command icon as an SVG string, or a data: PNG URI, or None

FreeCAD/Qt access is injected so this unit-tests without FreeCAD.
"""

SVG_RESOURCE = ":/icons/%s.svg"


def _default_file(path):
    """Return raw bytes of a Qt resource, or None if it can't be opened."""
    from PySide6 import QtCore
    f = QtCore.QFile(path)
    if not f.open(QtCore.QIODevice.ReadOnly):
        return None
    try:
        return bytes(f.readAll())
    finally:
        f.close()


def _default_icon_b64(path):
    """Rasterize a Qt icon resource to a 48x48 PNG, return base64 str or None."""
    from PySide6 import QtCore, QtGui
    icon = QtGui.QIcon(path)
    if icon.isNull():
        return None
    pm = icon.pixmap(48, 48)
    ba = QtCore.QByteArray()
    buf = QtCore.QBuffer(ba)
    buf.open(QtCore.QIODevice.WriteOnly)
    if not pm.save(buf, "PNG"):
        buf.close()
        return None
    buf.close()
    return bytes(ba.toBase64()).decode()


def read_icon(pixmap, file_factory=None, icon_factory=None):
    """Resolve a pixmap resource name to SVG text, a data: PNG URI, or None."""
    if not pixmap:
        return None
    file_factory = file_factory or _default_file
    icon_factory = icon_factory or _default_icon_b64

    res = SVG_RESOURCE % pixmap
    data = file_factory(res)
    if data:
        return data.decode("utf-8", "replace") if isinstance(data, (bytes, bytearray)) else data

    b64 = icon_factory(res)
    if b64:
        return "data:image/png;base64," + b64
    return None


def _default_command_get(name):
    import FreeCADGui
    return FreeCADGui.Command.get(name)


def resolve(name, command_get=None, icon_resolver=None):
    """Return {"label", "icon"} for a command name. Unknown -> name as label."""
    command_get = command_get or _default_command_get
    icon_resolver = icon_resolver or read_icon

    cmd = command_get(name)
    if cmd is None:
        return {"label": name, "icon": None}
    info = cmd.getInfo()
    label = info.get("menuText", name).replace("&", "")
    icon = icon_resolver(info.get("pixmap", ""))
    return {"label": label, "icon": icon}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd freecad-addon && python -m pytest tests/test_commands.py -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/commands.py freecad-addon/tests/test_commands.py
git commit -m "feat(addon): resolve command label + icon (svg/png)"
```

---

## Task 5: Assemble the `lcd_set_state` payload

`state.build_state()` composes the 12 bindings + their resolved label/icon into the daemon dict (`profile`, `mode`, `left` 6, `right` 6). Unbound → blank tile (`{"label":"","icon":None}`), which Task 1 renders as nothing.

**Files:**
- Create: `freecad-addon/space_elevator/state.py`
- Test: `freecad-addon/tests/test_state.py`

- [ ] **Step 1: Write the failing test**

Create `freecad-addon/tests/test_state.py`:

```python
from space_elevator import state


def test_build_state_splits_left_right_and_blanks_unbound():
    names = [f"Cmd{i}" if i not in (3, 7) else "" for i in range(12)]

    def read_bindings():
        return names

    def resolve(name):
        return {"label": name.upper(), "icon": f"<svg>{name}</svg>"}

    def active_wb():
        return "SketcherWorkbench"

    result = state.build_state(read_bindings=read_bindings, resolve=resolve, active_wb=active_wb)

    assert result["profile"] == "FreeCAD"
    assert result["mode"] == "SketcherWorkbench"
    assert len(result["left"]) == 6 and len(result["right"]) == 6

    # bound tile carries label + icon
    assert result["left"][0] == {"label": "CMD0", "icon": "<svg>Cmd0</svg>", "active": False}
    # unbound index 3 -> blank tile
    assert result["left"][3] == {"label": "", "icon": None, "active": False}
    # right cluster starts at index 6; index 7 unbound
    assert result["right"][1] == {"label": "", "icon": None, "active": False}


def test_build_state_unknown_resolve_none_uses_name():
    def read_bindings():
        return ["Mystery"] + [""] * 11

    def resolve(name):
        return {"label": name, "icon": None}

    result = state.build_state(read_bindings=read_bindings, resolve=resolve, active_wb=lambda: "PartWorkbench")
    assert result["left"][0] == {"label": "Mystery", "icon": None, "active": False}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd freecad-addon && python -m pytest tests/test_state.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'space_elevator.state'`

- [ ] **Step 3: Implement `state.py`**

Create `freecad-addon/space_elevator/state.py`:

```python
"""Build the lcd_set_state payload from FreeCAD's current bindings + workbench.

profile is fixed "FreeCAD"; mode is the active workbench. Twelve button
bindings (indices 0-11) become two 6-tile clusters. Unbound buttons become
blank tiles (the daemon renders them as nothing). No category yet -> neutral.
"""

from space_elevator import bindings as _bindings
from space_elevator import commands as _commands

PROFILE = "FreeCAD"


def _default_active_wb():
    import FreeCADGui
    return FreeCADGui.activeWorkbench().name()


def _blank_tile():
    return {"label": "", "icon": None, "active": False}


def build_state(read_bindings=None, resolve=None, active_wb=None):
    read_bindings = read_bindings or _bindings.read_bindings
    resolve = resolve or _commands.resolve
    active_wb = active_wb or _default_active_wb

    tiles = []
    for name in read_bindings():
        if not name:
            tiles.append(_blank_tile())
            continue
        info = resolve(name)
        tiles.append({"label": info["label"], "icon": info["icon"], "active": False})

    return {
        "profile": PROFILE,
        "mode": active_wb(),
        "left": tiles[:6],
        "right": tiles[6:12],
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd freecad-addon && python -m pytest tests/test_state.py -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/state.py freecad-addon/tests/test_state.py
git commit -m "feat(addon): build lcd_set_state payload from bindings + workbench"
```

---

## Task 6: Service — push state, swallow daemon-down

`service.push_once()` builds state and sends it via `LcdClient`, returning `False` (not raising) when the daemon is unavailable. `service.start()` wires the workbench-activated signal and pushes once.

**Files:**
- Create: `freecad-addon/space_elevator/service.py`
- Test: `freecad-addon/tests/test_service.py`

- [ ] **Step 1: Write the failing test**

Create `freecad-addon/tests/test_service.py`:

```python
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd freecad-addon && python -m pytest tests/test_service.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'space_elevator.service'`

- [ ] **Step 3: Implement `service.py`**

Create `freecad-addon/space_elevator/service.py`:

```python
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd freecad-addon && python -m pytest tests/test_service.py -v`
Expected: PASS

- [ ] **Step 5: Run the full addon suite**

Run: `cd freecad-addon && python -m pytest -v`
Expected: PASS for `test_bindings`, `test_commands`, `test_state`, `test_service`, `test_lcd_client` (the obsolete tests are deleted in Task 7).

- [ ] **Step 6: Commit**

```bash
git add freecad-addon/space_elevator/service.py freecad-addon/tests/test_service.py
git commit -m "feat(addon): service wires workbench signal to LCD push"
```

---

## Task 7: Replace the addon entrypoints + delete the driver layer

Rewrite `InitGui.py` to a thin no-workbench bootstrap, blank `Init.py`, trim `package.xml`, and delete the obsolete driver modules + their tests.

**Files:**
- Rewrite: `freecad-addon/InitGui.py`
- Rewrite: `freecad-addon/Init.py`
- Modify: `freecad-addon/package.xml`
- Delete: driver modules, `preferences.ui`, obsolete tests (listed in File Structure)

- [ ] **Step 1: Delete the obsolete driver files**

```bash
cd freecad-addon
git rm space_elevator/event_pump.py space_elevator/navigator.py space_elevator/libspnav.py \
       space_elevator/axes.py space_elevator/buttons.py space_elevator/button_dialog.py \
       space_elevator/settings.py space_elevator/preferences.py space_elevator/lifecycle.py \
       Resources/ui/preferences.ui \
       tests/test_axes.py tests/test_buttons.py tests/test_libspnav_open.py \
       tests/test_libspnav_types.py tests/test_settings.py
```

- [ ] **Step 2: Rewrite `InitGui.py`**

Replace the entire contents of `freecad-addon/InitGui.py` with:

```python
"""Space Elevator — thin LCD client for FreeCAD.

No workbench, no spnav. On GUI startup it connects the workbench-activated
signal and pushes the current button bindings to space-elevatord for display
on the SpaceMouse Enterprise LCD. All LCD output is best-effort.
"""

import FreeCAD

try:
    from space_elevator import service
    service.start()
    FreeCAD.Console.PrintMessage("Space Elevator: LCD client active\n")
except Exception as exc:  # never break FreeCAD startup over the LCD
    FreeCAD.Console.PrintWarning(f"Space Elevator: LCD client disabled ({exc})\n")
```

- [ ] **Step 3: Rewrite `Init.py`**

Replace the entire contents of `freecad-addon/Init.py` with:

```python
# Space Elevator is GUI-only (LCD client). No console-mode initialization.
```

- [ ] **Step 4: Trim `package.xml`**

Replace the `<content>...</content>` block in `freecad-addon/package.xml` (the workbench is gone) and bump the version. New file contents:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<package format="1" xmlns="https://wiki.freecad.org/Package_Metadata">
  <name>Space Elevator</name>
  <description>SpaceMouse Enterprise LCD client for FreeCAD (pushes button bindings to space-elevatord).</description>
  <version>0.2.0</version>
  <maintainer email="christopher@muehl.dev">Christopher Mühl</maintainer>
  <license>MIT</license>
  <url type="repository" branch="main">https://github.com/tophcodes/space-elevator</url>
  <depend condition="$installed_FreeCAD &gt; '1.0'">FreeCAD</depend>
</package>
```

- [ ] **Step 5: Verify the suite still passes (no imports of deleted modules)**

Run: `cd freecad-addon && python -m pytest -v`
Expected: PASS, and no `ImportError` referencing deleted modules. If any kept file still imports a deleted module, remove that import.

- [ ] **Step 6: Commit**

```bash
git add -A freecad-addon
git commit -m "refactor(addon): thin LCD client, drop obsolete driver layer"
```

---

## Task 8: Manual end-to-end verification (FreeCAD GUI + hardware)

No automated coverage of the Qt signal / live device — verify by hand.

**Files:** none (manual).

- [ ] **Step 1: Install/symlink the addon into FreeCAD's Mod dir**

```bash
ln -sfn "$PWD/freecad-addon" ~/.local/share/FreeCAD/Mod/SpaceElevator
```
(Use the version-specific dir `~/.local/share/FreeCAD/Mod` per this FreeCAD 1.1 install. If FreeCAD loads from `v1-1/Mod`, symlink there instead.)

- [ ] **Step 2: Start the daemon (built binary, sudo for USB)**

Run (Terminal 1): `sudo XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR ./target/debug/space-elevatord`
Expected: `space-elevatord listening`. (Build first as user with `cargo build -p space-elevatord` so the root run needs no devenv/libudev.)

- [ ] **Step 3: Launch FreeCAD, confirm the client loads**

Open FreeCAD. In the Report view / Python console, expect: `Space Elevator: LCD client active`.
Expected on the LCD: the 12 current button bindings, profile `FREECAD`, mode = active workbench. Unbound buttons show as blank tiles.

- [ ] **Step 4: Switch workbenches**

Change workbench (e.g. Part → Sketcher).
Expected: the LCD `mode` text updates to the new workbench name on each switch.

- [ ] **Step 5: Rebind a button, confirm refresh**

Tools → Customize → Spaceball Buttons → change one of buttons 1–12. Switch workbench once (to trigger a push) and confirm the tile's label/icon updates.

- [ ] **Step 6: Daemon-down resilience**

Stop the daemon (Ctrl-C). Switch workbenches in FreeCAD.
Expected: no error dialog, no traceback in the console — just the `LCD client disabled`/silent path. Restart daemon, switch workbench, LCD updates again.

- [ ] **Step 7: Clean up the leftover driver param (one-time)**

In the FreeCAD Python console:
```python
import FreeCAD
FreeCAD.ParamGet("User parameter:BaseApp/Preferences/Mod").RemGroup("SpaceElevator")
```
Expected: removes the abandoned `config_json` blob from the old driver. (Cosmetic; safe to skip.)

---

## After all tasks

- Final review of the branch, then `superpowers:finishing-a-development-branch` (PR → main), same flow as the daemon branch.
- Out of scope (future): per-workbench bindings (upstream FreeCAD C++), curated command→`cat` colour map, long-press tiles, Onshape web client.

## Self-review notes

- **Spec coverage:** thin-plugin section of `2026-06-02-lcd-service-pivot-design.md` — startup + workbench-activated push (T6/T7), read global bindings (T3), label=menuText + icon=command SVG (T4), `set_state` via `LcdClient` (T2), silent on daemon-down (T6), neutral `cat` (T5). The spec's guessed param path is corrected per the spike (T3).
- **Open spec items resolved by spike:** param storage (T3), command-icon resolution (T4). Per-workbench palettes remain future (noted).
- **Type consistency:** `read_bindings`/`resolve`/`build_state`/`push_once`/`set_state` signatures and the tile dict shape `{"label","icon","active"}` match across tasks; tile shape matches the daemon's serde `Tile` (`label`, `icon` Option, `cat` optional/omitted, `active`).
