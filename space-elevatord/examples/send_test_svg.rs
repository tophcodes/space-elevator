//! Manual smoke-test client: send a test SVG to a running space-elevatord over
//! its unix socket and print the response. Use the SAME XDG_RUNTIME_DIR as the
//! daemon (the socket lives at $XDG_RUNTIME_DIR/space-elevator.sock).

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn socket_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").expect("set XDG_RUNTIME_DIR to match the daemon");
    PathBuf::from(dir).join("space-elevator.sock")
}

fn main() -> std::io::Result<()> {
    // Single-quoted SVG attributes so the JSON string needs no escaping.
    // Three colour bars (red/green/blue) double as a BGR-order check through
    // the daemon's SVG path.
    let svg = "<svg xmlns='http://www.w3.org/2000/svg' width='640' height='150'>\
        <rect width='640' height='150' fill='#102030'/>\
        <rect x='20' y='25' width='180' height='100' fill='#e00000'/>\
        <rect x='230' y='25' width='180' height='100' fill='#00c000'/>\
        <rect x='440' y='25' width='180' height='100' fill='#1060ff'/>\
        <text x='320' y='98' font-size='34' fill='white' text-anchor='middle' \
        font-family='sans-serif'>SPACE ELEVATOR</text></svg>";
    let req = format!("{{\"v\":1,\"id\":1,\"cmd\":\"lcd_display_svg\",\"svg\":\"{svg}\"}}\n");

    let path = socket_path();
    println!("connecting to {}", path.display());
    let mut s = UnixStream::connect(&path)?;
    s.write_all(req.as_bytes())?;

    let mut buf = [0u8; 8192];
    let n = s.read(&mut buf)?;
    println!("response: {}", String::from_utf8_lossy(&buf[..n]).trim());
    Ok(())
}
