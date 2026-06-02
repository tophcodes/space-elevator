# LCD Service (daemon) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `space-elevatord` into a generic LCD service: accept a structured `lcd_set_state` IPC message (profile, mode, 12 tiles with label/icon/category/active, theme) and render it to the SpaceMouse Enterprise LCD by porting the `renderLCD` template to Rust.

**Architecture:** A new `lcd_template` module ports the bundled `docs/lcd-template/lcd-template.js` (the approved Claude Design template) to Rust — same 640×150 SVG: two 3×2 tile clusters + a centre profile/mode gutter, three themes (default `signal`), a fixed glyph library, and client-supplied icon embedding. The IPC gains an `lcd_set_state` variant dispatched transport-independently; the generated SVG flows through the existing `svg_render` (resvg) → `LcdHandle` (512-header/BGR565/fixed-Huffman/USB) pipeline.

**Tech Stack:** Rust, serde/serde_json (already deps), resvg (via existing `svg_render`), tokio. Tests are headless Rust unit tests (`cargo test -p space-elevatord`), no device. The FreeCAD client is a SEPARATE later plan.

**Conventions:** Build/test from the repo root inside the devenv shell:
`devenv shell -- bash -c 'cargo test -p space-elevatord'` (or the `nix shell` fallback used elsewhere). The canonical render reference is `docs/lcd-template/lcd-template.js` — port it faithfully; when a task says "port verbatim from the reference", translate that exact JS function to Rust producing the identical SVG substring.

---

## File Structure

- Create `space-elevatord/src/lcd_template.rs` — state types (`LcdState`, `Tile`, `Theme`), category palettes, theme table, glyph library, geometry helpers, and `render(&LcdState) -> String`.
- Modify `space-elevatord/src/lib.rs` — add `pub mod lcd_template;`.
- Modify `space-elevatord/src/ipc.rs` — add the `LcdSetState(LcdState)` request variant.
- Modify `space-elevatord/src/server.rs` — dispatch `LcdSetState`: render → rasterize → display.
- Modify `space-elevatord/examples/` (new) — `send_test_state.rs` smoke-test client.

Reused unchanged: `svg_render.rs` (`render_to_rgb888`), `lcd_handle.rs` (`display_rgb888`), the unix-socket server loop.

---

## Task 1: State types + IPC variant

**Files:**
- Create: `space-elevatord/src/lcd_template.rs`
- Modify: `space-elevatord/src/lib.rs`, `space-elevatord/src/ipc.rs`

- [ ] **Step 1: Write the failing test** — append to `space-elevatord/src/ipc.rs` inside its existing `mod tests`:

```rust
    #[test]
    fn parses_lcd_set_state_request() {
        let raw = r#"{"v":1,"id":9,"cmd":"lcd_set_state","theme":"signal","profile":"FreeCAD","mode":"Sketcher","left":[{"label":"Line","icon":"line","cat":"draw"}],"right":[]}"#;
        let req: Request = serde_json::from_str(raw).unwrap();
        assert_eq!(req.id, 9);
        match req.payload {
            RequestPayload::LcdSetState(s) => {
                assert_eq!(s.profile, "FreeCAD");
                assert_eq!(s.mode, "Sketcher");
                assert_eq!(s.left.len(), 1);
                assert_eq!(s.left[0].label, "Line");
                assert_eq!(s.left[0].cat.as_deref(), Some("draw"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn lcd_set_state_defaults_theme_signal() {
        let raw = r#"{"v":1,"id":1,"cmd":"lcd_set_state"}"#;
        let req: Request = serde_json::from_str(raw).unwrap();
        match req.payload {
            RequestPayload::LcdSetState(s) => assert_eq!(s.theme, crate::lcd_template::Theme::Signal),
            _ => panic!("wrong variant"),
        }
    }
```

- [ ] **Step 2: Run, expect FAIL** (no `LcdSetState`, no `lcd_template`):
`devenv shell -- bash -c 'cargo test -p space-elevatord parses_lcd_set_state 2>&1 | tail -20'`
Expected: compile error — unknown variant / module.

- [ ] **Step 3: Create `space-elevatord/src/lcd_template.rs` with the state types:**

