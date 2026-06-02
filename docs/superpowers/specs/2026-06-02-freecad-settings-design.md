# FreeCAD addon settings — design

Date: 2026-06-02
Status: approved (design), pending implementation plan

## Goal

Give the Space Elevator FreeCAD addon a coherent settings UX before release.
Today settings are split (a FreeCAD preferences page for axis tuning + a
separate button-bindings dialog), some controls exist only in the backend
(per-axis invert) and others are hardcoded (navigation speed). Replace this
with a single, live-applying settings dialog whose entire state is
JSON-serializable for export/import.

## Decisions

- **One custom workbench dialog**, not FreeCAD's Preferences page. Full layout
  control and live preview. Opened from the `Space Elevator` menu → `Settings…`.
- **Live, immediate apply.** Moving a control changes navigation/LCD right away
  while the puck is moving; values persist on close. No Apply/Revert button.
- **Shared `Config` object** as the single source of truth (approach A). Held in
  memory; the event hot path reads it directly. ParamGet is touched only on
  load/save, not per event. No observer/signal machinery.
- **Bindings: global + per-workbench overrides.** Effective binding =
  override-if-set else global. LCD labels reflect the effective set, so they
  follow the active workbench.
- **JSON-serializable config** with a `version` field, for file export/import.

## Data model

`Config` is a plain, FreeCAD-free object, fully dict/JSON-serializable.
Canonical form:

```json
{
  "version": 1,
  "navigation": {
    "sensitivity": 1.0, "pan_speed": 60.0, "rot_speed": 1.2,
    "deadzone": 0.02, "dominant_axis": false,
    "axes": {
      "tx": {"scale": 1.0, "invert": false, "enabled": true},
      "ty": {"scale": 1.0, "invert": false, "enabled": true},
      "tz": {"scale": 1.0, "invert": false, "enabled": true},
      "rx": {"scale": 1.0, "invert": false, "enabled": true},
      "ry": {"scale": 1.0, "invert": false, "enabled": true},
      "rz": {"scale": 1.0, "invert": false, "enabled": true}
    }
  },
  "lcd": { "enabled": true, "daemon_socket_path": "" },
  "bindings": {
    "global": {"0": "Std_ViewFit"},
    "overrides": {"SketcherWorkbench": {"2": "Sketcher_NewSketch"}}
  }
}
```

- `Config.default()`, `Config.to_dict()`, `Config.from_dict()` — pure.
- JSON keys: button numbers as strings (`"0"`), axis names as strings (`"tx"`).
- **Effective bindings** (pure): `effective(wb) = { b: overrides.get(wb,{}).get(b) or global.get(b) }`.
- `pan_speed` (mm/s at full deflection) and `rot_speed` (rad/s at full
  deflection) replace the hardcoded navigator constants.
- `sensitivity` is an overall multiplier; per-axis `scale` is fine-tuning;
  `enabled=false` locks (mutes) that DOF; `dominant_axis=true` keeps only the
  strongest axis per event.

Dropped from earlier discussion: LCD idle/status screen, brightness (brightness
needs device-protocol R&D; out of scope here).

## Persistence & export/import

- The whole config is stored as **one JSON string** in ParamGet under the
  addon's parameter group (key `config_json`). No per-field param keys.
- **Export** writes that JSON to a user-chosen `.json` file.
- **Import** reads a `.json` file → `from_dict` (with `version` check) →
  applies live + persists. Invalid JSON or unsupported version → error dialog,
  no change.

## Modules

Core (FreeCAD-free, unit-tested):

| module | role |
|--------|------|
| `config.py` *(new)* | `Config` + `default`/`to_dict`/`from_dict` |
| `axes.py` *(extend)* | `transform()` honors sensitivity, per-axis scale/invert, `enabled` lock, deadzone, `dominant_axis` |
| `bindings.py` *(from buttons.py)* | pure `effective(global, overrides, wb)` resolution + label extraction |
| `lcd_labels.py` *(exists)* | labels → SVG |

FreeCAD adapters (thin, manual test):

| module | role |
|--------|------|
| `store.py` *(new)* | `Config` ↔ ParamGet (`config_json`) + JSON file export/import |
| `navigator.py` *(extend)* | reads `pan_speed`/`rot_speed` from `Config` |
| `dialog.py` *(new, replaces button_dialog.py)* | unified Qt dialog: Navigation / Buttons / LCD tabs + Export/Import |
| `lifecycle.py` *(extend)* | owns live `Config`, wires workbench-change, handler reads `Config`, idempotent libspnav open/close |
| `lcd_client.py`, `event_pump.py`, `libspnav.py` | unchanged |

Removed: `preferences.py` + `Resources/ui/preferences.ui` (FreeCAD pref page),
`button_dialog.py` (folded into the Buttons tab). `InitGui.py` menu offers a
single `Settings…` entry.

## Dialog layout

Modeless `QDialog` with a `QTabWidget`.

- **Navigation**: sensitivity, pan-speed, rot-speed, deadzone, dominant-axis
  checkbox; then a 6-row per-axis grid (tx ty tz rx ry rz) × columns
  scale (spinbox) · invert (check) · active (check, = lock).
- **Buttons**: scope dropdown (`Global` + each workbench); a table of
  button-number → command picker (searchable combo of valid Gui commands). In a
  workbench scope, unset rows show the inherited global value greyed out with a
  "reset to global" control.
- **LCD**: enable checkbox; advanced: daemon socket path.
- **Always**: `Export…` / `Import…` (JSON file) and `Close`.

Every change mutates `Config`, calls `store.save`, and applies immediately.

## Live data flow

- **Motion event**: handler reads live `Config` → `axes.transform(raw, Config)`
  → `navigator.apply(transformed, period, pan_speed, rot_speed)`.
- **Button event**: handler looks up the pre-resolved effective bindings for the
  current workbench → `Gui.runCommand(cmd)`.
- **Dialog change**: mutate `Config` + `store.save`. Navigation takes effect on
  the next event automatically; binding/LCD changes re-resolve effective
  bindings and re-push LCD labels.
- **Workbench changed** (FreeCAD signal): re-resolve effective bindings + push
  new labels to the LCD. Labels follow the active workbench.
- **Activate**: `store.load` → start pump → resolve bindings → push labels.
  **Deactivate**: stop pump → close libspnav (idempotent, so re-activation does
  not spuriously fail `spnav_open`).

## Error handling

- Daemon unavailable / LCD write fails → skip silently or warn (as today).
- libspnav missing → existing error; `spnav_open`/`close` made idempotent to
  kill the spurious "spnav_open failed" on re-activation.
- Import of invalid JSON or unsupported `version` → error dialog, config
  unchanged.
- Unknown command → command picker only offers valid commands; `runCommand`
  stays wrapped in try/except.

## Testing

Pure core, via the existing pytest setup:

- `config`: `to_dict`/`from_dict` round-trip, `default`, version handling.
- `axes.transform`: sensitivity, invert, `enabled` lock, `dominant_axis`,
  deadzone — table-driven (extends `test_axes`).
- `bindings.effective`: override-vs-global resolution.
- `store`: load/save round-trip against a fake parameter group (like
  `test_settings`).

Dialog, lifecycle, navigator: manual hardware test (Qt/FreeCAD GUI).

## Out of scope

LCD brightness and idle/status screen; per-axis acceleration curves; importing
3Dconnexion-driver profiles.
```

