//! Manual smoke-test client: push an `lcd_set_state` to a running
//! space-elevatord (the daemon renders the SVG from the template and drives the
//! LCD). Use the SAME XDG_RUNTIME_DIR as the daemon. Mirrors the template's
//! DEMO config (FreeCAD / Sketch, Signal theme).

use space_elevatord::ipc::{Request, RequestPayload, PROTOCOL_VERSION};
use space_elevatord::lcd_template::{LcdState, Theme, Tile};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn socket_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").expect("set XDG_RUNTIME_DIR to match the daemon");
    PathBuf::from(dir).join("space-elevator.sock")
}

fn tile(label: &str, icon: &str, cat: &str, active: bool) -> Tile {
    Tile {
        label: label.into(),
        icon: Some(icon.into()),
        cat: Some(cat.into()),
        active,
    }
}

fn main() -> std::io::Result<()> {
    let state = LcdState {
        theme: Theme::Signal,
        profile: "FreeCAD".into(),
        mode: "Sketch".into(),
        left: vec![
            tile("Line", "line", "draw", false),
            tile("Rectangle", "rectangle", "draw", false),
            tile("Circle", "circle", "draw", false),
            tile("Arc", "arc", "draw", false),
            tile("Trim", "trim", "modify", false),
            tile("Construction", "construction", "modify", true),
        ],
        right: vec![
            tile("Coincident", "coincident", "constrain", false),
            tile("Horizontal", "horizontal", "constrain", false),
            tile("Vertical", "vertical", "constrain", false),
            tile("Parallel", "parallel", "constrain", false),
            tile("Equal", "equal", "constrain", false),
            tile("Dimension", "dimension", "view", false),
        ],
    };

    let req = Request {
        v: PROTOCOL_VERSION,
        id: 1,
        payload: RequestPayload::LcdSetState(state),
    };
    let line = serde_json::to_string(&req).expect("serialise request") + "\n";

    let path = socket_path();
    println!("connecting to {}", path.display());
    let mut s = UnixStream::connect(&path)?;
    s.write_all(line.as_bytes())?;

    let mut buf = [0u8; 8192];
    let n = s.read(&mut buf)?;
    println!("response: {}", String::from_utf8_lossy(&buf[..n]).trim());
    Ok(())
}
