# Space Elevator

Space Elevator is a display driver for the 3Dconnexion SpaceMouse Enterprise on Linux. It puts the active CAD app's mode and button labels on the device's built-in screen. It does not handle motion or buttons; those go through spacenavd as usual. A CAD app pushes its current state (profile, mode, button tiles) to a small daemon, which renders that and sends it to the screen.

Primary targets are FreeCAD and Onshape.

## Components

- `spnav-sys/`, `spnav/`: Rust bindings for libspnav. They carry the LCD pipeline and are also published as standalone crates.
- `space-elevatord/`: Rust daemon. Owns the SpaceMouse Enterprise LCD. It takes a structured "state" over a Unix-socket JSON IPC, renders an SVG, and pushes it to the device. The message handler is transport-agnostic, so a loopback WebSocket listener can be added later for browser clients (Onshape).
- `freecad-addon/`: Thin FreeCAD client. On startup, workbench change, and binding change it gathers `{profile, mode, 12 button tiles}` and pushes it to the daemon. No input handling, no navigation.

## Status

- Daemon and LCD render pipeline: working.
- FreeCAD client: working.
- Onshape client: planned. Needs a WebSocket listener on the daemon and a browser extension or userscript, since a browser page can't open a Unix socket. For Onshape the tiles would be curated per mode rather than read from button bindings.

## Supported hardware

- 3Dconnexion SpaceMouse Enterprise (motion, buttons, and LCD).

Other SpaceMice work for motion and buttons through spacenavd; the LCD path is Enterprise-only.