```rust
//! Port of the Claude Design LCD template (docs/lcd-template/lcd-template.js).
//! Renders a 640x150 SVG for the SpaceMouse Enterprise: two 3x2 tile clusters
//! plus a centre profile/mode gutter. FreeCAD-agnostic; clients send state.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Console,
    Signal,
    Paper,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Signal
    }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Tile {
    #[serde(default)]
    pub label: String,
    /// Client-supplied icon: an SVG string (inlined), a `data:` URI (as <image>),
    /// or a built-in glyph name (e.g. "line"). None => generic fallback glyph.
    #[serde(default)]
    pub icon: Option<String>,
    /// Category key: draw|modify|constrain|view|cut -> colour. None => neutral.
    #[serde(default)]
    pub cat: Option<String>,
    #[serde(default)]
    pub active: bool,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LcdState {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub profile: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub left: Vec<Tile>,
    #[serde(default)]
    pub right: Vec<Tile>,
}
```

- [ ] **Step 4: Register the module** — add to `space-elevatord/src/lib.rs`:

```rust
pub mod lcd_template;
```

- [ ] **Step 5: Add the IPC variant** — in `space-elevatord/src/ipc.rs`, add a `use` and a variant. At the top, after the existing `use serde…` line add:

```rust
use crate::lcd_template::LcdState;
```

and extend `RequestPayload`:

```rust
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum RequestPayload {
    Ping,
    LcdClear,
    LcdDisplayImage { path: String },
    LcdDisplaySvg { svg: String },
    LcdSetState(LcdState),
}
```

(`Tile`/`LcdState` derive `PartialEq` so `RequestPayload` still derives it.)

- [ ] **Step 6: Run, expect PASS:**
`devenv shell -- bash -c 'cargo test -p space-elevatord parses_lcd_set_state lcd_set_state_defaults 2>&1 | tail -8'`
Expected: 2 passed.

- [ ] **Step 7: Commit**

```bash
git add space-elevatord/src/lcd_template.rs space-elevatord/src/lib.rs space-elevatord/src/ipc.rs
git commit -m "feat(daemon): lcd_set_state IPC + LcdState types"
```

---

## Task 2: Geometry helpers

**Files:** Modify `space-elevatord/src/lcd_template.rs`

Port the four SVG primitives from the reference (`L`, `dot`, `arrow`, `hexPts`). These build the glyph library in Task 3.

- [ ] **Step 1: Write the failing test** — append a `#[cfg(test)] mod tests` to `lcd_template.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_helper_emits_svg_line() {
        let s = line(4.0, 20.0, 20.0, 4.0, "#fff", 1.7);
        assert!(s.contains("<line"));
        assert!(s.contains("x1=\"4\""));
        assert!(s.contains("stroke=\"#fff\""));
        assert!(s.contains("stroke-width=\"1.7\""));
    }

    #[test]
    fn dot_helper_emits_circle() {
        assert!(dot(4.0, 20.0, "#abc").contains("<circle"));
        assert!(dot(4.0, 20.0, "#abc").contains("r=\"2.1\""));
    }
}
```

- [ ] **Step 2: Run, expect FAIL** (helpers undefined):
`devenv shell -- bash -c 'cargo test -p space-elevatord line_helper 2>&1 | tail -10'`

- [ ] **Step 3: Implement the helpers** — add to `lcd_template.rs` (translating the JS `L`, `dot`, `arrow`, `hexPts`; numbers format like JS: integers without trailing `.0`, two-decimal for computed coords):

