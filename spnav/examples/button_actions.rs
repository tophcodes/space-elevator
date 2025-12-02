// examples/button_actions.rs

use spnav::{Event, SpaceNav, config::ButtonAction};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut spacenav = SpaceNav::connect()?;

    // Configure button actions
    {
        let mut cfg = spacenav.config();

        // Left button (0) - disable rotation for pure panning
        cfg.set_button_action(0, ButtonAction::DisableRotation)?;
        println!("Button 0: Disable rotation (for panning)");

        // Right button (1) - disable translation for pure rotation
        cfg.set_button_action(1, ButtonAction::DisableTranslation)?;
        println!("Button 1: Disable translation (for rotation only)");

        // Menu button (2) - reset sensitivity
        cfg.set_button_action(2, ButtonAction::SensitivityReset)?;
        println!("Button 2: Reset sensitivity");

        // Optional: save configuration
        // cfg.save()?;
    }

    println!("\nTry using the buttons while moving the SpaceMouse:");
    println!("- Button 0: Only translation (no rotation)");
    println!("- Button 1: Only rotation (no translation)");
    println!("- Button 2: Reset to normal sensitivity\n");

    // Event loop to show it working
    loop {
        match spacenav.wait_event()? {
            Event::Motion(m) if !m.is_zero() => {
                println!(
                    "Motion: T=[{:4}, {:4}, {:4}] R=[{:4}, {:4}, {:4}]",
                    m.x, m.y, m.z, m.rx, m.ry, m.rz
                );
            }
            Event::Button(b) => {
                println!(
                    "Button {}: {}",
                    b.button,
                    if b.pressed { "pressed" } else { "released" }
                );
            }
            _ => {}
        }
    }
}
