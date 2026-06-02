use spnav::lcd::{Lcd, ScrollMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SpaceMouse Enterprise LCD Test\n");

    println!("Connecting...");
    let mut lcd = Lcd::new()?;
    println!("✓ Connected!\n");

    // Black/white are endian-invariant — confirm EFFECT_CUT works at all
    println!("Test 1: Black (endian-invariant)...");
    lcd.display_bitmap(&create_solid_color(0, 0, 0))?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("Test 2: White (endian-invariant)...");
    lcd.display_bitmap(&create_solid_color(255, 255, 255))?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("Test 3: Solid red (RGB check)...");
    lcd.display_bitmap(&create_solid_color(255, 0, 0))?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("Test 4: Solid green (RGB check)...");
    lcd.display_bitmap(&create_solid_color(0, 255, 0))?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("Test 5: Solid blue (RGB check)...");
    lcd.display_bitmap(&create_solid_color(0, 0, 255))?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("Test 6: Rainbow stripes (no scroll)...");
    let stripes = create_striped_pattern();
    lcd.display_bitmap(&stripes)?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("Test 7: Rainbow stripes (scroll left)...");
    lcd.display_bitmap_with_scroll(&stripes, ScrollMode::Left)?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("\nClearing...");
    lcd.clear()?;

    println!("✓ Done!");

    Ok(())
}

fn create_solid_color(r: u8, g: u8, b: u8) -> Vec<u8> {
    vec![r, g, b].repeat(640 * 150)
}

fn create_striped_pattern() -> Vec<u8> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 150;
    const STRIPE_WIDTH: usize = 80;

    let colors = [
        (255u8, 0u8, 0u8),
        (255, 165, 0),
        (255, 255, 0),
        (0, 255, 0),
        (0, 127, 255),
        (75, 0, 130),
        (148, 0, 211),
        (255, 255, 255),
    ];

    let mut bitmap = Vec::with_capacity(WIDTH * HEIGHT * 3);
    for _y in 0..HEIGHT {
        for x in 0..WIDTH {
            let (r, g, b) = colors[(x / STRIPE_WIDTH).min(7)];
            bitmap.push(r);
            bitmap.push(g);
            bitmap.push(b);
        }
    }
    bitmap
}
