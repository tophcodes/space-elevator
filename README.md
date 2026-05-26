# Space Elevator

Space Elevator is a FreeCAD addon for 6-degree-of-freedom (6DOF) input devices — primarily 3Dconnexion SpaceMice — backed by an optional Rust daemon that drives the LCD on the SpaceMouse Enterprise.

## Components

- `spnav-sys/`, `spnav/` — Rust bindings for libspnav (used by the daemon; published as standalone crates).
- `space-elevatord/` — Rust daemon. Owns the SpaceMouse Enterprise LCD. Exposes a Unix-socket JSON IPC.
- `freecad-addon/` — Python FreeCAD workbench. Reads motion and button events from spacenavd via libspnav directly. Drives view navigation. Optionally pushes SVG-rendered images to the daemon for LCD output.

## Supported hardware

- 3Dconnexion SpaceMouse Enterprise (motion + buttons + LCD)

Other SpaceMice work for motion and buttons; the LCD path is Enterprise-only.

## Supported applications

- FreeCAD 1.0+

The previous Onshape browser-extension target has been dropped from active scope.