```rust
fn num(n: f64) -> String {
    // Match JS number formatting: integers print without a decimal point.
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.2}", n);
        s
    }
}

fn line(x1: f64, y1: f64, x2: f64, y2: f64, c: &str, w: f64) -> String {
    format!(
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
        num(x1), num(y1), num(x2), num(y2), c, num(w)
    )
}

fn dot(x: f64, y: f64, c: &str) -> String {
    format!("<circle cx=\"{}\" cy=\"{}\" r=\"2.1\" fill=\"{}\"/>", num(x), num(y), c)
}

fn arrow(cx: f64, cy: f64, ang: f64, c: &str, sz: f64) -> String {
    let a = ang * std::f64::consts::PI / 180.0;
    let p = |d: f64, o: f64| {
        let x = cx + a.cos() * d - a.sin() * o;
        let y = cy + a.sin() * d + a.cos() * o;
        format!("{:.2},{:.2}", x, y)
    };
    format!(
        "<polygon points=\"{} {} {}\" fill=\"{}\"/>",
        p(0.0, 0.0),
        p(-sz, sz * 0.62),
        p(-sz, -sz * 0.62),
        c
    )
}

fn hex_pts(cx: f64, cy: f64, r: f64) -> String {
    (0..6)
        .map(|i| {
            let a = (i as f64 * 60.0) * std::f64::consts::PI / 180.0;
            format!("{:.2},{:.2}", cx + r * a.cos(), cy + r * a.sin())
        })
        .collect::<Vec<_>>()
        .join(" ")
}
```

- [ ] **Step 4: Run, expect PASS:**
`devenv shell -- bash -c 'cargo test -p space-elevatord line_helper dot_helper 2>&1 | tail -8'`

- [ ] **Step 5: Commit**

```bash
git add space-elevatord/src/lcd_template.rs
git commit -m "feat(daemon): SVG geometry helpers for glyph port"
```

---

## Task 3: Glyph library

**Files:** Modify `space-elevatord/src/lcd_template.rs`

Port the full `ICONS` library from `docs/lcd-template/lcd-template.js` (lines ~50-154). Each glyph is a function of icon options → SVG string. Represent options as a struct and each glyph as a match arm returning the same SVG the JS produces.

- [ ] **Step 1: Write the failing test** — add to the `tests` module:

```rust
    #[test]
    fn glyph_line_matches_reference_shape() {
        let o = IconOpts { c1: "#A", c2: "#B", sw: 1.7, fill: "none" };
        let g = glyph("line", &o);
        // line glyph = L(4,20,20,4) + dot(4,20) + dot(20,4)
        assert!(g.contains("<line"));
        assert_eq!(g.matches("<circle").count(), 2);
    }

    #[test]
    fn glyph_unknown_falls_back_to_point() {
        let o = IconOpts { c1: "#A", c2: "#B", sw: 1.7, fill: "none" };
        assert_eq!(glyph("does-not-exist", &o), glyph("point", &o));
    }
```

- [ ] **Step 2: Run, expect FAIL.**
`devenv shell -- bash -c 'cargo test -p space-elevatord glyph_ 2>&1 | tail -10'`

- [ ] **Step 3: Implement `IconOpts` + `glyph`** — translate EACH `ICONS.<name>` from the reference. Add to `lcd_template.rs`:

```rust
pub struct IconOpts<'a> {
    pub c1: &'a str,   // stroke
    pub c2: &'a str,   // accent
    pub sw: f64,       // stroke width
    pub fill: &'a str, // shape fill
}

/// Built-in glyph library. Ported 1:1 from docs/lcd-template/lcd-template.js
/// `ICONS`. Unknown names fall back to `point` (as the JS does).
pub fn glyph(name: &str, o: &IconOpts) -> String {
    match name {
        "line" => format!("{}{}{}",
            line(4.0, 20.0, 20.0, 4.0, o.c1, o.sw), dot(4.0, 20.0, o.c2), dot(20.0, 4.0, o.c2)),
        "point" => format!("{}{}<circle cx=\"12\" cy=\"12\" r=\"3.2\" fill=\"{}\"/>",
            line(12.0, 4.0, 12.0, 20.0, o.c1, o.sw * 0.7),
            line(4.0, 12.0, 20.0, 12.0, o.c1, o.sw * 0.7), o.c2),
        "rectangle" => format!(
            "<rect x=\"4\" y=\"6.5\" width=\"16\" height=\"11\" rx=\"1.4\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>{}{}",
            o.fill, o.c1, num(o.sw), dot(4.0, 6.5, o.c2), dot(20.0, 17.5, o.c2)),
        "circle" => format!(
            "<circle cx=\"12\" cy=\"12\" r=\"8\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/><circle cx=\"12\" cy=\"12\" r=\"1.5\" fill=\"{}\"/>",
            o.fill, o.c1, num(o.sw), o.c2),
        // ... PORT THE REMAINING GLYPHS HERE, 1:1 FROM THE REFERENCE:
        //   arc, polygon, slot, trim, construction, extend, coincident,
        //   horizontal, vertical, parallel, perpendicular, equal, tangent,
        //   dimension, radius, angle, fillet, fit
        // Each is a direct translation of the matching ICONS.<name> function in
        // docs/lcd-template/lcd-template.js using the line/dot/arrow/hex_pts
        // helpers from Task 2 and inline <rect>/<circle>/<path>/<polygon> exactly
        // as written there (same coordinates, same attribute order).
        _ => glyph("point", o),
    }
}
```

