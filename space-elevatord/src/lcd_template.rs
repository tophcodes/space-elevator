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

fn num(n: f64) -> String {
    // Match JS number formatting: integers without decimal point, floats with
    // up to 2 decimal places but trailing zeros stripped (e.g. 1.70 → "1.7").
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.2}", n);
        s.trim_end_matches('0').to_string()
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

pub struct IconOpts<'a> {
    pub c1: &'a str,
    pub c2: &'a str,
    pub sw: f64,
    pub fill: &'a str,
}

/// Built-in glyph library, ported 1:1 from docs/lcd-template/lcd-template.js
/// `ICONS`. Unknown names fall back to `point` (as the JS does).
pub fn glyph(name: &str, o: &IconOpts) -> String {
    match name {
        "line" => format!(
            "{}{}{}",
            line(4.0, 20.0, 20.0, 4.0, o.c1, o.sw),
            dot(4.0, 20.0, o.c2),
            dot(20.0, 4.0, o.c2)
        ),
        "point" => format!(
            "{}{}<circle cx=\"12\" cy=\"12\" r=\"3.2\" fill=\"{}\"/>",
            line(12.0, 4.0, 12.0, 20.0, o.c1, o.sw * 0.7),
            line(4.0, 12.0, 20.0, 12.0, o.c1, o.sw * 0.7),
            o.c2
        ),
        "rectangle" => format!(
            "<rect x=\"4\" y=\"6.5\" width=\"16\" height=\"11\" rx=\"1.4\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>{}{}",
            o.fill, o.c1, num(o.sw), dot(4.0, 6.5, o.c2), dot(20.0, 17.5, o.c2)
        ),
        "circle" => format!(
            "<circle cx=\"12\" cy=\"12\" r=\"8\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/><circle cx=\"12\" cy=\"12\" r=\"1.5\" fill=\"{}\"/>",
            o.fill, o.c1, num(o.sw), o.c2
        ),
        "arc" => format!(
            "<path d=\"M4 18.5 A 11 11 0 0 1 20 18.5\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>{}{}{}",
            o.c1, num(o.sw), dot(4.0, 18.5, o.c2), dot(20.0, 18.5, o.c2), dot(12.0, 18.5, o.c2)
        ),
        "polygon" => format!(
            "<polygon points=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"round\"/>",
            hex_pts(12.0, 12.0, 8.5), o.fill, o.c1, num(o.sw)
        ),
        "slot" => format!(
            "<rect x=\"3\" y=\"8\" width=\"18\" height=\"8\" rx=\"4\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>{}{}",
            o.fill, o.c1, num(o.sw), dot(7.0, 12.0, o.c2), dot(17.0, 12.0, o.c2)
        ),
        "trim" => format!(
            "{}{}{}",
            line(3.5, 12.0, 20.5, 12.0, o.c1, o.sw),
            line(9.0, 9.0, 15.0, 15.0, o.c2, o.sw),
            line(15.0, 9.0, 9.0, 15.0, o.c2, o.sw)
        ),
        "construction" => format!(
            "<rect x=\"4\" y=\"6.5\" width=\"16\" height=\"11\" rx=\"1\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-dasharray=\"3 2.4\"/>{}{}",
            o.c1, num(o.sw), dot(4.0, 6.5, o.c2), dot(20.0, 17.5, o.c2)
        ),
        "extend" => format!(
            "{}<path d=\"M14 12 L18.5 12\" stroke=\"{}\" stroke-width=\"{}\" stroke-dasharray=\"2 2\"/>{}{}{}",
            line(3.5, 12.0, 14.0, 12.0, o.c1, o.sw),
            o.c2, num(o.sw),
            arrow(19.5, 12.0, 0.0, o.c2, 3.6),
            line(21.0, 6.5, 21.0, 17.5, o.c1, o.sw),
            ""
        ),
        "coincident" => format!(
            "<circle cx=\"12\" cy=\"12\" r=\"6.2\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/><circle cx=\"12\" cy=\"12\" r=\"2.6\" fill=\"{}\"/>",
            o.c1, num(o.sw), o.c2
        ),
        "horizontal" => format!(
            "{}{}{}",
            line(3.5, 12.0, 20.5, 12.0, o.c1, o.sw),
            dot(3.5, 12.0, o.c2),
            dot(20.5, 12.0, o.c2)
        ),
        "vertical" => format!(
            "{}{}{}",
            line(12.0, 3.5, 12.0, 20.5, o.c1, o.sw),
            dot(12.0, 3.5, o.c2),
            dot(12.0, 20.5, o.c2)
        ),
        "parallel" => format!(
            "{}{}",
            line(6.0, 20.0, 13.0, 4.0, o.c1, o.sw),
            line(12.0, 20.0, 19.0, 4.0, o.c1, o.sw)
        ),
        "perpendicular" => format!(
            "{}{}<rect x=\"6\" y=\"13.5\" width=\"4.5\" height=\"4.5\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>",
            line(6.0, 3.5, 6.0, 18.0, o.c1, o.sw),
            line(6.0, 18.0, 20.5, 18.0, o.c1, o.sw),
            o.c2, num(o.sw * 0.8)
        ),
        "equal" => format!(
            "{}{}{}{}",
            line(5.0, 9.5, 19.0, 9.5, o.c1, o.sw),
            line(5.0, 14.5, 19.0, 14.5, o.c1, o.sw),
            dot(5.0, 9.5, o.c2),
            dot(19.0, 14.5, o.c2)
        ),
        "tangent" => format!(
            "<circle cx=\"9.5\" cy=\"14.5\" r=\"6\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>{}{}",
            o.fill, o.c1, num(o.sw),
            line(2.5, 8.5, 21.5, 8.5, o.c1, o.sw),
            dot(9.5, 8.5, o.c2)
        ),
        "dimension" => format!(
            "{}{}{}{}{}",
            line(4.0, 6.5, 4.0, 18.0, o.c1, o.sw * 0.8),
            line(20.0, 6.5, 20.0, 18.0, o.c1, o.sw * 0.8),
            line(4.0, 13.5, 20.0, 13.5, o.c1, o.sw),
            arrow(4.5, 13.5, 180.0, o.c2, 3.6),
            arrow(19.5, 13.5, 0.0, o.c2, 3.6)
        ),
        "radius" => format!(
            "<circle cx=\"11\" cy=\"13\" r=\"8\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>{}{}{}",
            o.fill, o.c1, num(o.sw),
            line(11.0, 13.0, 18.5, 8.0, o.c1, o.sw),
            arrow(18.5, 8.0, -34.0, o.c2, 3.6),
            dot(11.0, 13.0, o.c2)
        ),
        "angle" => format!(
            "{}{}<path d=\"M16 20 A 11 11 0 0 0 12.2 12.2\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>",
            line(5.0, 20.0, 21.0, 20.0, o.c1, o.sw),
            line(5.0, 20.0, 19.0, 7.0, o.c1, o.sw),
            o.c2, num(o.sw * 0.85)
        ),
        "fillet" => format!(
            "<path d=\"M6 4 L6 13 Q6 18 11 18 L20 18\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/><path d=\"M6 18 L6 18 M6 18 L20 18\" fill=\"none\"/><path d=\"M6 13 L6 18 L11 18\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-dasharray=\"1.6 1.8\"/>",
            o.c1, num(o.sw),
            o.c2, num(o.sw * 0.8)
        ),
        "fit" => {
            let k = |x: f64, y: f64, rx: f64, ry: f64| -> String {
                format!(
                    "<path d=\"M{} {} L{} {} L{} {}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
                    x, y + 5.0 * ry, x, y, x + 5.0 * rx, y,
                    o.c1, num(o.sw)
                )
            };
            format!(
                "{}{}{}{}<rect x=\"9.5\" y=\"9.5\" width=\"5\" height=\"5\" rx=\"0.8\" fill=\"{}\"/>",
                k(4.0, 4.0, 1.0, 1.0),
                k(20.0, 4.0, -1.0, 1.0),
                k(4.0, 20.0, 1.0, -1.0),
                k(20.0, 20.0, -1.0, -1.0),
                o.c2
            )
        }
        _ => glyph("point", o),
    }
}

