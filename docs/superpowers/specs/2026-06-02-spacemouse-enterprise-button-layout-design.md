# SpaceMouse Enterprise Button Layout — Design

**Date:** 2026-06-02
**Status:** Approved
**Scope:** `freecad-addon` only (Python side)

## Goal

Provide a canonical, reusable mapping of SpaceMouse Enterprise physical buttons
to event indices, human-readable names, and default FreeCAD command bindings.
Each button supports two bindings — a **short press** and a **long press** —
because the Enterprise's alternate functions (View Top/Bottom, V1 Restore/Save,
ISO1/ISO2, …) are accessed via press duration, not modifier keys.

Defaults apply as a **runtime fallback**: a user binding stored in FreeCAD
parameters always wins; when absent, the layout default is used. Nothing is
written to the parameter store on behalf of defaults, so updating the layout
table reaches existing users without clobbering their settings.

## Non-Goals

- No Rust / `spnav` / daemon changes. The mapping lives in the Python addon.
- No "saved camera view" feature. V1–V3 Restore/Save default to `null`
  (no FreeCAD standard command exists); the user may bind them manually.
- No modifier-key handling. The Enterprise alternates are short/long press.
- Long-press bindings are layout defaults + user-editable; no per-button
  enable/disable toggles beyond leaving a binding empty.

## Button Index Map

Indices are the raw `event.button` values reported by libspnav for the
SpaceMouse Enterprise (USB `256f:c633`). 31 buttons, indices 0–30
(matches the existing `N_BUTTONS = 31`).

| Idx | Short (name → default) | Long (name → default) |
|----|----|----|
| 0–11 | "1"–"12" → `null` | — |
| 12 | Menu → `null` | — |
| 13 | Fit → `Std_ViewFit` | — |
| 14 | View Top → `Std_ViewTop` | View Bottom → `Std_ViewBottom` |
| 15 | View Right → `Std_ViewRight` | View Left → `Std_ViewLeft` |
| 16 | View Front → `Std_ViewFront` | View Back → `Std_ViewRear` |
| 17 | Rotate Right → `Std_ViewRotateRight` | Rotate Left → `Std_ViewRotateLeft` |
| 18 | Esc → `null` | — |
| 19 | Alt → `null` | — |
| 20 | Shift → `null` | — |
| 21 | Ctrl → `null` | — |
| 22 | Rot Lock → `null` | — |
| 23 | Enter → `null` | — |
| 24 | Delete → `Std_Delete` | — |
| 25 | Tab → `null` | — |
| 26 | Space → `null` | — |
| 27 | V1 Restore → `null` | V1 Save → `null` |
| 28 | V2 Restore → `null` | V2 Save → `null` |
| 29 | V3 Restore → `null` | V3 Save → `null` |
| 30 | ISO1 → `Std_ViewIsometric` | ISO2 → `Std_ViewAxonometric` |

## Components

### 1. `freecad-addon/space_elevator/enterprise_layout.json`

Single source of truth. Top-level `device`/`usb_id` metadata plus a `buttons`
array. One entry per named button (unnamed indices may be omitted).

```json
{
  "device": "SpaceMouse Enterprise",
  "usb_id": "256f:c633",
  "buttons": [
    { "index": 0,  "name": "1",            "default": null },
    { "index": 13, "name": "Fit",          "default": "Std_ViewFit" },
    { "index": 14, "name": "View Top",     "default": "Std_ViewTop",
      "long_name": "View Bottom",          "long_default": "Std_ViewBottom" },
    { "index": 27, "name": "V1 Restore",   "default": null,
      "long_name": "V1 Save",              "long_default": null }
  ]
}
```

Fields: `index` (int), `name` (str), `default` (str|null). Optional
`long_name` (str), `long_default` (str|null) for buttons with a long-press
function. `null` defaults are kept explicit for documentation value.

### 2. `freecad-addon/space_elevator/layout.py` — loader