The remaining glyph arms are not optional and not "TBD" — they are the verbatim translations of the named functions in the reference file. Implement every one of: `arc, polygon, slot, trim, construction, extend, coincident, horizontal, vertical, parallel, perpendicular, equal, tangent, dimension, radius, angle, fillet, fit`. Read `docs/lcd-template/lcd-template.js` lines 50-154 and translate each. Example — `trim` in the reference is `L(3.5,12,20.5,12,c1,sw) + L(9,9,15,15,c2,sw) + L(15,9,9,15,c2,sw)`, so:

```rust
        "trim" => format!("{}{}{}",
            line(3.5, 12.0, 20.5, 12.0, o.c1, o.sw),
            line(9.0, 9.0, 15.0, 15.0, o.c2, o.sw),
            line(15.0, 9.0, 9.0, 15.0, o.c2, o.sw)),
```

- [ ] **Step 4: Run, expect PASS:**
`devenv shell -- bash -c 'cargo test -p space-elevatord glyph_ 2>&1 | tail -8'`

- [ ] **Step 5: Commit**

```bash
git add space-elevatord/src/lcd_template.rs
git commit -m "feat(daemon): port glyph library from LCD template"
```

---

## Task 4: Category palettes + themes

**Files:** Modify `space-elevatord/src/lcd_template.rs`

Port `CAT_DARK`, `CAT_LIGHT`, and the three `THEMES` entries from the reference (lines 162-204).

- [ ] **Step 1: Write the failing test:**

```rust
    #[test]
    fn theme_signal_palette() {
        let t = theme_def(Theme::Signal);
        assert_eq!(t.bg, "#0A0B0D");
        assert_eq!(cat_color(&t, Some("draw")), "#4DA3FF");
        assert_eq!(cat_color(&t, Some("constrain")), "#46D08A");
    }

    #[test]
    fn theme_paper_uses_light_palette() {
        let t = theme_def(Theme::Paper);
        assert_eq!(cat_color(&t, Some("draw")), "#1F6FE0");
    }

    #[test]
    fn cat_color_unknown_falls_back_to_label() {
        let t = theme_def(Theme::Signal);
        assert_eq!(cat_color(&t, None), t.label);
        assert_eq!(cat_color(&t, Some("nonsense")), t.label);
    }
```

- [ ] **Step 2: Run, expect FAIL.**
`devenv shell -- bash -c 'cargo test -p space-elevatord theme_ cat_color 2>&1 | tail -10'`

- [ ] **Step 3: Implement palettes + theme table:**

