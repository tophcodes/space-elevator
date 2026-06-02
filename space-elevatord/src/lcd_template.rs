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
