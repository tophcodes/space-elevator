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