```rust
fn cat_dark(cat: &str) -> Option<&'static str> {
    Some(match cat {
        "draw" => "#4DA3FF",
        "modify" => "#FFB454",
        "constrain" => "#46D08A",
        "view" => "#B98CFF",
        "cut" => "#FF6B6B",
        _ => return None,
    })
}

fn cat_light(cat: &str) -> Option<&'static str> {
    Some(match cat {
        "draw" => "#1F6FE0",
        "modify" => "#B5760C",
        "constrain" => "#0E9152",
        "view" => "#7A47CF",
        "cut" => "#D7453F",
        _ => return None,
    })
}

pub struct ThemeDef {
    pub bg: &'static str,
    pub light_cats: bool,        // paper uses CAT_LIGHT
    pub tile_fill: &'static str, // "none" or colour
    pub tile_radius: f64,
    pub label: &'static str,
    pub label_weight: u32,
    pub label_size: f64,
    pub icon_sw: f64,
    pub icon_fill_alpha: Option<&'static str>, // hex alpha byte, e.g. "33"
    pub icon_mono: bool,
    pub icon_base_color: Option<&'static str>,
    pub gutter_line: &'static str,
    pub profile: &'static str,
    pub mode: &'static str,
    pub active_fg: &'static str,
    pub active_radius: f64,
}

fn theme_def(t: Theme) -> ThemeDef {
    match t {
        Theme::Console => ThemeDef {
            bg: "#0C0D10", light_cats: false, tile_fill: "none", tile_radius: 0.0,
            label: "#C7CBD3", label_weight: 500, label_size: 13.5,
            icon_sw: 1.7, icon_fill_alpha: None, icon_mono: false, icon_base_color: None,
            gutter_line: "#22252D", profile: "#5E646F", mode: "#AEB4BF",
            active_fg: "#0C0D10", active_radius: 9.0,
        },
        Theme::Signal => ThemeDef {
            bg: "#0A0B0D", light_cats: false, tile_fill: "#15171C", tile_radius: 9.0,
            label: "#E6E9EF", label_weight: 600, label_size: 13.5,
            icon_sw: 2.3, icon_fill_alpha: Some("33"), icon_mono: false, icon_base_color: None,
            gutter_line: "#20242C", profile: "#6A7180", mode: "#D6DBE4",
            active_fg: "#0A0B0D", active_radius: 9.0,
        },
        Theme::Paper => ThemeDef {
            bg: "#ECEAE4", light_cats: true, tile_fill: "none", tile_radius: 0.0,
            label: "#37332C", label_weight: 600, label_size: 13.5,
            icon_sw: 1.8, icon_fill_alpha: Some("1f"), icon_mono: true, icon_base_color: Some("#615C52"),
            gutter_line: "#CFCAC0", profile: "#7A746A", mode: "#37332C",
            active_fg: "#FCFBF8", active_radius: 9.0,
        },
    }
}

/// Category colour for a tile, falling back to the theme label colour (mirrors
/// `t.cat[it.cat] || t.label` in the reference).
fn cat_color(t: &ThemeDef, cat: Option<&str>) -> &'static str {
    let lookup = if t.light_cats { cat_light } else { cat_dark };
    cat.and_then(lookup).unwrap_or(t.label)
}
```

- [ ] **Step 4: Run, expect PASS:**
`devenv shell -- bash -c 'cargo test -p space-elevatord theme_ cat_color 2>&1 | tail -8'`

- [ ] **Step 5: Commit**

```bash
git add space-elevatord/src/lcd_template.rs
git commit -m "feat(daemon): port category palettes and themes"
```

---

## Task 5: Tile + cluster + render()

**Files:** Modify `space-elevatord/src/lcd_template.rs`

Port `renderTile`, `cluster`, `renderLCD` (reference lines 235-323) into `render(&LcdState) -> String`, plus client-icon embedding.

- [ ] **Step 1: Write the failing test:**

```rust
    fn sample() -> LcdState {
        LcdState {
            theme: Theme::Signal,
            profile: "FreeCAD".into(),
            mode: "Sketcher".into(),
            left: vec![Tile { label: "Line".into(), icon: Some("line".into()), cat: Some("draw".into()), active: false }],
            right: vec![Tile { label: "Fit".into(), icon: None, cat: Some("view".into()), active: true }],
        }
    }

    #[test]
    fn render_is_640x150_svg_with_theme_bg() {
        let svg = render(&sample());
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("width=\"640\" height=\"150\""));
        assert!(svg.contains("fill=\"#0A0B0D\"")); // signal bg
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn render_includes_labels_and_gutter_text() {
        let svg = render(&sample());
        assert!(svg.contains(">Line<"));
        assert!(svg.contains(">Fit<"));
        assert!(svg.contains(">FREECAD<")); // profile uppercased
        assert!(svg.contains(">Sketcher<"));
    }

    #[test]
    fn render_inlines_client_svg_icon() {
        let mut st = sample();
        st.left[0].icon = Some("<path d=\"M1 2\" id=\"client-glyph\"/>".into());
        let svg = render(&st);
        assert!(svg.contains("client-glyph"));
    }

    #[test]
    fn render_escapes_labels() {
        let mut st = sample();
        st.left[0].label = "A & B".into();
        assert!(render(&st).contains("A &amp; B"));
    }
```

