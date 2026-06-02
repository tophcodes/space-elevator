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
}
