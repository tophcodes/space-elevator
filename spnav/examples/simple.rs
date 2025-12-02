// examples/simple.rs
use spnav::{Event, SpaceNav};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to spacenavd...");
    let mut spacenav = SpaceNav::connect()?;

    println!("Connected! Move your SpaceMouse or press buttons.");
    println!("Press Ctrl+C to exit.\n");

    loop {
        match spacenav.wait_event()? {
            Event::Motion(m) => {
                if !m.is_zero() {
                    println!(
                        "Motion: trans=[{:4}, {:4}, {:4}] rot=[{:4}, {:4}, {:4}]",
                        m.x, m.y, m.z, m.rx, m.ry, m.rz
                    );
                }
            }
            Event::Button(b) => {
                println!(
                    "Button {}: {}",
                    b.button,
                    if b.pressed { "pressed" } else { "released" }
                );
            }
        }
    }
}
