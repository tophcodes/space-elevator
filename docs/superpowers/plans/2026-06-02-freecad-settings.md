# FreeCAD Settings Dialog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the addon's split settings (FreeCAD preferences page + separate bindings dialog) with one live-applying custom dialog backed by a single JSON-serializable `Config` object.

**Architecture:** A FreeCAD-free core (`config`, `axes`, `bindings`, `store`) holds and (de)serializes all state and does pure transforms; thin FreeCAD adapters (`dialog`, `lifecycle`, `navigator`) wire it to the GUI. The event hot path reads an in-memory `Config`; ParamGet is touched only on load/save. Bindings resolve as override-if-set-else-global per workbench.

**Tech Stack:** Python, FreeCAD/PySide6 (PySide2 fallback), pytest. The core modules import no FreeCAD and are unit-tested; GUI modules are verified manually in FreeCAD.

**Conventions:** Tests run from `freecad-addon/` inside the devenv shell:
`devenv shell -- bash -c 'cd freecad-addon && python -m pytest -q'`.
After GUI-affecting tasks, re-sync the working copy into FreeCAD's Mod dir (Task 10) and restart FreeCAD to verify.

---

## File Structure

Core (FreeCAD-free, unit-tested):
- Create `freecad-addon/space_elevator/config.py` — `Config` dataclass + `default`/`to_dict`/`from_dict` + `effective_bindings`.
- Modify `freecad-addon/space_elevator/axes.py` — extend `transform` for sensitivity, per-axis `enabled` lock, `dominant_axis`.
- Create `freecad-addon/space_elevator/bindings.py` — pure label extraction helper (resolution lives on `Config`).
- Create `freecad-addon/space_elevator/store.py` — `Config` ↔ ParamGet (`config_json`) + JSON file export/import.

FreeCAD adapters (manual test):
- Modify `freecad-addon/space_elevator/navigator.py` — take pan/rot speed as args.
- Modify `freecad-addon/space_elevator/libspnav.py` — idempotent `close()`.
- Rewrite `freecad-addon/space_elevator/lifecycle.py` — own live `Config`, workbench-change wiring, handler reads `Config`.
- Create `freecad-addon/space_elevator/dialog.py` — unified Qt dialog (replaces `button_dialog.py`).
- Modify `freecad-addon/InitGui.py` — single `Settings…` menu command; drop preferences-page registration.

Removed:
- `freecad-addon/space_elevator/preferences.py`, `freecad-addon/Resources/ui/preferences.ui`
- `freecad-addon/space_elevator/button_dialog.py`
- `freecad-addon/space_elevator/buttons.py`, `freecad-addon/space_elevator/settings.py`
- `freecad-addon/tests/test_buttons.py`, `freecad-addon/tests/test_settings.py`

---

## Task 1: Config data model

**Files:**
- Create: `freecad-addon/space_elevator/config.py`
- Test: `freecad-addon/tests/test_config.py`

- [ ] **Step 1: Write the failing test**

```python
# freecad-addon/tests/test_config.py
import pytest
from space_elevator.config import Config, AXES, CONFIG_VERSION


def test_default_has_six_axes_at_unity():
    c = Config.default()
    assert set(c.axes) == set(AXES)
    for a in AXES:
        assert c.axes[a].scale == 1.0
        assert c.axes[a].invert is False
        assert c.axes[a].enabled is True
    assert c.sensitivity == 1.0
    assert c.pan_speed == 60.0
    assert c.rot_speed == 1.2
    assert c.deadzone == 0.02
    assert c.dominant_axis is False
    assert c.lcd_enabled is True


def test_to_from_dict_roundtrip():
    c = Config.default()
    c.sensitivity = 2.0
    c.axes["tx"].invert = True
    c.axes["rz"].enabled = False
    c.global_bindings = {0: "Std_ViewFit"}
    c.wb_overrides = {"SketcherWorkbench": {2: "Sketcher_NewSketch"}}
    d = c.to_dict()
    assert d["version"] == CONFIG_VERSION
    assert d["bindings"]["global"] == {"0": "Std_ViewFit"}
    assert d["bindings"]["overrides"]["SketcherWorkbench"] == {"2": "Sketcher_NewSketch"}
    c2 = Config.from_dict(d)
    assert c2.sensitivity == 2.0
    assert c2.axes["tx"].invert is True
    assert c2.axes["rz"].enabled is False
    assert c2.global_bindings == {0: "Std_ViewFit"}
    assert c2.wb_overrides == {"SketcherWorkbench": {2: "Sketcher_NewSketch"}}


def test_from_dict_fills_missing_with_defaults():
    c = Config.from_dict({"version": 1, "navigation": {"sensitivity": 3.0}})
    assert c.sensitivity == 3.0
    assert c.deadzone == 0.02
    assert c.axes["tx"].scale == 1.0


def test_from_dict_rejects_newer_version():
    with pytest.raises(ValueError):
        Config.from_dict({"version": CONFIG_VERSION + 1})


def test_effective_bindings_override_else_global():
    c = Config.default()
    c.global_bindings = {0: "Std_ViewFit", 1: "Std_New"}
    c.wb_overrides = {"PartWorkbench": {1: "Part_Box", 2: "Part_Cylinder"}}
    eff = c.effective_bindings("PartWorkbench")
    assert eff == {0: "Std_ViewFit", 1: "Part_Box", 2: "Part_Cylinder"}
    assert c.effective_bindings("OtherWorkbench") == {0: "Std_ViewFit", 1: "Std_New"}


def test_effective_bindings_ignores_empty_strings():
    c = Config.default()
    c.global_bindings = {0: "Std_ViewFit", 1: ""}
    c.wb_overrides = {"PartWorkbench": {0: ""}}
    assert c.effective_bindings("PartWorkbench") == {0: "Std_ViewFit"}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_config.py -q'`