// --- palettes + themes (port of CAT_DARK/CAT_LIGHT + THEMES) ---------------

const W: f64 = 640.0;
const H: f64 = 150.0;

/// Resolved theme palette. Mirrors the JS `THEMES[name]` objects.
struct ThemeDef {
    bg: &'static str,
    /// false => CAT_DARK, true => CAT_LIGHT
    light_cat: bool,
    tile_fill: &'static str,
    tile_radius: i32,
    label: &'static str,
    label_weight: i32,
    label_size: f64,
    icon_sw: f64,
    /// Translucent category fill behind strokes (alpha byte), or None.
    icon_fill_alpha: Option<&'static str>,
    /// Fallback icon fill when no alpha (JS `icon.fill || 'none'`).
    icon_fill: &'static str,
    /// Two-tone: grey body (icon_base_color) + category accent.
    icon_mono: bool,
    icon_base_color: Option<&'static str>,
    gutter_line: &'static str,
    profile: &'static str,
    mode: &'static str,
    active_fg: &'static str,
    active_radius: i32,
}

/// Category colour for a key, or None for unknown/neutral (caller falls back
/// to the theme label colour, as JS `t.cat[it.cat] || t.label`).
fn cat_color(light: bool, key: &str) -> Option<&'static str> {
    let (draw, modify, constrain, view, cut) = if light {
        ("#1F6FE0", "#B5760C", "#0E9152", "#7A47CF", "#D7453F")
    } else {
        ("#4DA3FF", "#FFB454", "#46D08A", "#B98CFF", "#FF6B6B")
    };
    Some(match key {
        "draw" => draw,
        "modify" => modify,
        "constrain" => constrain,
        "view" => view,
        "cut" => cut,
        _ => return None,
    })
}

