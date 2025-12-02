// examples/config.rs

use spnav::{
    config::{DeviceAxis, InputAxis, LedState},
    SpaceNav,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut spacenav = SpaceNav::connect()?;

    // Temporary configuration changes (not saved)
    {
        let mut cfg = spacenav.config();

        // Set deadzones
        cfg.set_deadzone(DeviceAxis::TranslationX, 10)?;
        cfg.set_deadzone(DeviceAxis::TranslationY, 10)?;

        // Set sensitivities
        cfg.set_input_axis_sensitivity(InputAxis::TX, 2.0)?;

        // Control LED
        cfg.set_led(LedState::Off)?;

        println!("Configuration applied (temporary)");
    }

    // Use the device with new settings
    let event = spacenav.wait_event()?;
    println!("Got event: {:?}", event);

    // Restore defaults
    {
        let mut cfg = spacenav.config();
        cfg.restore()?;
        println!("Configuration restored from file");
    }

    Ok(())
}