Expected: FAIL with `ModuleNotFoundError: No module named 'space_elevator.config'`

- [ ] **Step 3: Write minimal implementation**

```python
# freecad-addon/space_elevator/config.py
"""Single source of truth for addon settings. FreeCAD-free and JSON-serializable.

The in-memory form uses int button numbers; the JSON form uses string keys
(JSON object keys must be strings).
"""

from dataclasses import dataclass, field

AXES = ("tx", "ty", "tz", "rx", "ry", "rz")
CONFIG_VERSION = 1


@dataclass
class AxisConfig:
    scale: float = 1.0
    invert: bool = False
    enabled: bool = True


@dataclass
class Config:
    sensitivity: float = 1.0
    pan_speed: float = 60.0
    rot_speed: float = 1.2
    deadzone: float = 0.02
    dominant_axis: bool = False
    axes: dict = field(default_factory=lambda: {a: AxisConfig() for a in AXES})
    lcd_enabled: bool = True
    daemon_socket_path: str = ""
    global_bindings: dict = field(default_factory=dict)   # {int bnum: str cmd}
    wb_overrides: dict = field(default_factory=dict)        # {str wb: {int bnum: str cmd}}

    @staticmethod
    def default():
        return Config()

    def to_dict(self):
        return {
            "version": CONFIG_VERSION,
            "navigation": {
                "sensitivity": self.sensitivity,
                "pan_speed": self.pan_speed,
                "rot_speed": self.rot_speed,
                "deadzone": self.deadzone,
                "dominant_axis": self.dominant_axis,
                "axes": {
                    a: {"scale": ax.scale, "invert": ax.invert, "enabled": ax.enabled}
                    for a, ax in self.axes.items()
                },
            },
            "lcd": {"enabled": self.lcd_enabled, "daemon_socket_path": self.daemon_socket_path},
            "bindings": {
                "global": {str(b): c for b, c in self.global_bindings.items()},
                "overrides": {
                    wb: {str(b): c for b, c in m.items()}
                    for wb, m in self.wb_overrides.items()
                },
            },
        }

    @staticmethod
    def from_dict(d):
        version = d.get("version", CONFIG_VERSION)
        if version > CONFIG_VERSION:
            raise ValueError(f"unsupported config version {version}")
        c = Config()
        nav = d.get("navigation", {})
        c.sensitivity = nav.get("sensitivity", c.sensitivity)
        c.pan_speed = nav.get("pan_speed", c.pan_speed)
        c.rot_speed = nav.get("rot_speed", c.rot_speed)
        c.deadzone = nav.get("deadzone", c.deadzone)
        c.dominant_axis = nav.get("dominant_axis", c.dominant_axis)
        for a, ax in nav.get("axes", {}).items():
            if a in c.axes:
                c.axes[a] = AxisConfig(
                    scale=ax.get("scale", 1.0),
                    invert=ax.get("invert", False),
                    enabled=ax.get("enabled", True),
                )
        lcd = d.get("lcd", {})
        c.lcd_enabled = lcd.get("enabled", c.lcd_enabled)
        c.daemon_socket_path = lcd.get("daemon_socket_path", c.daemon_socket_path)
        binds = d.get("bindings", {})
        c.global_bindings = {int(b): cmd for b, cmd in binds.get("global", {}).items()}
        c.wb_overrides = {
            wb: {int(b): cmd for b, cmd in m.items()}
            for wb, m in binds.get("overrides", {}).items()
        }
        return c

    def effective_bindings(self, workbench):
        eff = {b: c for b, c in self.global_bindings.items() if c}
        for b, c in self.wb_overrides.get(workbench, {}).items():
            if c:
                eff[b] = c
        return eff
```

- [ ] **Step 4: Run test to verify it passes**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_config.py -q'`
Expected: PASS (6 passed)

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/config.py freecad-addon/tests/test_config.py
git commit -m "feat(addon): JSON-serializable Config data model"
```

---

## Task 2: Pure label extraction (bindings.py)

**Files:**
- Create: `freecad-addon/space_elevator/bindings.py`
- Test: `freecad-addon/tests/test_bindings.py`

Note: binding *resolution* lives on `Config.effective_bindings`. This module only turns an effective binding map into the ordered label list the LCD renders.

- [ ] **Step 1: Write the failing test**

```python
# freecad-addon/tests/test_bindings.py
from space_elevator.bindings import labels_for