fn theme_def(theme: Theme) -> ThemeDef {
    match theme {
        Theme::Console => ThemeDef {
            bg: "#0C0D10",
            light_cat: false,
            tile_fill: "none",
            tile_radius: 0,
            label: "#C7CBD3",
            label_weight: 500,
            label_size: 13.5,
            icon_sw: 1.7,
            icon_fill_alpha: None,
            icon_fill: "none",
            icon_mono: false,
            icon_base_color: None,
            gutter_line: "#22252D",
            profile: "#5E646F",
            mode: "#AEB4BF",
            active_fg: "#0C0D10",
            active_radius: 9,
        },
        Theme::Signal => ThemeDef {
            bg: "#0A0B0D",
            light_cat: false,
            tile_fill: "#15171C",
            tile_radius: 9,
            label: "#E6E9EF",
            label_weight: 600,
            label_size: 13.5,
            icon_sw: 2.3,
            icon_fill_alpha: Some("33"),
            icon_fill: "none",
            icon_mono: false,
            icon_base_color: None,
            gutter_line: "#20242C",
            profile: "#6A7180",
            mode: "#D6DBE4",
            active_fg: "#0A0B0D",
            active_radius: 9,
        },
        Theme::Paper => ThemeDef {
            bg: "#ECEAE4",
            light_cat: true,
            tile_fill: "none",
            tile_radius: 0,
            label: "#37332C",
            label_weight: 600,
            label_size: 13.5,
            icon_sw: 1.8,
            icon_fill_alpha: Some("1f"),
            icon_fill: "none",
            icon_mono: true,
            icon_base_color: Some("#615C52"),
            gutter_line: "#CFCAC0",
            profile: "#7A746A",
            mode: "#37332C",
            active_fg: "#FCFBF8",
            active_radius: 9,
        },
    }
}

// --- renderer (port of renderTile/cluster/renderLCD) ----------------------

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn f1(n: f64) -> String {
    format!("{:.1}", n)
}

/// Whether a tile icon string is client-supplied content (inline SVG or a
/// `data:` URI) rather than a built-in glyph name.
fn is_client_icon(s: &str) -> bool {
    let t = s.trim_start();
    t.starts_with('<') || t.starts_with("data:")
}