- [ ] **Step 2: Run, expect FAIL.**
`devenv shell -- bash -c 'cargo test -p space-elevatord render_ 2>&1 | tail -10'`

- [ ] **Step 3: Implement render + helpers:**

```rust
const W: f64 = 640.0;
const H: f64 = 150.0;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// True if the icon string is client-supplied content (SVG markup or data URI)
/// rather than a built-in glyph name.
fn is_client_icon(icon: &str) -> bool {
    let t = icon.trim_start();
    t.starts_with('<') || t.starts_with("data:")
}

fn render_icon(icon: Option<&str>, o: &IconOpts) -> String {
    match icon {
        Some(s) if is_client_icon(s) => {
            let t = s.trim_start();
            if t.starts_with("data:") {
                format!("<image x=\"0\" y=\"0\" width=\"24\" height=\"24\" href=\"{}\"/>", s)
            } else {
                s.to_string() // inline SVG fragment, placed in the scaled <g>
            }
        }
        Some(name) => glyph(name, o),
        None => glyph("point", o),
    }
}

fn render_tile(it: &Tile, x: f64, y: f64, w: f64, h: f64, t: &ThemeDef) -> String {
    let cat = cat_color(t, it.cat.as_deref());
    let mut out = String::new();
    let pad = 4.0;
    let in_x = x + pad;
    let in_y = y + 5.0;
    let in_w = w - pad * 2.0;
    let in_h = h - 10.0;

    if it.active {
        out.push_str(&format!(
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"{}\" fill=\"{}\"/>",
            in_x, in_y, in_w, in_h, num(t.active_radius), cat));
    } else if t.tile_fill != "none" {
        out.push_str(&format!(
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"{}\" fill=\"{}\"/>",
            in_x, in_y, in_w, in_h, num(t.tile_radius), t.tile_fill));
    }

    // icon colours by theme (mirrors the reference's fg/mono/category logic)
    let (c1, c2, fill): (String, String, String) = if it.active {
        (t.active_fg.into(), t.active_fg.into(), format!("{}{}", t.active_fg, "30"))
    } else if t.icon_mono {
        let base = t.icon_base_color.unwrap_or(t.label);
        let f = t.icon_fill_alpha.map(|a| format!("{}{}", cat, a)).unwrap_or_else(|| "none".into());
        (base.into(), cat.into(), f)
    } else {
        let f = t.icon_fill_alpha.map(|a| format!("{}{}", cat, a)).unwrap_or_else(|| "none".into());
        (cat.into(), cat.into(), f)
    };

    let icon_size = 27.0;
    let s = icon_size / 24.0;
    let cxp = x + w / 2.0;
    let icon_cy = y + h * 0.36;
    let o = IconOpts { c1: &c1, c2: &c2, sw: t.icon_sw, fill: &fill };
    let g = render_icon(it.icon.as_deref(), &o);
    out.push_str(&format!(
        "<g transform=\"translate({:.2},{:.2}) scale({:.3})\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</g>",
        cxp - 12.0 * s, icon_cy - 12.0 * s, s, g));

    let lbl_color = if it.active { t.active_fg } else { t.label };
    out.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"'Source Sans 3','DejaVu Sans','Segoe UI',sans-serif\" font-size=\"{}\" font-weight=\"{}\" letter-spacing=\"0.1\" fill=\"{}\">{}</text>",
        cxp, y + h * 0.80, num(t.label_size), t.label_weight, lbl_color, esc(&it.label)));

    out
}

fn cluster(items: &[Tile], x0: f64, x1: f64, t: &ThemeDef) -> String {
    let top = 6.0;
    let bottom = H - 6.0;
    let col_w = (x1 - x0) / 3.0;
    let row_h = (bottom - top) / 2.0;
    let mut out = String::new();
    for (i, it) in items.iter().take(6).enumerate() {
        let col = (i % 3) as f64;
        let row = (i / 3) as f64;
        out.push_str(&render_tile(it, x0 + col * col_w, top + row * row_h, col_w, row_h, t));
    }
    out
}

pub fn render(state: &LcdState) -> String {
    let t = theme_def(state.theme);
    let gutter_w = 40.0;
    let cx = W / 2.0;
    let g_x0 = cx - gutter_w / 2.0;
    let g_x1 = cx + gutter_w / 2.0;

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"640\" height=\"150\" viewBox=\"0 0 640 150\" shape-rendering=\"geometricPrecision\">"));
    svg.push_str(&format!("<rect width=\"640\" height=\"150\" fill=\"{}\"/>", t.bg));
    svg.push_str(&cluster(&state.left, 6.0, g_x0, &t));
    svg.push_str(&cluster(&state.right, g_x1, W - 6.0, &t));

    svg.push_str(&format!(
        "<line x1=\"{}\" y1=\"20\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>",
        num(cx), num(cx), num(H - 20.0), t.gutter_line));
    svg.push_str(&format!(
        "<g transform=\"translate({},{}) rotate(-90)\" font-family=\"'Source Sans 3','DejaVu Sans',sans-serif\" text-anchor=\"middle\">",
        num(cx), num(H / 2.0)));
    svg.push_str(&format!(
        "<text x=\"0\" y=\"-4.5\" font-size=\"9.5\" font-weight=\"600\" letter-spacing=\"2\" fill=\"{}\">{}</text>",
        t.profile, esc(&state.profile.to_uppercase())));
    svg.push_str(&format!(
        "<text x=\"0\" y=\"8.5\" font-size=\"13\" font-weight=\"700\" letter-spacing=\"0.5\" fill=\"{}\">{}</text>",
        t.mode, esc(&state.mode)));
    svg.push_str("</g>");
    svg.push_str("</svg>");
    svg
}
```

