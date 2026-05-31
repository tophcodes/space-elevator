# Space Elevator — FreeCAD-first pivot

**Date**: 2026-05-26
**Status**: Design

## Summary

Pivot Space Elevator from "Tauri+Vue GUI config tool with Onshape browser extension" to "FreeCAD-native addon backed by a small Rust daemon". Drop the Tauri shell and the Onshape extension as first-class targets. Keep the existing Rust crates (`spnav-sys`, `spnav`) — they carry the LCD work that nothing else has.

## Motivation

- Onshape extension is browser-side and cannot reach the LCD on the SpaceMouse Enterprise. The LCD is the single most distinctive feature this project can deliver.
- FreeCAD already integrates with `spacenavd` upstream for motion, but offers no LCD output, no per-app button mapping, no axis tuning UI, no navigation presets matching Onshape/Fusion conventions. These are the value-adds.
- The user runs NixOS with a SpaceMouse Enterprise and uses FreeCAD as primary CAD. First target should be the one the user actually uses.

## Architecture

Two components, both delivered from this repo:

### 1. `space-elevator` (FreeCAD addon — Python)

A FreeCAD workbench installed through the FreeCAD Addon Manager.

- **Motion**: ctypes wrapper around `libspnav`. Uses `spnav_open()` and `spnav_fd()` (file descriptor). Wires that fd into a Qt `QSocketNotifier` so events flow through FreeCAD's main loop without polling. Replaces FreeCAD's built-in handling for the SpaceMouse while the addon is active.
- **View navigation**: translates motion events into camera transforms on the active 3D view. Ships with a small set of named presets (FreeCAD-default, Onshape-style, Fusion-style) selectable in the addon's preferences. Per-axis deadzone and scale also configurable, with live preview.
- **Button mapping**: per-workbench button → FreeCAD command bindings. Stored in FreeCAD's user parameter system (`App.ParamGet`).
- **LCD bridge**: optional. If the `space-elevatord` Unix socket exists, the addon connects and pushes button labels matching the current workbench's bindings, plus optional status text. If the daemon is absent, LCD features hide; motion still works.
- **Preferences UI**: a Preferences page inside FreeCAD (no separate window) for presets, axis tuning, button bindings, LCD on/off.
- **Distribution**: pure Python. Manifest `package.xml` for Addon Manager. No compiled deps in the addon itself — `libspnav.so.0` already present on every distro that ships spacenavd.

### 2. `space-elevatord` (Rust daemon)

A small long-running process that owns the LCD on the SpaceMouse Enterprise.

- Reuses the existing `spnav` crate's `lcd` module verbatim — that code is already debugged against the real hardware.
- Exposes a Unix socket at `$XDG_RUNTIME_DIR/space-elevator.sock`.
- Wire protocol: newline-delimited JSON. Small command surface — `set_buttons(labels: [String])`, `set_status(text: String)`, `clear()`, `ping()`. Versioned with a `version` field on every message.
- Single-client at a time. New connection kicks the previous one (the FreeCAD addon is the only consumer).
- No motion handling. No config handling. Just the LCD. Keeping it narrow means we don't have to design a sprawling IPC and the daemon stays optional.
- Distribution: cargo / nix flake / AUR. Out-of-band from the addon. The addon detects the socket and offers a one-liner install hint if missing.

### What goes away

- `space-elevator/src-tauri/` — already deleted in the working tree. Confirm and commit the deletion.
- `space-elevator/src/` (Vue frontend) — delete. Config UI moves into FreeCAD itself; the standalone GUI tool is no longer the product.
- `space-elevator/` package.json, vite, bun.lock, node_modules — delete with the Vue frontend.
- README references to the Onshape browser extension — rewrite around FreeCAD.

The `spnav-sys` and `spnav` crates stay. They keep their crate boundaries (publishable independently). The new `space-elevatord` binary is added as a third Cargo workspace member.

## Repo layout after the pivot

```
space-elevator/
  Cargo.toml                  # workspace root
  spnav-sys/                  # unchanged
  spnav/                      # unchanged (lcd module reused by daemon)
  space-elevatord/            # new — Rust binary crate
    Cargo.toml
    src/main.rs               # socket server + IPC dispatch
  freecad-addon/              # new — Python addon
    package.xml               # FreeCAD Addon Manager manifest
    InitGui.py
    space_elevator/
      __init__.py
      libspnav.py             # ctypes bindings + QSocketNotifier glue
      navigation.py           # motion → camera transforms, presets
      buttons.py              # button binding storage and dispatch
      lcd_client.py           # Unix socket client for space-elevatord
      preferences.py          # FreeCAD preferences page
    Resources/
      icons/, ui/
  docs/
    superpowers/specs/
  README.md                   # rewritten
```

## Data flow

```
SpaceMouse Enterprise (HID)
    │
    ├── spacenavd ──► libspnav ──► [ctypes] ──► QSocketNotifier ──► addon
    │                                                                  │
    │                                                                  ├── view transforms
    │                                                                  ├── button dispatch
    │                                                                  └── lcd_client ──► Unix socket ──► space-elevatord ──► HID (LCD interface)
    │
    └── (LCD HID interface owned exclusively by space-elevatord)
```

Motion path and LCD path are independent. Failure in one does not affect the other.

## Error handling

- **spacenavd not running**: addon shows a one-time notification with the systemd unit name; motion features disabled until reconnected. Reconnect attempted on workbench activation.
- **`space-elevatord` not running**: LCD features silently hidden. No nagging. Preferences page shows a "daemon not detected" line with install hint.
- **libspnav not installed**: addon prints a clear error in the FreeCAD report view and disables itself. No crash.
- **Daemon IPC version mismatch**: refuse to connect, log expected vs. actual version. User updates whichever side is behind.

## Testing

- `spnav` and `spnav-sys` keep whatever tests they have.
- `space-elevatord`: unit tests for IPC protocol parsing. Manual hardware test for LCD output (already how the existing code is verified).
- FreeCAD addon: unit tests for pure-Python pieces (axis math, preset config, button-binding storage). Manual integration test inside a running FreeCAD instance — Python addons are hard to test headlessly and we will not pretend otherwise.

## Out of scope

- macOS and Windows support. spacenavd is Linux-first; revisit later.
- The Onshape browser extension. Not deleted from anyone's mind, just not part of this pivot. If revived, it lives in a separate repo.
- A general "input device abstraction layer". One device, one CAD app, ship it.
- Driving the LCD without spacenavd running (would require the daemon to also speak the motion HID interface and arbitrate with spacenavd). Today the daemon only touches the LCD HID interface; spacenavd owns motion.

## Open question deferred to implementation

- Exact IPC framing format (JSON lines vs. length-prefixed). Defaulting to JSON lines for ease of debugging; revisit if it becomes a problem.