fn render_icon(icon: Option<&str>, o: &IconOpts, s: f64, cxp: f64, icon_cy: f64) -> String {
    let tx = cxp - 12.0 * s;
    let ty = icon_cy - 12.0 * s;
    match icon {
        Some(c) if is_client_icon(c) => {
            let t = c.trim_start();
            if t.starts_with("data:") {
                // raster icon -> <image> scaled into the 24x24 icon box
                format!(
                    "<g transform=\"translate({:.2},{:.2}) scale({:.3})\"><image href=\"{}\" x=\"0\" y=\"0\" width=\"24\" height=\"24\"/></g>",
                    tx, ty, s, esc(c)
                )
            } else {
                // inline SVG content (client supplies its own colours)
                format!(
                    "<g transform=\"translate({:.2},{:.2}) scale({:.3})\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</g>",
                    tx, ty, s, c
                )
            }
        }
        other => {
            // built-in glyph by name; None => generic fallback (glyph() maps
            // unknown names to "point" too)
            let g = glyph(other.unwrap_or("point"), o);
            format!(
                "<g transform=\"translate({:.2},{:.2}) scale({:.3})\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</g>",
                tx, ty, s, g
            )
        }
    }
}

fn render_tile(it: &Tile, x: f64, y: f64, w: f64, h: f64, t: &ThemeDef) -> String {
    let cat = cat_color(t.light_cat, it.cat.as_deref().unwrap_or("")).unwrap_or(t.label);
    let mut out = String::new();
    let pad = 4.0;
    let in_x = x + pad;
    let in_y = y + 5.0;
    let in_w = w - pad * 2.0;
    let in_h = h - 10.0;

    // tile background or active highlight
    if it.active {
        out.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\"/>",
            f1(in_x), f1(in_y), f1(in_w), f1(in_h), t.active_radius, cat
        ));
    } else if t.tile_fill != "none" {
        out.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\"/>",
            f1(in_x), f1(in_y), f1(in_w), f1(in_h), t.tile_radius, t.tile_fill
        ));
    }

    // icon colours by theme/state
    let (c1, c2, fill): (String, String, String);
    if it.active {
        let fg = t.active_fg;
        c1 = fg.to_string();
        c2 = fg.to_string();
        fill = format!("{}30", fg);
    } else if t.icon_mono {
        c1 = t.icon_base_color.unwrap_or(t.label).to_string();
        c2 = cat.to_string();
        fill = match t.icon_fill_alpha {
            Some(a) => format!("{}{}", cat, a),
            None => t.icon_fill.to_string(),
        };
    } else {
        c1 = cat.to_string();
        c2 = cat.to_string();
        fill = match t.icon_fill_alpha {
            Some(a) => format!("{}{}", cat, a),
            None => t.icon_fill.to_string(),
        };
    }

    let s = 27.0 / 24.0;
    let cxp = x + w / 2.0;
    let icon_cy = y + h * 0.36;
    let o = IconOpts { c1: &c1, c2: &c2, sw: t.icon_sw, fill: &fill };
    out.push_str(&render_icon(it.icon.as_deref(), &o, s, cxp, icon_cy));

    // label
    let lbl_color = if it.active { t.active_fg } else { t.label };
    out.push_str(&format!(
        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"'Source Sans 3','DejaVu Sans','Segoe UI',sans-serif\" font-size=\"{}\" font-weight=\"{}\" letter-spacing=\"0.1\" fill=\"{}\">{}</text>",
        f1(cxp), f1(y + h * 0.80), num(t.label_size), t.label_weight, lbl_color, esc(&it.label)
    ));

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

