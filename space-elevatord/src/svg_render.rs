use resvg::tiny_skia;
use usvg::Tree;

pub const LCD_WIDTH: u32 = 640;
pub const LCD_HEIGHT: u32 = 150;

pub fn render_to_rgb888(svg: &str) -> Result<Vec<u8>, String> {
    let opt = usvg::Options::default();
    let tree = Tree::from_str(svg, &opt).map_err(|e| format!("svg parse: {e}"))?;

    let mut pixmap = tiny_skia::Pixmap::new(LCD_WIDTH, LCD_HEIGHT)
        .ok_or("pixmap alloc")?;

    let view_w = tree.size().width();
    let view_h = tree.size().height();
    let scale_x = LCD_WIDTH as f32 / view_w;
    let scale_y = LCD_HEIGHT as f32 / view_h;
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut rgb = Vec::with_capacity((LCD_WIDTH * LCD_HEIGHT * 3) as usize);
    for px in pixmap.pixels() {
        rgb.push(px.red());
        rgb.push(px.green());
        rgb.push(px.blue());
    }
    Ok(rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_solid_rectangle() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 150"><rect width="640" height="150" fill="#ff0000"/></svg>"##;
        let rgb = render_to_rgb888(svg).unwrap();
        assert_eq!(rgb.len(), (640 * 150 * 3) as usize);
        // First pixel red
        assert!(rgb[0] > 200 && rgb[1] < 50 && rgb[2] < 50);
    }
}