Loads the JSON once on import (stdlib `json`, zero dependencies). Pure lookup,
no FreeCAD state.

API:
- `name(index) -> str | None` — short-press physical name.
- `long_name(index) -> str | None` — long-press name, or `None`.
- `default_command(index) -> str | None` — short-press default command.
- `long_default_command(index) -> str | None` — long-press default command.
- `count -> int` — number of mappable buttons (31).

A missing or malformed JSON file yields an empty layout (all lookups return
`None`); the addon must not crash.

### 3. `Bindings` (extend `buttons.py`)

Add a parallel long-press track. Stored as ParamGet strings under the existing
group.

- Short binding key: `button_<n>` (unchanged).
- Long binding key: `button_<n>_long`.

New/changed methods:
- `get(bnum) -> str | None` — short user binding (unchanged).
- `get_long(bnum) -> str | None` — long user binding.
- `set(bnum, command) ` — short (unchanged).
- `set_long(bnum, command)` — long.

`Bindings` stays a pure parameter store. It does **not** know about the layout
or fallback logic — that lives in the caller.

### 4. Event handling (`lifecycle.py`)

Press duration decides short vs long.

- **Fire on release** (changed from fire-on-press). On a button-press event,
  record the press timestamp keyed by button index. On the matching release,
  compute `held_ms = release_ts − press_ts` and dispatch.
- Threshold: `LONG_PRESS_MS = 400`. `held_ms >= LONG_PRESS_MS` → long, else
  short.
- Resolution order (short example; long is analogous):
  1. `Bindings.get(bnum)` — user short binding.
  2. else `layout.default_command(bnum)`.
  - For long: `Bindings.get_long(bnum)` → `layout.long_default_command(bnum)`
    → **fall back to the resolved short command** (long-pressing a button with
    no long binding still does its short action).
- If the resolved command is `None`, do nothing.

Timestamps use `time.monotonic()`: the button event tuple is
`("button", bnum, pressed)` and carries no timestamp, so the handler stamps
press and release itself. `held_ms = (release_mono − press_mono) * 1000`.

### 5. Dialog (`button_dialog.py`)

- Column 0: `layout.name(i)` instead of `#i` (fall back to `#i` if unnamed).
- Two command combos per row: **Short** and **Long**. Empty `(none)` means
  "use layout default" (preserves runtime fallback — saving does not write the
  default into params).
- `long_name(i)` shown as a hint/tooltip on the Long combo where present.
- Save writes both `set` and `set_long` for each row.

### 6. LCD labels (`lifecycle._push_labels`)

For the first 8 buttons, label fallback becomes:
`short-name-of(user binding) or layout.name(i)` — buttons with no user binding
show the physical name instead of blank.

## Boundaries Check

- `layout.py`: knows only the JSON. No FreeCAD, no params.
- `Bindings`: knows only params. No layout, no fallback.
- `lifecycle`: composes the two (fallback + short/long dispatch). The only unit
  that knows both.
- `button_dialog`: presentation; reads layout for labels/defaults, writes
  user bindings.

## Testing

- `tests/test_layout.py`:
  - Loads the real JSON; spot-check `name`/`default_command`/`long_*` for a
    known index (e.g. 14 → "View Top"/`Std_ViewTop`, long "View Bottom"/
    `Std_ViewBottom`).
  - Unnamed index → `None`.
  - Malformed/missing file → empty layout, no exception.
- `tests/test_buttons.py`: extend for the long track (`set_long`/`get_long`,
  independence from short, clear). Existing short tests unchanged.
- Short/long dispatch logic in `lifecycle` is covered by extracting the
  resolution into a small testable function (`resolve(bindings, layout, bnum,
  held_ms)`) so it can be unit-tested without FreeCAD.

## Open / Deferred

- Per-button "saved camera view" backing V1–V3 (no FreeCAD command today).
- Making `LONG_PRESS_MS` a user setting (constant for now).
- Modifier-key alternates (explicitly out — the device uses press duration).
</content>
</invoke>