def test_labels_for_strips_command_prefix_and_orders():
    eff = {0: "Std_ViewFit", 2: "Sketcher_NewSketch"}
    assert labels_for(eff, count=4) == ["ViewFit", "", "NewSketch", ""]


def test_labels_for_command_without_prefix_kept_whole():
    assert labels_for({0: "ViewFit"}, count=2) == ["ViewFit", ""]


def test_labels_for_empty():
    assert labels_for({}, count=3) == ["", "", ""]
```

- [ ] **Step 2: Run test to verify it fails**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_bindings.py -q'`
Expected: FAIL with `ModuleNotFoundError: No module named 'space_elevator.bindings'`

- [ ] **Step 3: Write minimal implementation**

```python
# freecad-addon/space_elevator/bindings.py
"""Turn an effective binding map into an ordered list of short labels for the LCD."""


def _short_name(command):
    if "_" in command:
        return command.split("_", 1)[1]
    return command


def labels_for(effective, count):
    """effective: {int bnum: command}. Returns `count` labels, index == button number."""
    return [_short_name(effective.get(i, "")) if effective.get(i) else "" for i in range(count)]
```

- [ ] **Step 4: Run test to verify it passes**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_bindings.py -q'`
Expected: PASS (3 passed)

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/bindings.py freecad-addon/tests/test_bindings.py
git commit -m "feat(addon): pure LCD label extraction"
```

---

## Task 3: Extend axes.transform (sensitivity, lock, dominant)

**Files:**
- Modify: `freecad-addon/space_elevator/axes.py`
- Test: `freecad-addon/tests/test_axes.py` (replace contents)

- [ ] **Step 1: Replace the test file with the new signature and cases**

```python
# freecad-addon/tests/test_axes.py
from space_elevator.axes import transform

_ON = (True,) * 6
_NOINV = (False,) * 6
_UNIT = (1.0,) * 6


def test_deadzone_suppresses_small_values():
    raw = (5, 5, 5, 5, 5, 5)  # inside 0.02 * 350 = 7
    out = transform(raw, _UNIT, _NOINV, _ON, deadzone=0.02, max_raw=350)
    assert out == (0.0,) * 6


def test_scale_applied_after_deadzone():
    raw = (350, 0, 0, 0, 0, 0)
    out = transform(raw, (2.0, 1, 1, 1, 1, 1), _NOINV, _ON, deadzone=0.02, max_raw=350)
    assert abs(out[0] - 2.0) < 1e-6


def test_invert_flips_sign():
    raw = (350, 0, 0, 0, 0, 0)
    out = transform(raw, _UNIT, (True, False, False, False, False, False), _ON, deadzone=0.0)
    assert out[0] == -1.0


def test_sensitivity_multiplies_all():
    raw = (350, 350, 0, 0, 0, 0)
    out = transform(raw, _UNIT, _NOINV, _ON, deadzone=0.0, sensitivity=0.5)
    assert abs(out[0] - 0.5) < 1e-6
    assert abs(out[1] - 0.5) < 1e-6


def test_disabled_axis_is_muted():
    raw = (350, 350, 0, 0, 0, 0)
    enabled = (False, True, True, True, True, True)
    out = transform(raw, _UNIT, _NOINV, enabled, deadzone=0.0)
    assert out[0] == 0.0
    assert abs(out[1] - 1.0) < 1e-6


def test_dominant_axis_keeps_only_strongest():
    raw = (350, 100, 0, 0, 0, 0)
    out = transform(raw, _UNIT, _NOINV, _ON, deadzone=0.0, dominant_axis=True)
    assert abs(out[0] - 1.0) < 1e-6
    assert out[1] == 0.0
```

- [ ] **Step 2: Run test to verify it fails**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_axes.py -q'`
Expected: FAIL (TypeError on the new `enabled` positional / unknown kwargs)

- [ ] **Step 3: Replace axes.py with the extended transform**

```python
# freecad-addon/space_elevator/axes.py
def transform(raw, scales, inverts, enabled, deadzone, sensitivity=1.0,
              dominant_axis=False, max_raw=350):
    """Map 6 raw libspnav axes to scaled floats (1.0 == full deflection).

    raw/scales/inverts/enabled: 6-tuples. A disabled axis is muted to 0.
    deadzone: fraction of max_raw below which a value clamps to 0.
    sensitivity: overall multiplier applied to every axis.
    dominant_axis: if True, keep only the largest-magnitude axis, zero the rest.
    """
    dz = deadzone * max_raw
    out = []
    for v, s, inv, en in zip(raw, scales, inverts, enabled):
        if not en or -dz < v < dz:
            out.append(0.0)
            continue
        norm = v / max_raw
        if inv:
            norm = -norm
        out.append(norm * s * sensitivity)
    result = tuple(out)
    if dominant_axis:
        result = _dominant(result)
    return result


def _dominant(axes):
    strongest = max(range(len(axes)), key=lambda i: abs(axes[i]))
    return tuple(v if i == strongest else 0.0 for i, v in enumerate(axes))
```

