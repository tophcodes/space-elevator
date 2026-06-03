//! End-to-end (headless, no device) check: an LcdState renders to a template
//! SVG and rasterises to a full 640x150 RGB888 frame — the same path the daemon
//! drives the LCD with. Requires DejaVu Sans / Source Sans 3 fonts installed for
//! the text glyphs (resvg loads system fonts).

use space_elevatord::lcd_template::{render, LcdState, Theme, Tile};
use space_elevatord::svg_render::{render_to_rgb888, LCD_HEIGHT, LCD_WIDTH};

fn tile(label: &str, icon: &str, cat: &str, active: bool) -> Tile {
    Tile { label: label.into(), icon: Some(icon.into()), cat: Some(cat.into()), active }
}

#[test]
fn set_state_renders_to_full_rgb888_frame() {
    let state = LcdState {
        theme: Theme::Signal,
        profile: "FreeCAD".into(),
        mode: "Sketch".into(),
        left: vec![
            tile("Line", "line", "draw", false),
            tile("Construction", "construction", "modify", true),
        ],
        right: vec![tile("Coincident", "coincident", "constrain", false)],
    };

    let svg = render(&state);
    let rgb = render_to_rgb888(&svg).expect("rasterise template svg");
    assert_eq!(rgb.len(), (LCD_WIDTH * LCD_HEIGHT * 3) as usize);
}

#[test]
fn all_themes_rasterise() {
    for theme in [Theme::Console, Theme::Signal, Theme::Paper] {
        let state = LcdState {
            theme,
            profile: "FreeCAD".into(),
            mode: "Part".into(),
            left: vec![tile("Box", "rectangle", "draw", false)],
            right: vec![],
        };
        let rgb = render_to_rgb888(&render(&state)).expect("rasterise");
        assert_eq!(rgb.len(), (LCD_WIDTH * LCD_HEIGHT * 3) as usize);
    }
}
