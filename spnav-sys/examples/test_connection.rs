use spnav_sys::*;

fn main() {
    unsafe {
        println!("Attempting to connect to spacenavd...");

        if spnav_open() == -1 {
            eprintln!("Failed to connect. Is spacenavd running?");
            std::process::exit(1);
        }

        println!("Connected! Waiting for events (Ctrl+C to exit)...");

        loop {
            let mut event: spnav_event = std::mem::zeroed();
            if spnav_wait_event(&mut event) != 0 {
                match event.type_ as u32 {
                    SPNAV_EVENT_MOTION => {
                        let motion = event.motion;
                        println!(
                            "Motion: x={} y={} z={} rx={} ry={} rz={}",
                            motion.x, motion.y, motion.z, motion.rx, motion.ry, motion.rz
                        );
                    }
                    SPNAV_EVENT_BUTTON => {
                        let button = event.button;
                        println!(
                            "Button {}: {}",
                            button.bnum,
                            if button.press != 0 {
                                "pressed"
                            } else {
                                "released"
                            }
                        );
                    }
                    _ => println!("Unknown event type: {}", event.type_),
                }
            }
        }
    };
}
