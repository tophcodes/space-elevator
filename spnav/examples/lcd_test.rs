// examples/lcd_test.rs

use spnav::SpaceNav;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SpaceMouse Enterprise LCD Test\n");

    // Connect to LCD
    println!("Connecting to SpaceMouse Enterprise...");
    let mut device = SpaceNav::connect()?;
    println!("Serial: {}", device.config().get_serial()?);
    let mut lcd = device.lcd()?;
    println!("✓ Connected!\n");

    // Create a striped test pattern
    println!("Generating striped test pattern...");
    let bitmap = create_striped_pattern();

    println!("Sending to display...");
    lcd.display_bitmap(&bitmap)?;

    println!("✓ Done! Check your SpaceMouse display.");
    println!("\nPress Enter to clear and exit...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    lcd.clear()?;
    println!("✓ Display cleared");

    Ok(())
}

/// Create a simple striped test pattern (640x150 RGB)
fn create_striped_pattern() -> Vec<u8> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 150;
    const STRIPE_WIDTH: usize = 80; // 8 stripes

    let mut bitmap = Vec::with_capacity(WIDTH * HEIGHT * 3);

    // Colors for each stripe (8 colors)
    let colors = [
        (255, 0, 0),     // Red
        (255, 165, 0),   // Orange
        (255, 255, 0),   // Yellow
        (0, 255, 0),     // Green
        (0, 127, 255),   // Blue
        (75, 0, 130),    // Indigo
        (148, 0, 211),   // Violet
        (255, 255, 255), // White
    ];

    for _ in 0..HEIGHT {
        for x in 0..WIDTH {
            // Determine which stripe we're in
            let stripe_index = (x / STRIPE_WIDTH).min(7);
            let (r, g, b) = colors[stripe_index];

            bitmap.push(r);
            bitmap.push(g);
            bitmap.push(b);
        }
    }

    bitmap
}
