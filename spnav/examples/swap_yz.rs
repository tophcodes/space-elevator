// examples/swap_yz.rs

use spnav::SpaceNav;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut spacenav = SpaceNav::connect()?;
    let mut cfg = spacenav.config();

    // Check current state
    let is_swapped = cfg.get_swap_yz()?;
    println!("Y-Z axes currently swapped: {}", is_swapped);

    // Toggle the swap
    cfg.set_swap_yz(!is_swapped)?;
    println!("Y-Z axes swap toggled to: {}", !is_swapped);

    // Test with events
    println!("\nMove your SpaceMouse to test the new axis configuration:");
    println!("(Press Ctrl+C to exit)\n");

    loop {
        let event = spacenav.wait_event()?;
        if let spnav::Event::Motion(m) = event {
            if !m.is_zero() {
                println!("Motion: Y={:4}, Z={:4}", m.y, m.z);
            }
        }
    }
}