- [ ] **Step 4: Run test to verify it passes**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_axes.py -q'`
Expected: PASS (6 passed)

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/axes.py freecad-addon/tests/test_axes.py
git commit -m "feat(addon): axes transform honors sensitivity, lock, dominant-axis"
```

---

## Task 4: Persistence + export/import (store.py)

**Files:**
- Create: `freecad-addon/space_elevator/store.py`
- Test: `freecad-addon/tests/test_store.py`

- [ ] **Step 1: Write the failing test**

```python
# freecad-addon/tests/test_store.py
import json
import pytest
from space_elevator import store
from space_elevator.config import Config


class FakeGroup:
    def __init__(self):
        self._strings = {}
    def GetString(self, k, d): return self._strings.get(k, d)
    def SetString(self, k, v): self._strings[k] = v


def fake_param_get():
    g = FakeGroup()
    return lambda path: g


def test_load_default_when_empty():
    c = store.load(fake_param_get())
    assert c.sensitivity == 1.0


def test_save_then_load_roundtrip():
    pg = fake_param_get()
    c = Config.default()
    c.sensitivity = 1.5
    c.global_bindings = {0: "Std_ViewFit"}
    store.save(pg, c)
    loaded = store.load(pg)
    assert loaded.sensitivity == 1.5
    assert loaded.global_bindings == {0: "Std_ViewFit"}


def test_load_tolerates_garbage():
    pg = fake_param_get()
    pg("x").SetString("config_json", "{not valid json")
    assert store.load(pg).sensitivity == 1.0


def test_export_import_file_roundtrip(tmp_path):
    c = Config.default()
    c.pan_speed = 99.0
    path = tmp_path / "profile.json"
    store.export_to_file(c, str(path))
    on_disk = json.loads(path.read_text())
    assert on_disk["navigation"]["pan_speed"] == 99.0
    c2 = store.import_from_file(str(path))
    assert c2.pan_speed == 99.0


def test_import_rejects_newer_version(tmp_path):
    path = tmp_path / "bad.json"
    path.write_text(json.dumps({"version": 999}))
    with pytest.raises(ValueError):
        store.import_from_file(str(path))
```

- [ ] **Step 2: Run test to verify it fails**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_store.py -q'`
Expected: FAIL with `ModuleNotFoundError: No module named 'space_elevator.store'`

- [ ] **Step 3: Write minimal implementation**

```python
# freecad-addon/space_elevator/store.py
"""Persist Config as one JSON string in ParamGet; export/import as a JSON file."""

import json

from .config import Config

_PATH = "User parameter:BaseApp/Preferences/Mod/SpaceElevator"
_KEY = "config_json"


def load(param_get):
    grp = param_get(_PATH)
    raw = grp.GetString(_KEY, "")
    if not raw:
        return Config.default()
    try:
        return Config.from_dict(json.loads(raw))
    except (ValueError, json.JSONDecodeError):
        return Config.default()


def save(param_get, config):
    grp = param_get(_PATH)
    grp.SetString(_KEY, json.dumps(config.to_dict()))


def export_to_file(config, path):
    with open(path, "w") as f:
        json.dump(config.to_dict(), f, indent=2)


def import_from_file(path):
    with open(path) as f:
        return Config.from_dict(json.load(f))  # raises ValueError on bad version
```

- [ ] **Step 4: Run test to verify it passes**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_store.py -q'`
Expected: PASS (5 passed)

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/store.py freecad-addon/tests/test_store.py
git commit -m "feat(addon): config persistence + JSON export/import"
```

---

## Task 5: Navigator takes speeds as arguments

**Files:**
- Modify: `freecad-addon/space_elevator/navigator.py`

No unit test (imports FreeCADGui). Verified via Task 10.

- [ ] **Step 1: Replace the tunable constants and `apply` signature**

Replace the top constants block and `apply` in `navigator.py` with:

```python
"""Apply transformed SpaceMouse motion to the active FreeCAD 3D view.

Translate the camera in view-local coordinates for tx/ty/tz, rotate about the
camera's local axes for rx/ry/rz. Speeds are passed in (from Config) so they
are live-tunable. `period_ms` from libspnav is the per-event timestep.
"""

import FreeCADGui as Gui


def _active_camera():
    doc = Gui.ActiveDocument
    if doc is None:
        return None
    view = doc.ActiveView
    if view is None:
        return None
    return view.getCameraNode()


def apply(transformed_axes, period_ms, pan_speed, rot_speed):
    cam = _active_camera()
    if cam is None:
        return
    tx, ty, tz, rx, ry, rz = transformed_axes
    dt = max(period_ms, 1) / 1000.0

    pan_factor = pan_speed * dt
    rot_factor = rot_speed * dt

    _translate(cam, tx * pan_factor, ty * pan_factor, tz * pan_factor)
    _rotate(cam, rx * rot_factor, ry * rot_factor, rz * rot_factor)
