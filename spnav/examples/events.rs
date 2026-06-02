//! Read motion (6-DOF axes) and button events from the SpaceMouse.
//!
//! Requires spacenavd to be running (libspnav is a client of the daemon).
//! Run: `cargo run --example events` — then move the puck and press buttons.

use spnav::{Event, SpaceNav};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to spacenavd...");
    let mut device = SpaceNav::connect()?;
    println!("✓ Connected. Move the puck / press buttons (Ctrl-C to quit).\n");

    loop {
        match device.wait_event()? {
            Event::Motion(m) => {
                // Skip the zero event spacenavd emits when motion settles.
                if m.is_zero() {
                    continue;
                }
                println!(
                    "motion  T[{:>6} {:>6} {:>6}]  R[{:>6} {:>6} {:>6}]",
                    m.x, m.y, m.z, m.rx, m.ry, m.rz
                );
            }
            Event::Button(b) => {
                println!(
                    "button  #{:<2} {}",
                    b.button,
                    if b.pressed { "press" } else { "release" }
                );
            }
        }
    }
}
