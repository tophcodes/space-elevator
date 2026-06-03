# Space Elevator — LCD service pivot — design

Date: 2026-06-02
Status: design (supersedes `2026-06-02-freecad-settings-design.md`, which is now obsolete)

## The pivot

The original plan made the addon an own-spnav-client driver (tuning + bindings + navigation + LCD). That is dead: FreeCAD 1.1.1 holds the process-global libspnav connection with no off-switch, and its spaceball Qt events aren't readable from Python. So an in-process addon can neither own spnav nor read FreeCAD's events.

Division of labour going forward:
- **Navigation tuning** (speed, dead-zone, invert, axis-lock, dominant-axis): already done by **spacenavd** (`~/.spnavrc` / `spnavcfg`), globally, for FreeCAD too. Space Elevator does NOT touch it.
- **Per-workbench button→command bindings**: contributed **upstream into FreeCAD** (C++). FreeCAD's native Spaceball button mapping is global-only today; per-workbench is the upstream feature. Out of scope for this spec.
- **LCD output**: Space Elevator's unique value. This spec covers it.

## Goal

`space-elevatord` becomes a generic **LCD service**: clients push a structured "state" (profile, mode, 12 button tiles with label/icon/category) over the existing unix socket; the daemon renders the SVG from a fixed 12-tile template and drives the SpaceMouse Enterprise LCD. A thin FreeCAD plugin (no workbench) is the first client.

## Architecture

```
FreeCAD plugin (thin, no workbench)            space-elevatord (LCD service)
  on activate / workbench change / binding   ── lcd_set_state ──▶  render (ported template, Signal theme)
  change: gather {profile, mode, 12 tiles}                          → resvg rasterize (svg_render)
  each tile {label, icon(svg), cat, active}                         → 512-hdr / BGR565 / fixed-Huffman → USB
```

No spnav, no libspnav, no tuning in the plugin. The daemon owns rendering policy (consistent look); clients send semantic data only.

## IPC protocol

New structured command on the existing JSON-line socket:

```json
{ "v": 1, "id": 1, "cmd": "lcd_set_state",
  "theme": "signal",
  "profile": "FreeCAD",
  "mode": "Sketcher",
  "left":  [ {"label":"Line","icon":"<svg…>","cat":"draw","active":false}, … 6 ],
  "right": [ {"label":"Coincident","icon":"<svg…>","cat":"constrain"}, … 6 ] }
```

- `theme`: `console|signal|paper` (default `signal`).
- `profile`, `mode`: centre-gutter text (app + active mode/workbench).
- `left`/`right`: up to 6 tiles each (3×2 clusters = 12 function keys). Fewer than 6 → remaining tiles blank.
- Tile fields:
  - `label` (string).
  - `icon` (optional): icon **content supplied by the client** — an SVG string (inlined) or a `data:` URI / PNG base64 (transcoded to `<image>`). If absent, fall back to a built-in named glyph (`"icon_name"`) from the ported library, else a generic glyph. The daemon normalises/scales the icon into the tile's 24×24 icon box.
  - `cat` (optional): `draw|modify|constrain|view|cut` → tile/accent colour. Default neutral.
  - `active` (optional bool): highlighted "mode active" tile.

Existing low-level `lcd_display_svg` / `lcd_display_image` / `lcd_clear` remain as escape hatches.

## Daemon: render engine (port of lcd-template.js → Rust)

Port `renderLCD(config, theme)` from the Claude Design bundle (`lcd-template.js`) to Rust, producing the same 640×150 SVG:
- Layout: two 3×2 clusters (left `x∈[6,300)`, right `x∈[340,634)`), 40px centre gutter with a hairline + rotated `profile` (uppercase, small) and `mode` (larger).
- Themes `console|signal|paper` (palettes/tile styles per the JS): bg, category palette (`CAT_DARK`/`CAT_LIGHT`), tile fill/stroke/radius, label colour/weight/size, icon stroke/fill rules, active-state fill/fg.
- Category colours: `draw #4DA3FF, modify #FFB454, constrain #46D08A, view #B98CFF, cut #FF6B6B` (dark); light variants for `paper`.
- Per tile: optional tile bg / active highlight, the icon (client-supplied or fallback glyph) centred at ~36% height scaled into a 27px box, label centred at ~80% height (`Source Sans 3`, `DejaVu Sans` fallback — must be installed for headless render).
- Built-in fallback glyph library (~22 sketch glyphs) ported as-is for the no-client-icon case and the demo.