```

Leave `_translate` and `_rotate` unchanged.

- [ ] **Step 2: Verify core tests still pass (no regression)**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest -q'`
Expected: PASS (config/bindings/axes/store + libspnav tests; no navigator import in tests)

- [ ] **Step 3: Commit**

```bash
git add freecad-addon/space_elevator/navigator.py
git commit -m "feat(addon): navigator pan/rot speed from config"
```

---

## Task 6: Idempotent libspnav close

**Files:**
- Modify: `freecad-addon/space_elevator/libspnav.py:78-80` (the `close` method)

This kills the spurious "spnav_open failed" seen when re-activating the workbench (close was unconditional; a stale open state then made the next open return -1).

- [ ] **Step 1: Replace the `close` method**

Find in `_Lib`:

```python
    def close(self):
        self._lib.spnav_close()
        self._opened = False
```

Replace with:

```python
    def close(self):
        if self._opened:
            self._lib.spnav_close()
            self._opened = False
```

- [ ] **Step 2: Verify existing libspnav tests still pass**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest tests/test_libspnav_open.py tests/test_libspnav_types.py -q'`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add freecad-addon/space_elevator/libspnav.py
git commit -m "fix(addon): idempotent libspnav close"
```

---

## Task 7: Lifecycle owns live Config

**Files:**
- Rewrite: `freecad-addon/space_elevator/lifecycle.py`

No unit test (imports FreeCAD). Verified via Task 10.

- [ ] **Step 1: Replace lifecycle.py**

```python
# freecad-addon/space_elevator/lifecycle.py
import FreeCAD
import FreeCADGui as Gui

from .event_pump import EventPump
from . import libspnav, axes, navigator, store, bindings, lcd_client, lcd_labels
from .config import AXES

N_BUTTONS = 31

_state = {
    "active": False,
    "pump": None,
    "config": None,
    "effective": {},          # {int bnum: command} for the current workbench
    "wb_callback": None,
}


def _current_workbench():
    try:
        return Gui.activeWorkbench().__class__.__name__
    except Exception:
        return ""


def _make_handler():
    def handle(ev):
        cfg = _state["config"]
        if cfg is None:
            return
        kind = ev[0]
        if kind == "motion":
            _, raw, period = ev
            scales = tuple(cfg.axes[a].scale for a in AXES)
            inverts = tuple(cfg.axes[a].invert for a in AXES)
            enabled = tuple(cfg.axes[a].enabled for a in AXES)
            t = axes.transform(raw, scales, inverts, enabled, cfg.deadzone,
                               cfg.sensitivity, cfg.dominant_axis)
            navigator.apply(t, period, cfg.pan_speed, cfg.rot_speed)
        elif kind == "button":
            _, bnum, pressed = ev
            if not pressed:
                return
            cmd = _state["effective"].get(bnum)
            if cmd:
                try:
                    Gui.runCommand(cmd)
                except Exception as exc:
                    FreeCAD.Console.PrintError(
                        f"Space Elevator: command {cmd!r} failed: {exc}\n")
    return handle


def _push_labels():
    cfg = _state["config"]
    if cfg is None or not cfg.lcd_enabled:
        return
    client = lcd_client.LcdClient(cfg.daemon_socket_path or None)
    try:
        client.connect()
    except lcd_client.DaemonUnavailable:
        return  # daemon optional
    try:
        labels = bindings.labels_for(_state["effective"], count=8)
        client.display_svg(lcd_labels.render(labels))
    except lcd_client.ProtocolError as exc:
        FreeCAD.Console.PrintWarning(f"Space Elevator LCD: {exc}\n")
    finally:
        client.close()


def _resolve_and_push():
    cfg = _state["config"]
    if cfg is None:
        return
    _state["effective"] = cfg.effective_bindings(_current_workbench())
    _push_labels()


def on_config_changed(config):
    """Called by the settings dialog after any edit. Persists + applies live."""
    _state["config"] = config
    store.save(FreeCAD.ParamGet, config)
    if _state["active"]:
        _resolve_and_push()


def _on_workbench_activated(_name=None):
    if _state["active"]:
        _resolve_and_push()


def get_config():
    if _state["config"] is None:
        _state["config"] = store.load(FreeCAD.ParamGet)
    return _state["config"]


def on_activate():
    if _state["active"]:
        return
    _state["config"] = store.load(FreeCAD.ParamGet)
    try:
        pump = EventPump(_make_handler())
    except libspnav.LibspnavMissing as e:
        FreeCAD.Console.PrintError(f"Space Elevator: {e}\n")
        return
    if not pump.start():
        return
    _state.update(active=True, pump=pump)
    cb = _on_workbench_activated
    _state["wb_callback"] = cb
    try:
        Gui.getMainWindow().workbenchActivated.connect(cb)
    except Exception:
        pass  # signal name varies; labels still push on activate
    _resolve_and_push()