- [ ] **Step 4: Run, expect PASS:**
`devenv shell -- bash -c 'cargo test -p space-elevatord render_ 2>&1 | tail -10'`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add space-elevatord/src/lcd_template.rs
git commit -m "feat(daemon): render LcdState to template SVG"
```

---

## Task 6: Dispatch lcd_set_state → LCD

**Files:** Modify `space-elevatord/src/server.rs`

- [ ] **Step 1: Read the current `dispatch` fn** in `space-elevatord/src/server.rs` to see the existing match arms (`Ping`, `LcdClear`, `LcdDisplayImage`, `LcdDisplaySvg`) and how each renders/sends via the `lcd` handle and `crate::svg_render`.

- [ ] **Step 2: Add the `LcdSetState` arm** inside `dispatch`, mirroring the `LcdDisplaySvg` arm but rendering the state first:

```rust
        RequestPayload::LcdSetState(state) => {
            let svg = crate::lcd_template::render(&state);
            match crate::svg_render::render_to_rgb888(&svg) {
                Ok(rgb) => match lcd.display_rgb888(&rgb).await {
                    Ok(()) => Response::ok(req.id),
                    Err(e) => Response::err(req.id, e.to_string()),
                },
                Err(e) => Response::err(req.id, e),
            }
        }
```

(Match the exact binding names used by the existing `LcdDisplaySvg` arm — e.g. if it uses `lcd` and `req.id`, use the same.)

- [ ] **Step 3: Build + run the full daemon test suite:**
`devenv shell -- bash -c 'cargo test -p space-elevatord 2>&1 | tail -12'`
Expected: all pass (ipc parse tests + lcd_template tests + existing ipc/svg_render tests).

- [ ] **Step 4: Commit**

```bash
git add space-elevatord/src/server.rs
git commit -m "feat(daemon): dispatch lcd_set_state to the LCD"
```

---

## Task 7: Smoke-test client + headless render check

**Files:** Create `space-elevatord/examples/send_test_state.rs`

A client that sends a representative `lcd_set_state` (mirrors the design DEMO) over the socket, for manual on-device verification. Also a headless test that the full state→rgb888 pipeline yields a correctly-sized frame.

- [ ] **Step 1: Add a headless pipeline test** to `lcd_template.rs` tests:

```rust
    #[test]
    fn render_rasterizes_to_correct_frame_size() {
        let svg = render(&sample());
        let rgb = crate::svg_render::render_to_rgb888(&svg).expect("rasterize");
        assert_eq!(rgb.len(), 640 * 150 * 3);
    }