/// Render a full 640x150 LCD SVG from client state. Port of `renderLCD`.
pub fn render(state: &LcdState) -> String {
    let t = theme_def(state.theme);
    let gutter_w = 40.0;
    let cx = W / 2.0;
    let g_x0 = cx - gutter_w / 2.0;
    let g_x1 = cx + gutter_w / 2.0;
    let left_x0 = 6.0;
    let left_x1 = g_x0;
    let right_x0 = g_x1;
    let right_x1 = W - 6.0;

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" shape-rendering=\"geometricPrecision\">",
        num(W), num(H), num(W), num(H)
    ));
    svg.push_str(&format!("<rect width=\"{}\" height=\"{}\" fill=\"{}\"/>", num(W), num(H), t.bg));

    svg.push_str(&cluster(&state.left, left_x0, left_x1, &t));
    svg.push_str(&cluster(&state.right, right_x0, right_x1, &t));

    // centre gutter: hairline + rotated profile / mode label
    svg.push_str(&format!(
        "<line x1=\"{}\" y1=\"20\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>",
        num(cx), num(cx), num(H - 20.0), t.gutter_line
    ));
    svg.push_str(&format!(
        "<g transform=\"translate({},{}) rotate(-90)\" font-family=\"'Source Sans 3','DejaVu Sans',sans-serif\" text-anchor=\"middle\">",
        num(cx), num(H / 2.0)
    ));
    svg.push_str(&format!(
        "<text x=\"0\" y=\"-4.5\" font-size=\"9.5\" font-weight=\"600\" letter-spacing=\"2\" fill=\"{}\">{}</text>",
        t.profile, esc(&state.profile.to_uppercase())
    ));
    svg.push_str(&format!(
        "<text x=\"0\" y=\"8.5\" font-size=\"13\" font-weight=\"700\" letter-spacing=\"0.5\" fill=\"{}\">{}</text>",
        t.mode, esc(&state.mode)
    ));
    svg.push_str("</g>");
    svg.push_str("</svg>");
    svg
}

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

    #[test]
    fn glyph_line_matches_reference_shape() {
        let o = IconOpts { c1: "#A", c2: "#B", sw: 1.7, fill: "none" };
        let g = glyph("line", &o);
        assert!(g.contains("<line"));
        assert_eq!(g.matches("<circle").count(), 2);
    }

    #[test]
    fn glyph_unknown_falls_back_to_point() {
        let o = IconOpts { c1: "#A", c2: "#B", sw: 1.7, fill: "none" };
        assert_eq!(glyph("does-not-exist", &o), glyph("point", &o));
    }

    fn demo_state() -> LcdState {
        LcdState {
            theme: Theme::Signal,
            profile: "FreeCAD".into(),
            mode: "Sketch".into(),
            left: vec![
                Tile { label: "Line".into(), icon: Some("line".into()), cat: Some("draw".into()), active: false },
                Tile { label: "Construction".into(), icon: Some("construction".into()), cat: Some("modify".into()), active: true },
            ],
            right: vec![
                Tile { label: "Coincident".into(), icon: Some("coincident".into()), cat: Some("constrain".into()), active: false },
            ],
        }
    }

    #[test]
    fn render_emits_canvas_and_gutter() {
        let svg = render(&demo_state());
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("viewBox=\"0 0 640 150\""));
        // Signal bg + gutter text
        assert!(svg.contains("fill=\"#0A0B0D\""));
        assert!(svg.contains(">FREECAD<"));
        assert!(svg.contains(">Sketch<"));
    }

    #[test]
    fn render_active_tile_uses_category_fill() {
        let svg = render(&demo_state());
        // active modify tile -> rounded rect filled with modify colour
        assert!(svg.contains("rx=\"9\" fill=\"#FFB454\""));
    }

    #[test]
    fn theme_paper_uses_light_palette() {
        let mut st = demo_state();
        st.theme = Theme::Paper;
        let svg = render(&st);
        assert!(svg.contains("fill=\"#ECEAE4\"")); // paper bg
        assert!(svg.contains("#1F6FE0")); // light draw colour
    }

    #[test]
    fn client_svg_icon_inlined_verbatim() {
        let mut st = demo_state();
        st.left[0].icon = Some("<path d=\"M0 0\" stroke=\"red\"/>".into());
        let svg = render(&st);
        assert!(svg.contains("<path d=\"M0 0\" stroke=\"red\"/>"));
    }

    #[test]
    fn client_data_uri_icon_becomes_image() {
        let mut st = demo_state();
        st.left[0].icon = Some("data:image/png;base64,AAAA".into());
        let svg = render(&st);
        assert!(svg.contains("<image href=\"data:image/png;base64,AAAA\""));
    }

    #[test]
    fn cluster_caps_at_six_tiles() {
        let mut st = demo_state();
        st.left = (0..9)
            .map(|i| Tile { label: format!("T{i}"), icon: Some("point".into()), cat: None, active: false })
            .collect();
        let svg = render(&st);
        // only first 6 labels rendered
        assert!(svg.contains(">T5<"));
        assert!(!svg.contains(">T6<"));
    }
}