def on_deactivate():
    if not _state["active"]:
        return
    cb = _state.get("wb_callback")
    if cb is not None:
        try:
            Gui.getMainWindow().workbenchActivated.disconnect(cb)
        except Exception:
            pass
    pump = _state.pop("pump", None)
    if pump is not None:
        pump.stop()
    _state.update(active=False, wb_callback=None, effective={})
```

- [ ] **Step 2: Verify core tests still pass**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest -q'`
Expected: PASS (lifecycle not imported by tests)

- [ ] **Step 3: Commit**

```bash
git add freecad-addon/space_elevator/lifecycle.py
git commit -m "feat(addon): lifecycle owns live config + workbench-aware bindings"
```

Note: confirm `event_pump.EventPump(handler).start()/stop()` exists with this shape (it does today). If `EventPump` calls `libspnav.get().open()`, the idempotent `close` from Task 6 makes re-activation clean.

---

## Task 8: Unified settings dialog

**Files:**
- Create: `freecad-addon/space_elevator/dialog.py`

No unit test (Qt/FreeCAD GUI). Verified via Task 10.

- [ ] **Step 1: Write dialog.py**

```python
# freecad-addon/space_elevator/dialog.py
"""Modeless settings dialog. Edits mutate the live Config, persist, and apply
immediately via the on_changed callback (lifecycle.on_config_changed)."""

try:
    from PySide6 import QtWidgets
except ImportError:
    from PySide2 import QtWidgets

import FreeCAD
import FreeCADGui as Gui

from . import store
from .config import AXES

N_BUTTONS = 31


class SettingsDialog(QtWidgets.QDialog):
    def __init__(self, config, on_changed, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Space Elevator — Settings")
        self.resize(560, 640)
        self._cfg = config
        self._on_changed = on_changed
        self._command_names = sorted(Gui.listCommands())

        root = QtWidgets.QVBoxLayout(self)
        tabs = QtWidgets.QTabWidget(self)
        tabs.addTab(self._navigation_tab(), "Navigation")
        tabs.addTab(self._buttons_tab(), "Buttons")
        tabs.addTab(self._lcd_tab(), "LCD")
        root.addWidget(tabs)

        bar = QtWidgets.QHBoxLayout()
        export_btn = QtWidgets.QPushButton("Export…")
        import_btn = QtWidgets.QPushButton("Import…")
        close_btn = QtWidgets.QPushButton("Close")
        export_btn.clicked.connect(self._export)
        import_btn.clicked.connect(self._import)
        close_btn.clicked.connect(self.accept)
        bar.addWidget(export_btn)
        bar.addWidget(import_btn)
        bar.addStretch(1)
        bar.addWidget(close_btn)
        root.addLayout(bar)

    # --- shared ---
    def _changed(self):
        self._on_changed(self._cfg)

    def _double_spin(self, value, lo, hi, step):
        w = QtWidgets.QDoubleSpinBox()
        w.setRange(lo, hi)
        w.setSingleStep(step)
        w.setValue(value)
        return w

    # --- Navigation tab ---
    def _navigation_tab(self):
        page = QtWidgets.QWidget()
        form = QtWidgets.QFormLayout(page)

        sens = self._double_spin(self._cfg.sensitivity, 0.1, 10.0, 0.1)
        sens.valueChanged.connect(lambda v: (setattr(self._cfg, "sensitivity", v), self._changed()))
        form.addRow("Sensitivity", sens)

        pan = self._double_spin(self._cfg.pan_speed, 1.0, 500.0, 5.0)
        pan.valueChanged.connect(lambda v: (setattr(self._cfg, "pan_speed", v), self._changed()))
        form.addRow("Pan speed (mm/s)", pan)

        rot = self._double_spin(self._cfg.rot_speed, 0.1, 10.0, 0.1)
        rot.valueChanged.connect(lambda v: (setattr(self._cfg, "rot_speed", v), self._changed()))
        form.addRow("Rotate speed (rad/s)", rot)

        dz = self._double_spin(self._cfg.deadzone, 0.0, 0.5, 0.01)
        dz.valueChanged.connect(lambda v: (setattr(self._cfg, "deadzone", v), self._changed()))
        form.addRow("Deadzone", dz)

        dom = QtWidgets.QCheckBox()
        dom.setChecked(self._cfg.dominant_axis)
        dom.toggled.connect(lambda v: (setattr(self._cfg, "dominant_axis", v), self._changed()))
        form.addRow("Dominant axis only", dom)

        grid_box = QtWidgets.QGroupBox("Per-axis")
        grid = QtWidgets.QGridLayout(grid_box)
        for col, head in enumerate(("axis", "scale", "invert", "active")):
            grid.addWidget(QtWidgets.QLabel(head), 0, col)
        for row, a in enumerate(AXES, start=1):
            ax = self._cfg.axes[a]
            grid.addWidget(QtWidgets.QLabel(a), row, 0)

            scale = self._double_spin(ax.scale, -10.0, 10.0, 0.1)
            scale.valueChanged.connect(
                lambda v, name=a: (setattr(self._cfg.axes[name], "scale", v), self._changed()))
            grid.addWidget(scale, row, 1)

            inv = QtWidgets.QCheckBox()
            inv.setChecked(ax.invert)
            inv.toggled.connect(
                lambda v, name=a: (setattr(self._cfg.axes[name], "invert", v), self._changed()))
            grid.addWidget(inv, row, 2)

            en = QtWidgets.QCheckBox()
            en.setChecked(ax.enabled)
            en.toggled.connect(
                lambda v, name=a: (setattr(self._cfg.axes[name], "enabled", v), self._changed()))
            grid.addWidget(en, row, 3)
        form.addRow(grid_box)
        return page

    # --- Buttons tab ---
    def _buttons_tab(self):
        page = QtWidgets.QWidget()
        v = QtWidgets.QVBoxLayout(page)

        self._scope = QtWidgets.QComboBox()
        self._scope.addItem("Global", "")
        for name in sorted(Gui.listWorkbenches().keys()):
            self._scope.addItem(name, name)
        self._scope.currentIndexChanged.connect(self._reload_button_table)
        v.addWidget(self._scope)

        self._btable = QtWidgets.QTableWidget(N_BUTTONS, 2, page)
        self._btable.setHorizontalHeaderLabels(["Button", "Command"])
        self._btable.horizontalHeader().setStretchLastSection(True)
        v.addWidget(self._btable)
        self._reload_button_table()
        return page

    def _reload_button_table(self):
        scope = self._scope.currentData()  # "" => global
        for i in range(N_BUTTONS):
            self._btable.setItem(i, 0, QtWidgets.QTableWidgetItem(f"#{i}"))
            combo = QtWidgets.QComboBox()
            if scope:
                combo.addItem("(inherit global)", "")
            else:
                combo.addItem("(none)", "")
            for name in self._command_names:
                combo.addItem(name, name)
            current = self._binding(scope, i)
            idx = combo.findData(current or "")
            combo.setCurrentIndex(idx if idx >= 0 else 0)
            combo.currentIndexChanged.connect(
                lambda _idx, bnum=i, c=combo, s=scope: self._set_binding(s, bnum, c.currentData()))
            self._btable.setCellWidget(i, 1, combo)

    def _binding(self, scope, bnum):
        if scope:
            return self._cfg.wb_overrides.get(scope, {}).get(bnum, "")
        return self._cfg.global_bindings.get(bnum, "")

    def _set_binding(self, scope, bnum, command):
        command = command or ""
        if scope:
            m = self._cfg.wb_overrides.setdefault(scope, {})
            if command:
                m[bnum] = command
            else:
                m.pop(bnum, None)
        else:
            if command:
                self._cfg.global_bindings[bnum] = command
            else:
                self._cfg.global_bindings.pop(bnum, None)
        self._changed()

    # --- LCD tab ---
    def _lcd_tab(self):
        page = QtWidgets.QWidget()
        form = QtWidgets.QFormLayout(page)

        en = QtWidgets.QCheckBox()
        en.setChecked(self._cfg.lcd_enabled)
        en.toggled.connect(lambda v: (setattr(self._cfg, "lcd_enabled", v), self._changed()))
        form.addRow("Enable LCD output", en)

        sock = QtWidgets.QLineEdit(self._cfg.daemon_socket_path)
        sock.textChanged.connect(
            lambda v: (setattr(self._cfg, "daemon_socket_path", v), self._changed()))
        form.addRow("Daemon socket (advanced)", sock)
        return page

    # --- export / import ---
    def _export(self):
        path, _ = QtWidgets.QFileDialog.getSaveFileName(self, "Export settings", "space-elevator.json", "JSON (*.json)")
        if path:
            store.export_to_file(self._cfg, path)

    def _import(self):
        path, _ = QtWidgets.QFileDialog.getOpenFileName(self, "Import settings", "", "JSON (*.json)")
        if not path:
            return
        try:
            new_cfg = store.import_from_file(path)
        except (ValueError, OSError) as exc:
            QtWidgets.QMessageBox.warning(self, "Import failed", str(exc))
            return
        self._cfg = new_cfg
        self._on_changed(self._cfg)
        QtWidgets.QMessageBox.information(self, "Imported", "Settings imported. Reopen the dialog to see all values.")


def show():
    from . import lifecycle
    cfg = lifecycle.get_config()
    dlg = SettingsDialog(cfg, lifecycle.on_config_changed, Gui.getMainWindow())
    dlg.setModal(False)
    dlg.show()
```