Output SVG → existing `svg_render::render_to_rgb888` (resvg) → existing LCD pipeline. Default theme **signal**.

Icon embedding detail: SVG icon strings are inlined inside a `<g transform>` slot (resvg renders nested SVG); raster icons become `<image href="data:…">`. Client-supplied colourful icons override theme tinting; `cat` still drives the tile accent/active fill.

## FreeCAD plugin (thin client)

A minimal addon: an `InitGui.py` that registers a background hook (no workbench, no menu beyond maybe an enable toggle). It:
- builds state on: startup, workbench-activated signal, and FreeCAD spaceball-binding change;
- `profile = "FreeCAD"`, `mode = active workbench name`;
- reads FreeCAD's **global** spaceball button→command bindings (param storage TBD — to be dumped from `BaseApp/Preferences/Spaceball`), maps the 12 function-key buttons to tiles;
- per tile: `label` = command menu text; `icon` = that command's SVG icon content (FreeCAD ships SVG command icons — resolve via the command's `Pixmap` resource); `cat` = derived (default neutral; optional curated map later);
- sends `lcd_set_state` via the existing `LcdClient` (extended with a `set_state` method).
- LCD failures stay silent/optional (daemon may be down).

Until upstream per-workbench bindings land, tiles reflect FreeCAD's global bindings; per-mode palettes come later.

## What is reused vs removed

Reused: daemon LCD pipeline (`spnav::lcd`: 512-header/BGR565/fixed-Huffman/USB), `svg_render` (resvg), the daemon socket/IPC scaffold, `lcd_client.py`, `enterprise_layout.json` (button names/indices).

Removed from the addon (obsolete driver layer): `event_pump.py`, `navigator.py`, `libspnav.py`, the settings `dialog.py`, `config.py`, `store.py`, `axes.py`, `bindings.py` (label helper may survive), and the `SPACE_ELEVATOR_LIBSPNAV` env requirement. The Rust `spnav` event bindings stay in the crate (still valid library code) but the addon no longer uses them.

## Future clients — web (Onshape)

The `lcd_set_state` schema is **transport-agnostic**: the same JSON messages can arrive over any transport, fed into one `handle_set_state(msg)` handler. To support Onshape (browser-based CAD), the daemon gains a **second listener** alongside the unix-domain socket — a loopback TCP/WebSocket listener on `127.0.0.1:<port>` — because browser extensions cannot open a unix socket. Both listeners accept connections and feed the same handler ("two transports, one brain"). v1 (FreeCAD) uses only the unix socket; the WS listener is added when web clients are built.

To keep this additive, v1 must **decouple message handling from the transport** (the IPC accept-loop hands raw messages to a transport-independent `handle_set_state`), so a second listener is purely additive later.

Onshape specifics / limitations:
- Client = a browser extension / userscript. It can read Onshape context (active document, sketch/part/assembly → `mode`) but most likely **cannot** read Onshape's 3D-mouse button bindings (foreign, vendor-controlled web app). So Onshape tiles are **curated palettes per mode**, not a mirror of button bindings.
- Navigation/input for Onshape is already handled by 3Dconnexion's web integration / spacenavd. Space Elevator's contribution for Onshape is the **LCD only** (as with FreeCAD). Buttons could optionally act via spacenavd `kbmapN` (button→keystroke→Onshape hotkey).
- Security: bind the WS listener to loopback only and add a minimal origin/token check — otherwise any local web page could post LCD state.

## Open items

- Locate FreeCAD's spaceball button-binding param storage (dump `BaseApp/Preferences/Spaceball`) — needed to read bindings for the tiles.
- Confirm how to resolve a FreeCAD command's SVG icon content from Python.
- `cat` derivation (neutral default v1; curated command→cat map later).
- Per-workbench palettes depend on the upstream FreeCAD contribution.

## Testing

- Daemon render port: golden-SVG / structural tests (tiles present, gutter text, theme colours) — pure Rust, no device.
- `lcd_set_state` round-trip: send state → daemon renders without error (headless, no device), like the existing ping/svg tests.
- Icon embedding: SVG-string inlines; PNG data-uri transcodes.
- Plugin: manual (FreeCAD GUI + hardware).
```