```

- [ ] **Step 2: Run, expect PASS:**
`devenv shell -- bash -c 'cargo test -p space-elevatord render_rasterizes 2>&1 | tail -8'`

- [ ] **Step 3: Create `space-elevatord/examples/send_test_state.rs`:**

```rust
//! Manual smoke test: send a demo lcd_set_state to a running space-elevatord.
//! Uses the same XDG_RUNTIME_DIR as the daemon.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn socket_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").expect("set XDG_RUNTIME_DIR to match the daemon");
    PathBuf::from(dir).join("space-elevator.sock")
}

fn main() -> std::io::Result<()> {
    let req = r#"{"v":1,"id":1,"cmd":"lcd_set_state","theme":"signal","profile":"FreeCAD","mode":"Sketch",
"left":[
 {"label":"Line","icon":"line","cat":"draw"},
 {"label":"Rectangle","icon":"rectangle","cat":"draw"},
 {"label":"Circle","icon":"circle","cat":"draw"},
 {"label":"Arc","icon":"arc","cat":"draw"},
 {"label":"Trim","icon":"trim","cat":"modify"},
 {"label":"Construction","icon":"construction","cat":"modify","active":true}],
"right":[
 {"label":"Coincident","icon":"coincident","cat":"constrain"},
 {"label":"Horizontal","icon":"horizontal","cat":"constrain"},
 {"label":"Vertical","icon":"vertical","cat":"constrain"},
 {"label":"Parallel","icon":"parallel","cat":"constrain"},
 {"label":"Equal","icon":"equal","cat":"constrain"},
 {"label":"Dimension","icon":"dimension","cat":"view"}]}"#
        .replace('\n', "");

    let path = socket_path();
    println!("connecting to {}", path.display());
    let mut s = UnixStream::connect(&path)?;
    s.write_all(req.as_bytes())?;
    s.write_all(b"\n")?;
    let mut buf = [0u8; 8192];
    let n = s.read(&mut buf)?;
    println!("response: {}", String::from_utf8_lossy(&buf[..n]).trim());
    Ok(())
}
```

- [ ] **Step 4: Build the example:**
`devenv shell -- bash -c 'cargo build -p space-elevatord --example send_test_state 2>&1 | tail -4'`
Expected: Finished.

- [ ] **Step 5: Commit**

```bash
git add space-elevatord/src/lcd_template.rs space-elevatord/examples/send_test_state.rs
git commit -m "feat(daemon): state pipeline test + send_test_state smoke client"
```

- [ ] **Step 6: Manual device verification** (needs hardware + the temp udev rule):

```bash
sudo systemctl start spacenavd   # for coexistence; not strictly needed for LCD
./target/debug/space-elevatord &
XDG_RUNTIME_DIR=/run/user/1000 ./target/debug/examples/send_test_state
```

Expect `response: {"v":1,"id":1,"ok":true}` and the Signal-theme 12-tile layout on the LCD (left draw cluster, right constrain cluster, "Construction" highlighted, "FREECAD / Sketch" in the centre gutter).

---

## Notes for the implementer

- The render port must match `docs/lcd-template/lcd-template.js` exactly for the geometry/colours — that JS is the authoritative design. When unsure about a glyph or theme value, read it there; do not invent.
- Number formatting: the JS prints integer coordinates without a decimal (`x1="4"`) and computed coords with `.toFixed(2)`. The `num()` helper and explicit `{:.1}`/`{:.2}` in the code above reproduce this; keep using them so the output matches.
- Fonts: headless rasterisation needs `Source Sans 3` / `DejaVu Sans` available to fontconfig (already handled for the daemon on the desktop; see the earlier font-loading work in `svg_render`).
- This plan is the daemon only. The thin FreeCAD client (reads FreeCAD's global spaceball bindings → sends `lcd_set_state`) is a SEPARATE plan, pending a discovery spike: dump `BaseApp/Preferences/Spaceball` and confirm how to resolve a command's SVG icon from Python.
```