- [ ] **Step 2: Commit**

```bash
git add freecad-addon/space_elevator/dialog.py
git commit -m "feat(addon): unified live settings dialog"
```

---

## Task 9: InitGui menu + remove old settings surfaces

**Files:**
- Modify: `freecad-addon/InitGui.py`
- Delete: `preferences.py`, `Resources/ui/preferences.ui`, `button_dialog.py`, `buttons.py`, `settings.py`, `tests/test_buttons.py`, `tests/test_settings.py`

- [ ] **Step 1: Update InitGui.py**

In `InitGui.py`, change `Initialize` to drop the preferences page and offer a Settings command, and replace the command class:

Replace:

```python
    def Initialize(self):
        FreeCAD.Console.PrintMessage("Space Elevator: workbench initialized\n")
        from space_elevator import preferences
        preferences.register()
        self.appendMenu("Space Elevator", ["SpaceElevator_ButtonBindings"])
```

with:

```python
    def Initialize(self):
        FreeCAD.Console.PrintMessage("Space Elevator: workbench initialized\n")
        self.appendMenu("Space Elevator", ["SpaceElevator_Settings"])
```

Replace the `_ButtonBindingsCmd` class and its `addCommand` line:

```python
class _ButtonBindingsCmd:
    def GetResources(self):
        return {"MenuText": "Button bindings…", "ToolTip": "Bind SpaceMouse buttons to FreeCAD commands"}

    def Activated(self):
        from space_elevator.button_dialog import show
        show()

    def IsActive(self):
        return True


Gui.addCommand("SpaceElevator_ButtonBindings", _ButtonBindingsCmd())
```

with:

```python
class _SettingsCmd:
    def GetResources(self):
        return {"MenuText": "Settings…", "ToolTip": "Space Elevator settings (navigation, buttons, LCD)"}

    def Activated(self):
        from space_elevator.dialog import show
        show()

    def IsActive(self):
        return True


Gui.addCommand("SpaceElevator_Settings", _SettingsCmd())
```

- [ ] **Step 2: Delete the superseded files**

```bash
git rm freecad-addon/space_elevator/preferences.py \
       freecad-addon/Resources/ui/preferences.ui \
       freecad-addon/space_elevator/button_dialog.py \
       freecad-addon/space_elevator/buttons.py \
       freecad-addon/space_elevator/settings.py \
       freecad-addon/tests/test_buttons.py \
       freecad-addon/tests/test_settings.py
```

- [ ] **Step 3: Verify full test suite passes**

Run: `devenv shell -- bash -c 'cd freecad-addon && python -m pytest -q'`
Expected: PASS — config, bindings, axes, store, libspnav tests; the deleted tests are gone.

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/InitGui.py
git commit -m "feat(addon): single Settings menu; remove old settings surfaces"
```

---

## Task 10: Install to FreeCAD and verify on hardware

**Files:** none (deployment + manual verification)

- [ ] **Step 1: Re-sync the working copy into FreeCAD's Mod dir**

```bash
DEST=/home/toph/.local/share/FreeCAD/v1-1/Mod/SpaceElevator
rm -rf "$DEST" && mkdir -p "$DEST"
cd /home/toph/Projects/space-elevator
cp -rL freecad-addon/InitGui.py freecad-addon/Init.py freecad-addon/pyproject.toml \
       freecad-addon/Resources freecad-addon/space_elevator "$DEST"/
find "$DEST" -name __pycache__ -type d -exec rm -rf {} + 2>/dev/null
```

(Note: `package.xml` stays out of the Mod copy for now — FreeCAD's modern loader rejects it; the classic loader runs InitGui.py directly. Re-enabling package.xml is tracked separately.)

- [ ] **Step 2: Start prerequisites**

```bash
sudo systemctl start spacenavd
./target/debug/space-elevatord &   # daemon as your user; needs the temp udev rule for LCD
```

- [ ] **Step 3: Launch FreeCAD and verify**

```fish
set -x SPACE_ELEVATOR_LIBSPNAV (nix build --no-link --print-out-paths nixpkgs#libspnav)/lib/libspnav.so.0
freecad
```

Verify manually:
- Switch to the "Space Elevator" workbench (no `spnav_open failed` on re-activation — Task 6).
- Menu "Space Elevator" → "Settings…" opens the dialog.
- Navigation tab: change Sensitivity / Pan speed while moving the puck → camera responds immediately (live).
- Toggle an axis "active" off → that DOF stops moving the view.
- Buttons tab: bind button #0 to `Std_ViewFit` (Global) → press it → view fits. The LCD shows the "ViewFit" label.
- Switch the scope dropdown to a workbench, override a button → switch to that workbench in FreeCAD → LCD labels change accordingly.
- LCD tab: untick "Enable LCD output" → labels stop updating.
- Export… to a file, change a value, Import… it back → value restored.

- [ ] **Step 4: Commit any fixups discovered during verification**

```bash
git add -A && git commit -m "fix(addon): settings dialog hardware-test fixups"
```

---

## Notes for the implementer

- `EventPump` API (`start()`/`stop()`, constructor takes the handler) is unchanged from today; Task 7 relies on it.
- The workbench-activated signal name (`Gui.getMainWindow().workbenchActivated`) is best-effort; if it does not connect, labels still refresh on workbench (de)activation through the addon's own activate path. Confirm the exact signal during Task 10 and adjust if needed.
- Button count `N_BUTTONS = 31` matches the SpaceMouse Enterprise (carried over from the old dialog).
