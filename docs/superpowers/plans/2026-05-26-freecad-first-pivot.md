# FreeCAD-first Pivot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pivot Space Elevator from a Tauri+Vue GUI with an Onshape browser extension to a FreeCAD-native Python addon backed by an optional Rust LCD daemon.

**Architecture:** Two artifacts. (1) `space-elevatord`: small Rust binary that owns the SpaceMouse Enterprise LCD interface and exposes JSON-over-Unix-socket IPC. (2) `space-elevator` FreeCAD workbench: pure Python addon that reads motion + buttons from `spacenavd` via `libspnav` (ctypes + `QSocketNotifier`), drives view navigation, dispatches button-to-command bindings, and optionally pushes SVG-rendered button labels to the daemon over the socket. Frontend Vue app and Tauri shell are deleted.

**Tech Stack:** Rust (existing `spnav-sys`, `spnav` crates + new `space-elevatord` binary), serde + serde_json (IPC), tokio (async socket server). Python 3 (FreeCAD's bundled interpreter), ctypes (libspnav bindings), PySide / Qt (`QSocketNotifier`, Preferences UI), FreeCAD `App.ParamGet` (config storage). FreeCAD Addon Manager `package.xml` for distribution.

**Scope cuts vs. spec (deferred to future plans):**
- Single navigation preset only (FreeCAD-default, with axis tuning). Onshape/Fusion presets deferred.
- Single global button-binding map. Per-workbench bindings deferred.
- Daemon exposes low-level LCD primitives only (`display_image`, `display_svg`, `clear`). Higher-level `set_buttons` / `set_status` semantics live in the addon, which renders SVG templates and sends them down.

---

## Phase 0 — Cleanup & workspace restructure

Goal: leave the repo in a clean state with a Cargo workspace, no Vue/Tauri leftovers, no half-staged deletions.

### Task 0.1: Confirm current repo state and clean staging

**Files:**
- Inspect: working tree

- [ ] **Step 1: Inspect status**

Run: `git status`
Expected: shows previously-staged deletions of `space-elevator/src-tauri/**` already in HEAD (from the spec commit `80b63bb`), plus working-tree changes in `spnav/` and `space-elevator/src/App.vue`.

- [ ] **Step 2: Restore the working-tree changes that are not part of this pivot**

Identify any uncommitted edits in `spnav/` or elsewhere that are unrelated experiments (e.g., `examples/lcd_*.rs` left from LCD reverse-engineering). Leave them alone if they are intentional — they are not blocking. Just be aware.

Run: `git diff spnav/`
Read the diff. If it represents in-progress LCD experimentation that the user wants kept, do nothing. If it is accidental, ask before discarding.

- [ ] **Step 3: Commit**

No commit needed — this task is inspection only.

### Task 0.2: Delete the Vue frontend

**Files:**
- Delete: `space-elevator/` (entire directory)

- [ ] **Step 1: Remove the directory**

Run: `git rm -rf space-elevator/`
Expected: removes `package.json`, `bun.lock`, `vite.config.ts`, `index.html`, `public/`, `src/`, `tsconfig*.json`, `README.md` inside that subdir, plus any tracked `node_modules` entries.

- [ ] **Step 2: Confirm**

Run: `ls space-elevator 2>&1 || echo absent`
Expected: `absent` (or `ls: cannot access 'space-elevator': No such file or directory`).

- [ ] **Step 3: Commit**

```bash
git commit -m "chore: remove Vue frontend (pivoting to FreeCAD addon)"
```

### Task 0.3: Establish Cargo workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Modify: `spnav-sys/Cargo.toml`, `spnav/Cargo.toml` (drop redundant duplicate keys if any; verify members resolve)

- [ ] **Step 1: Write workspace `Cargo.toml`**

Create `/home/toph/Projects/space-elevator/Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "spnav-sys",
    "spnav",
    "space-elevatord",
]

[workspace.package]
edition = "2021"
authors = ["Christopher Mühl <christopher@muehl.dev>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/padarom/space-elevator"

[workspace.dependencies]
spnav-sys = { version = "0.1", path = "spnav-sys" }
spnav = { version = "0.1", path = "spnav" }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["rt", "macros", "net", "io-util", "signal", "sync"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

- [ ] **Step 2: Verify the workspace resolves**

Run: `cargo metadata --no-deps --format-version 1 > /dev/null`
Expected: exit code 0. If a member's `Cargo.toml` declares `package.workspace = true`-style inheritance keys that conflict, fix them — but do not change `spnav` / `spnav-sys` semver fields yet.

- [ ] **Step 3: Build the existing crates from the workspace root**

Run: `cargo build -p spnav -p spnav-sys`
Expected: builds clean. If `space-elevatord` is missing it has not been created yet — that is fine; do not include it in `-p` flags here.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: establish Cargo workspace"
```

### Task 0.4: Create top-level layout for new components

**Files:**
- Create: `space-elevatord/` (empty dir placeholder, populated in Phase 1)
- Create: `freecad-addon/` (empty dir placeholder, populated in Phase 3)
- Modify: `README.md`

- [ ] **Step 1: Create placeholder dirs**

Run: `mkdir -p space-elevatord/src freecad-addon`
(Don't `git add` empty dirs; the next tasks will populate them.)

- [ ] **Step 2: Rewrite top-level README**

Replace `/home/toph/Projects/space-elevator/README.md` with:

```markdown
# Space Elevator

Space Elevator is a FreeCAD addon for 6-degree-of-freedom (6DOF) input devices — primarily 3Dconnexion SpaceMice — backed by an optional Rust daemon that drives the LCD on the SpaceMouse Enterprise.

## Components

- `spnav-sys/`, `spnav/` — Rust bindings for libspnav (used by the daemon; published as standalone crates).
- `space-elevatord/` — Rust daemon. Owns the SpaceMouse Enterprise LCD. Exposes a Unix-socket JSON IPC.
- `freecad-addon/` — Python FreeCAD workbench. Reads motion and button events from spacenavd via libspnav directly. Drives view navigation. Optionally pushes SVG-rendered images to the daemon for LCD output.

## Supported hardware

- 3Dconnexion SpaceMouse Enterprise (motion + buttons + LCD)

Other SpaceMice work for motion and buttons; the LCD path is Enterprise-only.

## Supported applications

- FreeCAD 1.0+

The previous Onshape browser-extension target has been dropped from active scope.
```

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: rewrite README for FreeCAD-first scope"
```

---

## Phase 1 — `space-elevatord` daemon skeleton

Goal: a Rust binary that listens on a Unix socket, accepts JSON-line commands, dispatches them. No LCD writes yet — just IPC plumbing. After this phase you can `echo '{"v":1,"id":1,"cmd":"ping"}' | nc -U /run/user/$UID/space-elevator.sock` and get `{"v":1,"id":1,"ok":true}`.

### Task 1.1: Create the daemon crate

**Files:**
- Create: `space-elevatord/Cargo.toml`
- Create: `space-elevatord/src/main.rs`

- [ ] **Step 1: Write `space-elevatord/Cargo.toml`**

```toml
[package]
name = "space-elevatord"
version = "0.1.0"
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Daemon for SpaceMouse Enterprise LCD output, paired with the space-elevator FreeCAD addon"

[dependencies]
spnav = { workspace = true, features = ["lcd", "svg"] }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

- [ ] **Step 2: Write a minimal `main.rs` stub**

`space-elevatord/src/main.rs`:

```rust
fn main() {
    eprintln!("space-elevatord placeholder");
}
```

- [ ] **Step 3: Verify it builds**

Run: `cargo build -p space-elevatord`
Expected: builds clean.

- [ ] **Step 4: Commit**

```bash
git add space-elevatord/Cargo.toml space-elevatord/src/main.rs
git commit -m "feat(daemon): scaffold space-elevatord crate"
```

### Task 1.2: Define the IPC protocol types

**Files:**
- Create: `space-elevatord/src/ipc.rs`
- Modify: `space-elevatord/src/main.rs` (declare `mod ipc;`)

- [ ] **Step 1: Write failing test**

Add to `space-elevatord/src/ipc.rs`:

```rust
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Request {
    pub v: u32,
    pub id: u64,
    #[serde(flatten)]
    pub payload: RequestPayload,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum RequestPayload {
    Ping,
    LcdClear,
    LcdDisplayImage { path: String },
    LcdDisplaySvg { svg: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub v: u32,
    pub id: u64,
    #[serde(flatten)]
    pub result: ResponseResult,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResponseResult {
    Ok { ok: bool },
    Err { ok: bool, error: String },
}

impl Response {
    pub fn ok(id: u64) -> Self {
        Self { v: PROTOCOL_VERSION, id, result: ResponseResult::Ok { ok: true } }
    }
    pub fn err(id: u64, error: impl Into<String>) -> Self {
        Self { v: PROTOCOL_VERSION, id, result: ResponseResult::Err { ok: false, error: error.into() } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ping_request() {
        let raw = r#"{"v":1,"id":42,"cmd":"ping"}"#;
        let parsed: Request = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed, Request { v: 1, id: 42, payload: RequestPayload::Ping });
    }

    #[test]
    fn parses_lcd_display_image_request() {
        let raw = r#"{"v":1,"id":7,"cmd":"lcd_display_image","path":"/tmp/img.png"}"#;
        let parsed: Request = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.payload, RequestPayload::LcdDisplayImage { path: "/tmp/img.png".into() });
    }

    #[test]
    fn serialises_ok_response() {
        let r = Response::ok(99);
        let s = serde_json::to_string(&r).unwrap();
        assert_eq!(s, r#"{"v":1,"id":99,"ok":true}"#);
    }

    #[test]
    fn serialises_err_response() {
        let r = Response::err(99, "boom");
        let s = serde_json::to_string(&r).unwrap();
        assert_eq!(s, r#"{"v":1,"id":99,"ok":false,"error":"boom"}"#);
    }
}
```

Modify `space-elevatord/src/main.rs`:

```rust
mod ipc;

fn main() {
    eprintln!("space-elevatord placeholder");
}
```

- [ ] **Step 2: Run tests, confirm they pass**

Run: `cargo test -p space-elevatord --lib`
Expected: 4 tests pass. (They were authored against the same module; this is the implementation step in one shot — acceptable for pure serde-derive code where the test IS the spec.)

- [ ] **Step 3: Commit**

```bash
git add space-elevatord/src/ipc.rs space-elevatord/src/main.rs
git commit -m "feat(daemon): add IPC protocol types"
```

### Task 1.3: Implement the socket server (no LCD yet)

**Files:**
- Create: `space-elevatord/src/server.rs`
- Modify: `space-elevatord/src/main.rs`

- [ ] **Step 1: Write `server.rs`**

```rust
use crate::ipc::{Request, RequestPayload, Response};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

pub fn socket_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/tmp/space-elevator-{}", whoami_uid()));
    PathBuf::from(dir).join("space-elevator.sock")
}

fn whoami_uid() -> u32 {
    // Avoid pulling a crate for this; getuid is libc but ubiquitous.
    unsafe { libc_getuid() }
}

extern "C" {
    #[link_name = "getuid"]
    fn libc_getuid() -> u32;
}

pub async fn run() -> std::io::Result<()> {
    let path = socket_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let listener = UnixListener::bind(&path)?;
    info!(?path, "space-elevatord listening");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                warn!("client closed with error: {e}");
            }
        });
    }
}

async fn handle_client(stream: UnixStream) -> std::io::Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half).lines();

    while let Some(line) = reader.next_line().await? {
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => dispatch(req).await,
            Err(e) => Response::err(0, format!("invalid request: {e}")),
        };
        let mut buf = serde_json::to_vec(&response).expect("serialise");
        buf.push(b'\n');
        write_half.write_all(&buf).await?;
    }
    Ok(())
}

async fn dispatch(req: Request) -> Response {
    if req.v != crate::ipc::PROTOCOL_VERSION {
        return Response::err(req.id, format!("unsupported protocol version {}", req.v));
    }
    match req.payload {
        RequestPayload::Ping => Response::ok(req.id),
        RequestPayload::LcdClear
        | RequestPayload::LcdDisplayImage { .. }
        | RequestPayload::LcdDisplaySvg { .. } => {
            // LCD wiring lands in Phase 2.
            error!("lcd commands not yet implemented");
            Response::err(req.id, "lcd not implemented")
        }
    }
}
```

Modify `space-elevatord/src/main.rs`:

```rust
mod ipc;
mod server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("space_elevatord=info".parse().unwrap()))
        .init();
    server::run().await
}
```

- [ ] **Step 2: Build**

Run: `cargo build -p space-elevatord`
Expected: builds clean.

- [ ] **Step 3: Manual smoke test**

In one terminal:
```bash
cargo run -p space-elevatord
```
Expected: log line `space-elevatord listening` with the socket path.

In a second terminal:
```bash
printf '{"v":1,"id":1,"cmd":"ping"}\n' | nc -U "$XDG_RUNTIME_DIR/space-elevator.sock"
```
Expected output: `{"v":1,"id":1,"ok":true}`

Also test rejection:
```bash
printf '{"v":1,"id":2,"cmd":"lcd_clear"}\n' | nc -U "$XDG_RUNTIME_DIR/space-elevator.sock"
```
Expected: `{"v":1,"id":2,"ok":false,"error":"lcd not implemented"}`

Stop the daemon with Ctrl+C.

- [ ] **Step 4: Commit**

```bash
git add space-elevatord/src/server.rs space-elevatord/src/main.rs
git commit -m "feat(daemon): unix socket server with JSON line protocol"
```

---

## Phase 2 — LCD wiring in the daemon

Goal: `lcd_clear`, `lcd_display_image`, `lcd_display_svg` actually drive the SpaceMouse Enterprise LCD. Requires the physical device — manual hardware test gates the phase.

**Important:** the existing `spnav::lcd` code still has TODOs around the packet header (see `spnav/src/lcd.rs:96-110`). This phase does NOT attempt to finish that reverse-engineering — it only wires the existing primitives behind the IPC. If `Lcd::display_bitmap` does not work end-to-end on real hardware, fix forward in the daemon by patching `spnav::lcd` directly; do not invent a new code path.

### Task 2.1: Wrap `spnav::lcd` behind a single-owner handle

**Files:**
- Create: `space-elevatord/src/lcd_handle.rs`
- Modify: `space-elevatord/src/main.rs`

- [ ] **Step 1: Write the handle**

`space-elevatord/src/lcd_handle.rs`:

```rust
use spnav::lcd::{Lcd, LcdError};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct LcdHandle {
    inner: Arc<Mutex<Option<Lcd>>>,
}

impl LcdHandle {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(None)) }
    }

    async fn ensure_open(&self) -> Result<tokio::sync::MutexGuard<'_, Option<Lcd>>, LcdError> {
        let mut guard = self.inner.lock().await;
        if guard.is_none() {
            *guard = Some(Lcd::new()?);
        }
        Ok(guard)
    }

    pub async fn clear(&self) -> Result<(), LcdError> {
        let mut g = self.ensure_open().await?;
        g.as_mut().unwrap().clear()
    }

    pub async fn display_rgb888(&self, rgb: &[u8]) -> Result<(), LcdError> {
        let mut g = self.ensure_open().await?;
        g.as_mut().unwrap().display_bitmap(rgb)
    }
}
```

Modify `space-elevatord/src/main.rs` to declare `mod lcd_handle;`.

- [ ] **Step 2: Build**

Run: `cargo build -p space-elevatord`
Expected: builds clean.

- [ ] **Step 3: Commit**

```bash
git add space-elevatord/src/lcd_handle.rs space-elevatord/src/main.rs
git commit -m "feat(daemon): LCD handle with lazy device open"
```

### Task 2.2: Implement `lcd_clear` and `lcd_display_image` dispatch

**Files:**
- Modify: `space-elevatord/src/server.rs`
- Modify: `space-elevatord/Cargo.toml` (add `image = "0.25"` for PNG/JPEG decoding)

- [ ] **Step 1: Add the image crate dep**

In `space-elevatord/Cargo.toml`, append to `[dependencies]`:
```toml
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
```

- [ ] **Step 2: Wire dispatch to the LCD handle**

Refactor `server.rs` so the per-client handler holds a clone of an `LcdHandle`. Change `run()`:

```rust
pub async fn run() -> std::io::Result<()> {
    let path = socket_path();
    if path.exists() { std::fs::remove_file(&path)?; }
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let listener = UnixListener::bind(&path)?;
    info!(?path, "space-elevatord listening");

    let lcd = crate::lcd_handle::LcdHandle::new();

    loop {
        let (stream, _) = listener.accept().await?;
        let lcd = lcd.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, lcd).await {
                warn!("client closed with error: {e}");
            }
        });
    }
}
```

Change `handle_client` signature to `async fn handle_client(stream: UnixStream, lcd: crate::lcd_handle::LcdHandle)` and pass `lcd` into `dispatch`.

Replace `dispatch` body:

```rust
async fn dispatch(req: Request, lcd: &crate::lcd_handle::LcdHandle) -> Response {
    if req.v != crate::ipc::PROTOCOL_VERSION {
        return Response::err(req.id, format!("unsupported protocol version {}", req.v));
    }
    match req.payload {
        RequestPayload::Ping => Response::ok(req.id),
        RequestPayload::LcdClear => match lcd.clear().await {
            Ok(()) => Response::ok(req.id),
            Err(e) => Response::err(req.id, e.to_string()),
        },
        RequestPayload::LcdDisplayImage { path } => match load_rgb888(&path) {
            Ok(rgb) => match lcd.display_rgb888(&rgb).await {
                Ok(()) => Response::ok(req.id),
                Err(e) => Response::err(req.id, e.to_string()),
            },
            Err(e) => Response::err(req.id, e),
        },
        RequestPayload::LcdDisplaySvg { .. } => Response::err(req.id, "svg not implemented yet"),
    }
}

fn load_rgb888(path: &str) -> Result<Vec<u8>, String> {
    let img = image::open(path).map_err(|e| format!("image open: {e}"))?;
    let rgb = img.into_rgb8();
    if rgb.width() != 640 || rgb.height() != 150 {
        return Err(format!(
            "image must be 640x150, got {}x{}", rgb.width(), rgb.height()
        ));
    }
    Ok(rgb.into_raw())
}
```

- [ ] **Step 3: Build**

Run: `cargo build -p space-elevatord`
Expected: builds clean.

- [ ] **Step 4: Manual hardware test**

Prepare a 640x150 RGB PNG at `/tmp/test.png` (any image cropped/resized to that size, e.g.: `convert -resize 640x150! input.jpg /tmp/test.png`).

In one terminal: `cargo run -p space-elevatord`
In another:
```bash
printf '{"v":1,"id":1,"cmd":"lcd_display_image","path":"/tmp/test.png"}\n' | nc -U "$XDG_RUNTIME_DIR/space-elevator.sock"
printf '{"v":1,"id":2,"cmd":"lcd_clear"}\n' | nc -U "$XDG_RUNTIME_DIR/space-elevator.sock"
```
Expected: image appears on the SpaceMouse Enterprise LCD, then clears. Each command returns `{"v":1,"id":N,"ok":true}`.

If LCD shows garbage or nothing: the `spnav::lcd` code has known TODOs. File a `TODO` issue noting which packet bytes are wrong; **do not** fix it inside the daemon — fix in `spnav/src/lcd.rs` so the standalone crate stays useful.

- [ ] **Step 5: Commit**

```bash
git add space-elevatord/Cargo.toml space-elevatord/src/server.rs
git commit -m "feat(daemon): dispatch lcd_clear and lcd_display_image"
```

### Task 2.3: SVG rendering via `resvg`

**Files:**
- Modify: `space-elevatord/Cargo.toml` (add `resvg`, `tiny-skia`)
- Modify: `space-elevatord/src/server.rs`
- Create: `space-elevatord/src/svg_render.rs`

- [ ] **Step 1: Add deps**

In `space-elevatord/Cargo.toml`:
```toml
resvg = "0.45"
tiny-skia = "0.11"
usvg = "0.45"
```

- [ ] **Step 2: Write the renderer**

`space-elevatord/src/svg_render.rs`:

```rust
use resvg::tiny_skia;
use usvg::Tree;

pub const LCD_WIDTH: u32 = 640;
pub const LCD_HEIGHT: u32 = 150;

pub fn render_to_rgb888(svg: &str) -> Result<Vec<u8>, String> {
    let opt = usvg::Options::default();
    let tree = Tree::from_str(svg, &opt).map_err(|e| format!("svg parse: {e}"))?;

    let mut pixmap = tiny_skia::Pixmap::new(LCD_WIDTH, LCD_HEIGHT)
        .ok_or("pixmap alloc")?;

    let view_w = tree.size().width();
    let view_h = tree.size().height();
    let scale_x = LCD_WIDTH as f32 / view_w;
    let scale_y = LCD_HEIGHT as f32 / view_h;
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut rgb = Vec::with_capacity((LCD_WIDTH * LCD_HEIGHT * 3) as usize);
    for px in pixmap.pixels() {
        rgb.push(px.red());
        rgb.push(px.green());
        rgb.push(px.blue());
    }
    Ok(rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_solid_rectangle() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 150"><rect width="640" height="150" fill="#ff0000"/></svg>"##;
        let rgb = render_to_rgb888(svg).unwrap();
        assert_eq!(rgb.len(), (640 * 150 * 3) as usize);
        // First pixel red
        assert!(rgb[0] > 200 && rgb[1] < 50 && rgb[2] < 50);
    }
}
```

Add `mod svg_render;` to `space-elevatord/src/main.rs`.

- [ ] **Step 3: Wire into dispatch**

Replace the `LcdDisplaySvg` arm in `server.rs::dispatch`:

```rust
RequestPayload::LcdDisplaySvg { svg } => match crate::svg_render::render_to_rgb888(&svg) {
    Ok(rgb) => match lcd.display_rgb888(&rgb).await {
        Ok(()) => Response::ok(req.id),
        Err(e) => Response::err(req.id, e.to_string()),
    },
    Err(e) => Response::err(req.id, e),
},
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p space-elevatord`
Expected: all tests pass including `renders_solid_rectangle`.

- [ ] **Step 5: Manual hardware test**

```bash
SVG='{"v":1,"id":1,"cmd":"lcd_display_svg","svg":"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 640 150\"><rect width=\"640\" height=\"150\" fill=\"#003366\"/><text x=\"320\" y=\"95\" font-size=\"72\" fill=\"white\" text-anchor=\"middle\" font-family=\"sans-serif\">Hello</text></svg>"}'
printf '%s\n' "$SVG" | nc -U "$XDG_RUNTIME_DIR/space-elevator.sock"
```
Expected: dark blue background with "Hello" centred, returns `{"v":1,"id":1,"ok":true}`.

- [ ] **Step 6: Commit**

```bash
git add space-elevatord/Cargo.toml space-elevatord/src/svg_render.rs space-elevatord/src/server.rs space-elevatord/src/main.rs
git commit -m "feat(daemon): render SVG to LCD via resvg"
```

---

## Phase 3 — FreeCAD addon skeleton

Goal: an installable FreeCAD workbench that shows up in the workbench selector, prints a "hello" line on activation, and has a `package.xml` manifest the Addon Manager understands. No motion or buttons yet.

### Task 3.1: Addon manifest and InitGui

**Files:**
- Create: `freecad-addon/package.xml`
- Create: `freecad-addon/InitGui.py`
- Create: `freecad-addon/Init.py`
- Create: `freecad-addon/space_elevator/__init__.py`
- Create: `freecad-addon/Resources/icons/space-elevator.svg`

- [ ] **Step 1: Manifest**

`freecad-addon/package.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<package format="1" xmlns="https://wiki.freecad.org/Package_Metadata">
  <name>Space Elevator</name>
  <description>6DOF input device integration for FreeCAD (SpaceMouse, etc.)</description>
  <version>0.1.0</version>
  <maintainer email="christopher@muehl.dev">Christopher Mühl</maintainer>
  <license file="LICENSE">MIT</license>
  <url type="repository" branch="main">https://github.com/padarom/space-elevator</url>
  <content>
    <workbench>
      <classname>SpaceElevatorWorkbench</classname>
      <name>Space Elevator</name>
      <description>Integration for 6DOF input devices.</description>
      <icon>Resources/icons/space-elevator.svg</icon>
    </workbench>
  </content>
  <depend condition="$installed_FreeCAD &gt; '1.0'">FreeCAD</depend>
</package>
```

- [ ] **Step 2: `Init.py`**

`freecad-addon/Init.py`:

```python
# Loaded for both GUI and headless. Keep empty for now.
```

- [ ] **Step 3: `InitGui.py`**

`freecad-addon/InitGui.py`:

```python
import os
import FreeCAD
import FreeCADGui as Gui

ADDON_DIR = os.path.dirname(__file__)
ICON_PATH = os.path.join(ADDON_DIR, "Resources", "icons", "space-elevator.svg")


class SpaceElevatorWorkbench(Gui.Workbench):
    MenuText = "Space Elevator"
    ToolTip = "6DOF input device integration"
    Icon = ICON_PATH

    def Initialize(self):
        FreeCAD.Console.PrintMessage("Space Elevator: workbench initialized\n")

    def Activated(self):
        from space_elevator import lifecycle
        lifecycle.on_activate()

    def Deactivated(self):
        from space_elevator import lifecycle
        lifecycle.on_deactivate()

    def GetClassName(self):
        return "Gui::PythonWorkbench"


Gui.addWorkbench(SpaceElevatorWorkbench())
```

- [ ] **Step 4: Package init + lifecycle stub**

`freecad-addon/space_elevator/__init__.py`:
```python
__version__ = "0.1.0"
```

`freecad-addon/space_elevator/lifecycle.py`:

```python
import FreeCAD

_state = {"active": False}


def on_activate():
    if _state["active"]:
        return
    _state["active"] = True
    FreeCAD.Console.PrintMessage("Space Elevator: activated\n")


def on_deactivate():
    if not _state["active"]:
        return
    _state["active"] = False
    FreeCAD.Console.PrintMessage("Space Elevator: deactivated\n")
```

- [ ] **Step 5: Placeholder icon**

`freecad-addon/Resources/icons/space-elevator.svg`:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <rect width="64" height="64" rx="12" fill="#1a3a5a"/>
  <circle cx="32" cy="28" r="14" fill="#f0f0f0"/>
  <rect x="20" y="40" width="24" height="14" rx="3" fill="#888"/>
</svg>
```

- [ ] **Step 6: Manual smoke test**

Symlink the addon into FreeCAD's Mod directory:
```bash
mkdir -p "$HOME/.local/share/FreeCAD/Mod"
ln -sfn "$PWD/freecad-addon" "$HOME/.local/share/FreeCAD/Mod/SpaceElevator"
```
Launch FreeCAD. Expected: "Space Elevator" appears in the workbench dropdown. Selecting it logs "Space Elevator: workbench initialized" and "Space Elevator: activated" in the Report View.

- [ ] **Step 7: Commit**

```bash
git add freecad-addon/
git commit -m "feat(addon): scaffold FreeCAD workbench"
```

---

## Phase 4 — libspnav ctypes bindings + Qt event integration

Goal: when the workbench is active, motion and button events from `spacenavd` flow into Python callbacks. No view manipulation yet — just print events to the Report View to prove the plumbing.

### Task 4.1: ctypes wrapper for libspnav

**Files:**
- Create: `freecad-addon/space_elevator/libspnav.py`
- Create: `freecad-addon/tests/test_libspnav_types.py`

- [ ] **Step 1: Write the wrapper**

`freecad-addon/space_elevator/libspnav.py`:

```python
"""ctypes wrapper for libspnav.

Exposes only what the addon needs: open/close, the file descriptor, and
non-blocking event polling. Magellan/X11 entry points are intentionally absent
- spacenavd recommends spnav_open() over them.
"""

import ctypes
import ctypes.util

SPNAV_EVENT_MOTION = 1
SPNAV_EVENT_BUTTON = 2


class _MotionEvent(ctypes.Structure):
    _fields_ = [
        ("type", ctypes.c_int),
        ("x", ctypes.c_int),
        ("y", ctypes.c_int),
        ("z", ctypes.c_int),
        ("rx", ctypes.c_int),
        ("ry", ctypes.c_int),
        ("rz", ctypes.c_int),
        ("period", ctypes.c_uint),
        ("data", ctypes.POINTER(ctypes.c_int)),
    ]


class _ButtonEvent(ctypes.Structure):
    _fields_ = [
        ("type", ctypes.c_int),
        ("press", ctypes.c_int),
        ("bnum", ctypes.c_int),
    ]


class _Event(ctypes.Union):
    _fields_ = [
        ("type", ctypes.c_int),
        ("motion", _MotionEvent),
        ("button", _ButtonEvent),
    ]


class LibspnavMissing(RuntimeError):
    pass


class SpacenavdUnavailable(RuntimeError):
    pass


class _Lib:
    def __init__(self):
        path = ctypes.util.find_library("spnav")
        if not path:
            raise LibspnavMissing("libspnav not found; install spacenavd's client lib")
        lib = ctypes.CDLL(path)

        lib.spnav_open.argtypes = []
        lib.spnav_open.restype = ctypes.c_int
        lib.spnav_close.argtypes = []
        lib.spnav_close.restype = ctypes.c_int
        lib.spnav_fd.argtypes = []
        lib.spnav_fd.restype = ctypes.c_int
        lib.spnav_poll_event.argtypes = [ctypes.POINTER(_Event)]
        lib.spnav_poll_event.restype = ctypes.c_int
        self._lib = lib

    def open(self):
        if self._lib.spnav_open() == -1:
            raise SpacenavdUnavailable("spnav_open failed; is spacenavd running?")

    def close(self):
        self._lib.spnav_close()

    def fd(self):
        return self._lib.spnav_fd()

    def poll(self):
        ev = _Event()
        if self._lib.spnav_poll_event(ctypes.byref(ev)) == 0:
            return None
        if ev.type == SPNAV_EVENT_MOTION:
            m = ev.motion
            return ("motion", (m.x, m.y, m.z, m.rx, m.ry, m.rz), m.period)
        if ev.type == SPNAV_EVENT_BUTTON:
            b = ev.button
            return ("button", b.bnum, bool(b.press))
        return None


_singleton = None


def get():
    global _singleton
    if _singleton is None:
        _singleton = _Lib()
    return _singleton
```

- [ ] **Step 2: Write the pure-Python tests**

`freecad-addon/tests/test_libspnav_types.py`:

```python
import ctypes
import pytest

from space_elevator.libspnav import _Event, _MotionEvent, _ButtonEvent, SPNAV_EVENT_MOTION, SPNAV_EVENT_BUTTON


def test_event_union_size_matches_motion_struct():
    # Union must be at least as large as the largest member.
    assert ctypes.sizeof(_Event) >= ctypes.sizeof(_MotionEvent)
    assert ctypes.sizeof(_Event) >= ctypes.sizeof(_ButtonEvent)


def test_event_type_field_alias():
    ev = _Event()
    ev.motion.type = SPNAV_EVENT_MOTION
    assert ev.type == SPNAV_EVENT_MOTION
    ev.button.type = SPNAV_EVENT_BUTTON
    assert ev.type == SPNAV_EVENT_BUTTON


def test_motion_struct_field_order():
    m = _MotionEvent(type=1, x=10, y=-20, z=30, rx=1, ry=2, rz=3, period=16, data=None)
    assert m.x == 10 and m.y == -20 and m.z == 30
```

- [ ] **Step 3: Add minimal pytest setup**

Create `freecad-addon/pyproject.toml`:

```toml
[project]
name = "space-elevator-addon"
version = "0.1.0"
requires-python = ">=3.9"

[tool.pytest.ini_options]
testpaths = ["tests"]
pythonpath = ["."]
```

- [ ] **Step 4: Run tests**

Run from `freecad-addon/`: `pytest -v`
Expected: 3 tests pass. (No FreeCAD or libspnav needed for these — they exercise only ctypes layout.)

- [ ] **Step 5: Commit**

```bash
git add freecad-addon/space_elevator/libspnav.py freecad-addon/tests/test_libspnav_types.py freecad-addon/pyproject.toml
git commit -m "feat(addon): libspnav ctypes bindings"
```

### Task 4.2: Qt event loop integration via `QSocketNotifier`

**Files:**
- Create: `freecad-addon/space_elevator/event_pump.py`
- Modify: `freecad-addon/space_elevator/lifecycle.py`

- [ ] **Step 1: Write the event pump**

`freecad-addon/space_elevator/event_pump.py`:

```python
"""Wires libspnav's fd into Qt's main loop.

Drains all pending events whenever the fd becomes readable and dispatches each
to a handler callback. The callback is invoked from Qt's thread; it can touch
FreeCAD's GUI safely.
"""

import FreeCAD

try:
    from PySide6 import QtCore
except ImportError:
    from PySide2 import QtCore  # FreeCAD 1.0 on most distros

from . import libspnav


class EventPump(QtCore.QObject):
    def __init__(self, on_event, parent=None):
        super().__init__(parent)
        self._lib = libspnav.get()
        self._on_event = on_event
        self._notifier = None

    def start(self):
        try:
            self._lib.open()
        except libspnav.SpacenavdUnavailable as e:
            FreeCAD.Console.PrintWarning(f"Space Elevator: {e}\n")
            return False
        fd = self._lib.fd()
        if fd < 0:
            FreeCAD.Console.PrintWarning("Space Elevator: no spacenavd fd\n")
            self._lib.close()
            return False
        self._notifier = QtCore.QSocketNotifier(fd, QtCore.QSocketNotifier.Read)
        self._notifier.activated.connect(self._drain)
        self._notifier.setEnabled(True)
        return True

    def stop(self):
        if self._notifier is not None:
            self._notifier.setEnabled(False)
            self._notifier.deleteLater()
            self._notifier = None
        self._lib.close()

    def _drain(self, _fd):
        # Drain everything queued — fd is edge-friendly; multiple events can
        # arrive between activations.
        while True:
            ev = self._lib.poll()
            if ev is None:
                return
            try:
                self._on_event(ev)
            except Exception as exc:
                FreeCAD.Console.PrintError(f"Space Elevator handler error: {exc}\n")
```

- [ ] **Step 2: Wire into lifecycle**

Replace `freecad-addon/space_elevator/lifecycle.py`:

```python
import FreeCAD

from .event_pump import EventPump
from . import libspnav

_state = {"active": False, "pump": None}


def _debug_handler(ev):
    kind = ev[0]
    if kind == "motion":
        _, axes, period = ev
        FreeCAD.Console.PrintMessage(f"motion {axes} period={period}\n")
    elif kind == "button":
        _, bnum, pressed = ev
        FreeCAD.Console.PrintMessage(f"button {bnum} {'press' if pressed else 'release'}\n")


def on_activate():
    if _state["active"]:
        return
    _state["active"] = True

    try:
        pump = EventPump(_debug_handler)
    except libspnav.LibspnavMissing as e:
        FreeCAD.Console.PrintError(f"Space Elevator: {e}\n")
        _state["active"] = False
        return

    if pump.start():
        _state["pump"] = pump
        FreeCAD.Console.PrintMessage("Space Elevator: event pump started\n")
    else:
        _state["active"] = False


def on_deactivate():
    if not _state["active"]:
        return
    pump = _state.pop("pump", None)
    if pump is not None:
        pump.stop()
    _state["active"] = False
    FreeCAD.Console.PrintMessage("Space Elevator: deactivated\n")
```

- [ ] **Step 3: Manual integration test**

Ensure `spacenavd` is running (`systemctl --user status spacenavd` or system unit). Restart FreeCAD. Switch to the Space Elevator workbench. Move the SpaceMouse.

Expected: lines like `motion (12, -3, 0, 0, 0, 0) period=16` and `button 0 press` / `button 0 release` appear in the Report View.

If nothing appears: verify libspnav is installed (`ldconfig -p | grep spnav`), spacenavd is running, and the user has read access to its socket (`/var/run/spnav.sock` or `/run/spnav.sock`).

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/event_pump.py freecad-addon/space_elevator/lifecycle.py
git commit -m "feat(addon): drain libspnav events through QSocketNotifier"
```

---

## Phase 5 — View navigation (default preset)

Goal: SpaceMouse motion moves the FreeCAD camera. Translation pans, twist/tilt rotates. One preset only (FreeCAD-default semantics). Per-axis scale knobs come from settings.

### Task 5.1: Settings backed by `App.ParamGet`

**Files:**
- Create: `freecad-addon/space_elevator/settings.py`
- Create: `freecad-addon/tests/test_settings.py` (with a fake ParamGet)

- [ ] **Step 1: Write settings**

`freecad-addon/space_elevator/settings.py`:

```python
"""Thin wrapper over FreeCAD's parameter system.

App.ParamGet returns a ParameterGrp; methods are
GetFloat/SetFloat/GetBool/SetBool/GetString/SetString, each with a default arg.

Settings layout (under "User parameter:BaseApp/Preferences/Mod/SpaceElevator"):

  axis_scale_tx, axis_scale_ty, axis_scale_tz   (float, default 1.0)
  axis_scale_rx, axis_scale_ry, axis_scale_rz   (float, default 1.0)
  invert_tx, invert_ty, ... (bool, default False)
  deadzone (float, default 0.02)         # 0..1 fraction of raw range
  lcd_enabled (bool, default True)
  daemon_socket_path (string, default "")  # "" => use $XDG_RUNTIME_DIR
"""

_PATH = "User parameter:BaseApp/Preferences/Mod/SpaceElevator"

_AXES = ("tx", "ty", "tz", "rx", "ry", "rz")


def _grp(param_get):
    return param_get(_PATH)


class Settings:
    def __init__(self, param_get):
        # param_get: callable taking a path string and returning a parameter group.
        self._grp = _grp(param_get)

    def axis_scale(self, axis):
        return self._grp.GetFloat(f"axis_scale_{axis}", 1.0)

    def set_axis_scale(self, axis, value):
        self._grp.SetFloat(f"axis_scale_{axis}", float(value))

    def invert(self, axis):
        return self._grp.GetBool(f"invert_{axis}", False)

    def set_invert(self, axis, value):
        self._grp.SetBool(f"invert_{axis}", bool(value))

    def deadzone(self):
        return self._grp.GetFloat("deadzone", 0.02)

    def set_deadzone(self, value):
        self._grp.SetFloat("deadzone", float(value))

    def lcd_enabled(self):
        return self._grp.GetBool("lcd_enabled", True)

    def set_lcd_enabled(self, value):
        self._grp.SetBool("lcd_enabled", bool(value))

    def daemon_socket_path(self):
        return self._grp.GetString("daemon_socket_path", "")

    def set_daemon_socket_path(self, value):
        self._grp.SetString("daemon_socket_path", str(value))


def from_freecad():
    import FreeCAD
    return Settings(FreeCAD.ParamGet)


AXES = _AXES
```

- [ ] **Step 2: Write tests with a fake ParamGet**

`freecad-addon/tests/test_settings.py`:

```python
from space_elevator.settings import Settings, AXES


class FakeGroup:
    def __init__(self):
        self._floats = {}
        self._bools = {}
        self._strings = {}

    def GetFloat(self, k, d): return self._floats.get(k, d)
    def SetFloat(self, k, v): self._floats[k] = v
    def GetBool(self, k, d): return self._bools.get(k, d)
    def SetBool(self, k, v): self._bools[k] = v
    def GetString(self, k, d): return self._strings.get(k, d)
    def SetString(self, k, v): self._strings[k] = v


def fake_param_get():
    grp = FakeGroup()
    def f(path):
        return grp
    return f


def test_axis_scale_roundtrip():
    s = Settings(fake_param_get())
    for a in AXES:
        assert s.axis_scale(a) == 1.0
        s.set_axis_scale(a, 2.5)
        assert s.axis_scale(a) == 2.5


def test_invert_roundtrip():
    s = Settings(fake_param_get())
    s.set_invert("tx", True)
    assert s.invert("tx") is True
    assert s.invert("ty") is False


def test_deadzone_default():
    s = Settings(fake_param_get())
    assert s.deadzone() == 0.02
```

- [ ] **Step 3: Run tests**

Run: `pytest -v freecad-addon/`
Expected: previous 3 + 3 new pass.

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/settings.py freecad-addon/tests/test_settings.py
git commit -m "feat(addon): settings backed by App.ParamGet"
```

### Task 5.2: Axis transform with deadzone and scale

**Files:**
- Create: `freecad-addon/space_elevator/axes.py`
- Create: `freecad-addon/tests/test_axes.py`

- [ ] **Step 1: Write failing tests**

`freecad-addon/tests/test_axes.py`:

```python
from space_elevator.axes import transform


def test_deadzone_suppresses_small_values():
    raw = (10, 10, 10, 10, 10, 10)  # well inside 0.02 * 350 ~= 7
    scales = (1.0,) * 6
    inverts = (False,) * 6
    out = transform(raw, scales=scales, inverts=inverts, deadzone=0.02, max_raw=350)
    assert out == (0.0,) * 6


def test_scale_applied_after_deadzone():
    raw = (350, 0, 0, 0, 0, 0)
    out = transform(raw, scales=(2.0, 1, 1, 1, 1, 1), inverts=(False,)*6, deadzone=0.02, max_raw=350)
    assert abs(out[0] - 2.0) < 1e-6  # normalised to 1.0, then *2


def test_invert_flips_sign():
    raw = (350, 0, 0, 0, 0, 0)
    out = transform(raw, scales=(1.0,)*6, inverts=(True, False, False, False, False, False), deadzone=0.0, max_raw=350)
    assert out[0] == -1.0
```

- [ ] **Step 2: Write `axes.py`**

`freecad-addon/space_elevator/axes.py`:

```python
def transform(raw, scales, inverts, deadzone, max_raw=350):
    """Map a 6-tuple of raw libspnav axes to scaled floats in roughly [-N, N].

    raw: tuple of six ints (x, y, z, rx, ry, rz)
    scales / inverts: 6-tuples
    deadzone: fraction of max_raw under which values clamp to 0
    max_raw: raw axis range — libspnav reports roughly +-350 for SpaceMouse Enterprise

    Returns a tuple of six floats. A value of 1.0 means "axis at full deflection".
    """
    dz = deadzone * max_raw
    out = []
    for v, s, inv in zip(raw, scales, inverts):
        if -dz < v < dz:
            out.append(0.0)
            continue
        norm = v / max_raw
        if inv:
            norm = -norm
        out.append(norm * s)
    return tuple(out)
```

- [ ] **Step 3: Run tests**

Run: `pytest -v freecad-addon/tests/test_axes.py`
Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/axes.py freecad-addon/tests/test_axes.py
git commit -m "feat(addon): axis transform with deadzone and scale"
```

### Task 5.3: Drive the active 3D view's camera

**Files:**
- Create: `freecad-addon/space_elevator/navigator.py`
- Modify: `freecad-addon/space_elevator/lifecycle.py`

- [ ] **Step 1: Write navigator**

`freecad-addon/space_elevator/navigator.py`:

```python
"""Apply transformed SpaceMouse motion to the active FreeCAD 3D view.

Strategy: translate the camera in view-local coordinates for tx/ty/tz, rotate
about the camera's local axes for rx/ry/rz. Period comes from libspnav in ms;
we use it as a per-event timestep so feel is consistent regardless of report
rate.
"""

import math
import FreeCADGui as Gui

# Tunables — exposed via Settings in a later iteration if needed.
PAN_UNITS_PER_FULL_DEFLECTION_PER_SECOND = 60.0   # mm/s at full stick
ROT_RADIANS_PER_FULL_DEFLECTION_PER_SECOND = 1.2  # rad/s at full stick


def _active_camera():
    doc = Gui.ActiveDocument
    if doc is None:
        return None
    view = doc.ActiveView
    if view is None:
        return None
    return view.getCameraNode()


def apply(transformed_axes, period_ms):
    cam = _active_camera()
    if cam is None:
        return
    tx, ty, tz, rx, ry, rz = transformed_axes
    dt = max(period_ms, 1) / 1000.0

    pan_factor = PAN_UNITS_PER_FULL_DEFLECTION_PER_SECOND * dt
    rot_factor = ROT_RADIANS_PER_FULL_DEFLECTION_PER_SECOND * dt

    _translate(cam, tx * pan_factor, ty * pan_factor, tz * pan_factor)
    _rotate(cam, rx * rot_factor, ry * rot_factor, rz * rot_factor)


def _translate(cam, dx, dy, dz):
    # Camera position lives in world space. Use the orientation to map view-
    # local (dx, dy, dz) into world deltas.
    orient = cam.orientation.getValue()
    world_delta = orient.multVec((dx, dy, dz))
    pos = cam.position.getValue().getValue()
    cam.position.setValue(pos[0] + world_delta[0], pos[1] + world_delta[1], pos[2] + world_delta[2])


def _rotate(cam, rx, ry, rz):
    # Compose three small-angle rotations about camera-local axes.
    # SbRotation(axis, angle); multiply in pre-order so each rotation is in the
    # camera's *current* frame.
    from pivy import coin
    cur = cam.orientation.getValue()
    if rx:
        cur = cur * coin.SbRotation(coin.SbVec3f(1, 0, 0), rx)
    if ry:
        cur = cur * coin.SbRotation(coin.SbVec3f(0, 1, 0), ry)
    if rz:
        cur = cur * coin.SbRotation(coin.SbVec3f(0, 0, 1), rz)
    cam.orientation.setValue(cur)
```

- [ ] **Step 2: Hook into lifecycle**

In `lifecycle.py`, replace `_debug_handler` and the activation block:

```python
import FreeCAD

from .event_pump import EventPump
from . import libspnav, settings, axes, navigator

_state = {"active": False, "pump": None, "settings": None}


def _make_handler(s):
    def handle(ev):
        kind = ev[0]
        if kind == "motion":
            _, raw, period = ev
            scales = tuple(s.axis_scale(a) for a in settings.AXES)
            inverts = tuple(s.invert(a) for a in settings.AXES)
            transformed = axes.transform(raw, scales, inverts, s.deadzone())
            navigator.apply(transformed, period)
        elif kind == "button":
            _, bnum, pressed = ev
            if pressed:
                FreeCAD.Console.PrintMessage(f"Space Elevator: button {bnum}\n")
    return handle


def on_activate():
    if _state["active"]:
        return
    s = settings.from_freecad()
    try:
        pump = EventPump(_make_handler(s))
    except libspnav.LibspnavMissing as e:
        FreeCAD.Console.PrintError(f"Space Elevator: {e}\n")
        return
    if pump.start():
        _state.update(active=True, pump=pump, settings=s)


def on_deactivate():
    if not _state["active"]:
        return
    pump = _state.pop("pump", None)
    if pump is not None:
        pump.stop()
    _state["active"] = False
    _state["settings"] = None
```

- [ ] **Step 3: Manual integration test**

Open a FreeCAD document with any geometry (or `View → Test → Test 3` to populate). Switch to Space Elevator workbench. Move the SpaceMouse.

Expected: the camera pans/rotates smoothly. Pushing forward moves into the scene; twisting rotates about the view axis. If motion is too fast/slow, edit the two tunable constants at the top of `navigator.py` and reload (Tools → Edit parameters → restart workbench, or restart FreeCAD).

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/navigator.py freecad-addon/space_elevator/lifecycle.py
git commit -m "feat(addon): apply SpaceMouse motion to active 3D view"
```

---

## Phase 6 — Button bindings

Goal: pressing buttons fires FreeCAD commands. Bindings live in `App.ParamGet`. One global map, no per-workbench scoping yet.

### Task 6.1: Binding storage

**Files:**
- Create: `freecad-addon/space_elevator/buttons.py`
- Create: `freecad-addon/tests/test_buttons.py`

- [ ] **Step 1: Failing tests**

`freecad-addon/tests/test_buttons.py`:

```python
from space_elevator.buttons import Bindings


class FakeGroup:
    def __init__(self):
        self._strings = {}
    def GetString(self, k, d): return self._strings.get(k, d)
    def SetString(self, k, v): self._strings[k] = v


def fake_param_get():
    g = FakeGroup()
    return lambda path: g


def test_bind_and_lookup():
    b = Bindings(fake_param_get())
    b.set(0, "Std_New")
    assert b.get(0) == "Std_New"


def test_unset_button_returns_none():
    b = Bindings(fake_param_get())
    assert b.get(7) is None


def test_clear_binding():
    b = Bindings(fake_param_get())
    b.set(3, "Std_Save")
    b.set(3, "")
    assert b.get(3) is None
```

- [ ] **Step 2: Implementation**

`freecad-addon/space_elevator/buttons.py`:

```python
"""Button -> FreeCAD command bindings.

Stored as ParamGet strings keyed `button_<n>` under the addon's parameter
group. Empty string means unbound.
"""

_PATH = "User parameter:BaseApp/Preferences/Mod/SpaceElevator/Buttons"


class Bindings:
    def __init__(self, param_get):
        self._grp = param_get(_PATH)

    def get(self, bnum):
        s = self._grp.GetString(f"button_{bnum}", "")
        return s if s else None

    def set(self, bnum, command_name):
        self._grp.SetString(f"button_{bnum}", command_name or "")

    def all(self):
        # ParamGet doesn't expose enumeration cleanly; we accept that the UI
        # always knows the button count it cares about and queries by index.
        # Returning a snapshot here would require iterating raw param keys.
        raise NotImplementedError("query by index; total button count comes from device")
```

- [ ] **Step 3: Run tests**

Run: `pytest -v freecad-addon/tests/test_buttons.py`
Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/buttons.py freecad-addon/tests/test_buttons.py
git commit -m "feat(addon): button binding storage"
```

### Task 6.2: Dispatch button presses to FreeCAD commands

**Files:**
- Modify: `freecad-addon/space_elevator/lifecycle.py`

- [ ] **Step 1: Wire bindings into the handler**

Replace `_make_handler` in `lifecycle.py`:

```python
def _make_handler(s, b):
    def handle(ev):
        kind = ev[0]
        if kind == "motion":
            _, raw, period = ev
            scales = tuple(s.axis_scale(a) for a in settings.AXES)
            inverts = tuple(s.invert(a) for a in settings.AXES)
            transformed = axes.transform(raw, scales, inverts, s.deadzone())
            navigator.apply(transformed, period)
        elif kind == "button":
            _, bnum, pressed = ev
            if not pressed:
                return
            cmd = b.get(bnum)
            if cmd is None:
                return
            try:
                import FreeCADGui as Gui
                Gui.runCommand(cmd)
            except Exception as exc:
                FreeCAD.Console.PrintError(f"Space Elevator: command {cmd!r} failed: {exc}\n")
    return handle
```

Add `from . import buttons` to the imports and update `on_activate`:

```python
def on_activate():
    if _state["active"]:
        return
    s = settings.from_freecad()
    b = buttons.Bindings(__import__("FreeCAD").ParamGet)
    try:
        pump = EventPump(_make_handler(s, b))
    except libspnav.LibspnavMissing as e:
        FreeCAD.Console.PrintError(f"Space Elevator: {e}\n")
        return
    if pump.start():
        _state.update(active=True, pump=pump, settings=s, bindings=b)
```

- [ ] **Step 2: Manual test**

Seed a binding via the FreeCAD Python console:
```python
g = App.ParamGet("User parameter:BaseApp/Preferences/Mod/SpaceElevator/Buttons")
g.SetString("button_0", "Std_New")
```
Activate the workbench. Press button 0 on the SpaceMouse. Expected: a new document is created.

- [ ] **Step 3: Commit**

```bash
git add freecad-addon/space_elevator/lifecycle.py
git commit -m "feat(addon): dispatch button presses to FreeCAD commands"
```

---

## Phase 7 — LCD client (optional path)

Goal: when the daemon socket exists and `lcd_enabled` is true, push a button-label image to the LCD on activation and on bindings change. Daemon absence does not break the addon.

### Task 7.1: Daemon socket client

**Files:**
- Create: `freecad-addon/space_elevator/lcd_client.py`
- Create: `freecad-addon/tests/test_lcd_client.py`

- [ ] **Step 1: Failing tests**

`freecad-addon/tests/test_lcd_client.py`:

```python
import json
import socket
import threading

import pytest

from space_elevator.lcd_client import LcdClient, DaemonUnavailable


def _fake_server(socket_path, responses):
    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(socket_path)
    srv.listen(1)
    def run():
        conn, _ = srv.accept()
        buf = b""
        sent = 0
        while sent < len(responses):
            chunk = conn.recv(4096)
            if not chunk:
                return
            buf += chunk
            while b"\n" in buf and sent < len(responses):
                line, buf = buf.split(b"\n", 1)
                conn.sendall((json.dumps(responses[sent]) + "\n").encode())
                sent += 1
        conn.close()
        srv.close()
    threading.Thread(target=run, daemon=True).start()


def test_ping_round_trip(tmp_path):
    sock = str(tmp_path / "s.sock")
    _fake_server(sock, [{"v": 1, "id": 1, "ok": True}])
    c = LcdClient(sock)
    c.connect()
    assert c.ping() is True


def test_missing_socket_raises():
    c = LcdClient("/no/such/path.sock")
    with pytest.raises(DaemonUnavailable):
        c.connect()
```

- [ ] **Step 2: Implementation**

`freecad-addon/space_elevator/lcd_client.py`:

```python
"""Blocking client for space-elevatord's JSON-line Unix socket.

Used from the addon's Qt thread; calls are short (small SVGs, ms-scale device
writes) so blocking is acceptable. If that ever bites, move to a worker thread.
"""

import json
import os
import socket
from itertools import count


class DaemonUnavailable(RuntimeError):
    pass


class ProtocolError(RuntimeError):
    pass


def default_socket_path():
    runtime = os.environ.get("XDG_RUNTIME_DIR")
    if not runtime:
        runtime = f"/tmp/space-elevator-{os.getuid()}"
    return os.path.join(runtime, "space-elevator.sock")


class LcdClient:
    def __init__(self, path=None):
        self._path = path or default_socket_path()
        self._sock = None
        self._buf = b""
        self._id = count(1)

    def connect(self):
        if not os.path.exists(self._path):
            raise DaemonUnavailable(f"socket not found: {self._path}")
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            s.connect(self._path)
        except OSError as e:
            raise DaemonUnavailable(str(e))
        self._sock = s

    def close(self):
        if self._sock is not None:
            self._sock.close()
            self._sock = None

    def _request(self, payload):
        if self._sock is None:
            raise DaemonUnavailable("not connected")
        req = {"v": 1, "id": next(self._id)}
        req.update(payload)
        self._sock.sendall((json.dumps(req) + "\n").encode())
        return self._read_response()

    def _read_response(self):
        while b"\n" not in self._buf:
            chunk = self._sock.recv(4096)
            if not chunk:
                raise ProtocolError("daemon closed connection")
            self._buf += chunk
        line, self._buf = self._buf.split(b"\n", 1)
        return json.loads(line.decode())

    def ping(self):
        r = self._request({"cmd": "ping"})
        return r.get("ok") is True

    def clear(self):
        r = self._request({"cmd": "lcd_clear"})
        if not r.get("ok"):
            raise ProtocolError(r.get("error", "lcd_clear failed"))

    def display_svg(self, svg):
        r = self._request({"cmd": "lcd_display_svg", "svg": svg})
        if not r.get("ok"):
            raise ProtocolError(r.get("error", "lcd_display_svg failed"))
```

- [ ] **Step 3: Run tests**

Run: `pytest -v freecad-addon/tests/test_lcd_client.py`
Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/lcd_client.py freecad-addon/tests/test_lcd_client.py
git commit -m "feat(addon): unix socket client for space-elevatord"
```

### Task 7.2: Render button labels to LCD on activation

**Files:**
- Create: `freecad-addon/space_elevator/lcd_labels.py`
- Modify: `freecad-addon/space_elevator/lifecycle.py`

- [ ] **Step 1: Label renderer**

`freecad-addon/space_elevator/lcd_labels.py`:

```python
"""Build an SVG showing labels for the first N buttons in a row.

Hard-coded layout for v1: 8 cells, equal width across 640px, 150px tall, white
text on dark background. Future iterations can do grids, per-workbench layouts,
icons, etc.
"""

from html import escape

_W = 640
_H = 150
_COLS = 8
_CELL_W = _W // _COLS
_FONT_SIZE = 22
_BG = "#001428"
_FG = "#f0f0f0"
_BORDER = "#274869"


def render(labels):
    """labels: iterable of strings; the first 8 entries are drawn."""
    labels = list(labels)[:_COLS] + [""] * max(0, _COLS - len(labels))
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {_W} {_H}">',
        f'<rect width="{_W}" height="{_H}" fill="{_BG}"/>',
    ]
    for i, lbl in enumerate(labels):
        x = i * _CELL_W
        parts.append(
            f'<rect x="{x}" y="0" width="{_CELL_W}" height="{_H}" fill="none" stroke="{_BORDER}" stroke-width="1"/>'
        )
        cx = x + _CELL_W // 2
        parts.append(
            f'<text x="{cx}" y="{_H//2 + _FONT_SIZE//3}" font-size="{_FONT_SIZE}" '
            f'font-family="sans-serif" fill="{_FG}" text-anchor="middle">{escape(lbl)}</text>'
        )
    parts.append("</svg>")
    return "".join(parts)
```

- [ ] **Step 2: Push labels on activate**

Edit `lifecycle.py`, add at top:
```python
from . import lcd_client, lcd_labels
```

Add a helper:
```python
def _push_labels(s, b):
    if not s.lcd_enabled():
        return
    path = s.daemon_socket_path() or None
    client = lcd_client.LcdClient(path)
    try:
        client.connect()
    except lcd_client.DaemonUnavailable:
        return  # daemon optional
    try:
        labels = [_short_name(b.get(i) or "") for i in range(8)]
        client.display_svg(lcd_labels.render(labels))
    except lcd_client.ProtocolError as exc:
        FreeCAD.Console.PrintWarning(f"Space Elevator LCD: {exc}\n")
    finally:
        client.close()


def _short_name(cmd):
    # "Std_New" -> "New"; "Sketcher_NewSketch" -> "NewSketch"
    if "_" in cmd:
        return cmd.split("_", 1)[1]
    return cmd
```

Call `_push_labels(s, b)` at the end of `on_activate` (after `pump.start()` returns True).

- [ ] **Step 3: Manual integration test**

With the daemon running and at least one button binding set, switch to the Space Elevator workbench. Expected: LCD shows 8 cells, populated cells display the short command name.

With the daemon **not** running: workbench still activates, motion still works, no error popups — just a quiet skip.

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/lcd_labels.py freecad-addon/space_elevator/lifecycle.py
git commit -m "feat(addon): push button labels to LCD on activation"
```

---

## Phase 8 — Preferences UI

Goal: a Preferences page inside FreeCAD's `Tools → Edit parameters` flow (or `Edit → Preferences → Space Elevator`) for axis scale, deadzone, LCD enable, and per-button command binding.

### Task 8.1: Register a Preferences page

**Files:**
- Create: `freecad-addon/Resources/ui/preferences.ui`
- Create: `freecad-addon/space_elevator/preferences.py`
- Modify: `freecad-addon/InitGui.py`

- [ ] **Step 1: Write the Qt UI file**

`freecad-addon/Resources/ui/preferences.ui` (hand-written XML — small and reproducible):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ui version="4.0">
 <class>SpaceElevatorPreferences</class>
 <widget class="QWidget" name="SpaceElevatorPreferences">
  <property name="windowTitle"><string>Space Elevator</string></property>
  <layout class="QVBoxLayout">
    <item>
      <widget class="QGroupBox" name="axisGroup">
        <property name="title"><string>Axis scaling</string></property>
        <layout class="QFormLayout">
          <item row="0" column="0"><widget class="QLabel"><property name="text"><string>tx</string></property></widget></item>
          <item row="0" column="1"><widget class="Gui::PrefDoubleSpinBox" name="tx"><property name="prefEntry" stdset="0"><cstring>axis_scale_tx</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="minimum"><double>-10.0</double></property><property name="maximum"><double>10.0</double></property><property name="singleStep"><double>0.1</double></property><property name="value"><double>1.0</double></property></widget></item>
          <item row="1" column="0"><widget class="QLabel"><property name="text"><string>ty</string></property></widget></item>
          <item row="1" column="1"><widget class="Gui::PrefDoubleSpinBox" name="ty"><property name="prefEntry" stdset="0"><cstring>axis_scale_ty</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="minimum"><double>-10.0</double></property><property name="maximum"><double>10.0</double></property><property name="singleStep"><double>0.1</double></property><property name="value"><double>1.0</double></property></widget></item>
          <item row="2" column="0"><widget class="QLabel"><property name="text"><string>tz</string></property></widget></item>
          <item row="2" column="1"><widget class="Gui::PrefDoubleSpinBox" name="tz"><property name="prefEntry" stdset="0"><cstring>axis_scale_tz</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="minimum"><double>-10.0</double></property><property name="maximum"><double>10.0</double></property><property name="singleStep"><double>0.1</double></property><property name="value"><double>1.0</double></property></widget></item>
          <item row="3" column="0"><widget class="QLabel"><property name="text"><string>rx</string></property></widget></item>
          <item row="3" column="1"><widget class="Gui::PrefDoubleSpinBox" name="rx"><property name="prefEntry" stdset="0"><cstring>axis_scale_rx</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="minimum"><double>-10.0</double></property><property name="maximum"><double>10.0</double></property><property name="singleStep"><double>0.1</double></property><property name="value"><double>1.0</double></property></widget></item>
          <item row="4" column="0"><widget class="QLabel"><property name="text"><string>ry</string></property></widget></item>
          <item row="4" column="1"><widget class="Gui::PrefDoubleSpinBox" name="ry"><property name="prefEntry" stdset="0"><cstring>axis_scale_ry</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="minimum"><double>-10.0</double></property><property name="maximum"><double>10.0</double></property><property name="singleStep"><double>0.1</double></property><property name="value"><double>1.0</double></property></widget></item>
          <item row="5" column="0"><widget class="QLabel"><property name="text"><string>rz</string></property></widget></item>
          <item row="5" column="1"><widget class="Gui::PrefDoubleSpinBox" name="rz"><property name="prefEntry" stdset="0"><cstring>axis_scale_rz</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="minimum"><double>-10.0</double></property><property name="maximum"><double>10.0</double></property><property name="singleStep"><double>0.1</double></property><property name="value"><double>1.0</double></property></widget></item>
          <item row="6" column="0"><widget class="QLabel"><property name="text"><string>Deadzone</string></property></widget></item>
          <item row="6" column="1"><widget class="Gui::PrefDoubleSpinBox" name="deadzone"><property name="prefEntry" stdset="0"><cstring>deadzone</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="minimum"><double>0.0</double></property><property name="maximum"><double>0.5</double></property><property name="singleStep"><double>0.01</double></property><property name="value"><double>0.02</double></property></widget></item>
        </layout>
      </widget>
    </item>
    <item>
      <widget class="QGroupBox" name="lcdGroup">
        <property name="title"><string>LCD</string></property>
        <layout class="QFormLayout">
          <item row="0" column="0"><widget class="QLabel"><property name="text"><string>Enable LCD output</string></property></widget></item>
          <item row="0" column="1"><widget class="Gui::PrefCheckBox" name="lcd_enabled"><property name="prefEntry" stdset="0"><cstring>lcd_enabled</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property><property name="checked"><bool>true</bool></property></widget></item>
          <item row="1" column="0"><widget class="QLabel"><property name="text"><string>Daemon socket (blank = default)</string></property></widget></item>
          <item row="1" column="1"><widget class="Gui::PrefLineEdit" name="daemon_socket_path"><property name="prefEntry" stdset="0"><cstring>daemon_socket_path</cstring></property><property name="prefPath" stdset="0"><cstring>Mod/SpaceElevator</cstring></property></widget></item>
        </layout>
      </widget>
    </item>
  </layout>
 </widget>
</ui>
```

(`Gui::PrefDoubleSpinBox`, `Gui::PrefCheckBox`, `Gui::PrefLineEdit` are FreeCAD's preference-aware widgets — they read/write `App.ParamGet` themselves, so no Python wiring is needed for these fields.)

- [ ] **Step 2: Page registration**

`freecad-addon/space_elevator/preferences.py`:

```python
import os
import FreeCADGui as Gui

_UI_PATH = os.path.join(os.path.dirname(__file__), "..", "Resources", "ui", "preferences.ui")


def register():
    Gui.addPreferencePage(os.path.abspath(_UI_PATH), "Space Elevator")
```

Modify `InitGui.py` to call `register()` from inside `SpaceElevatorWorkbench.Initialize`:

```python
def Initialize(self):
    FreeCAD.Console.PrintMessage("Space Elevator: workbench initialized\n")
    from space_elevator import preferences
    preferences.register()
```

- [ ] **Step 3: Manual test**

Restart FreeCAD. Open `Edit → Preferences`. Expected: "Space Elevator" section in the left tree with the axis-scale, deadzone, LCD-enable, and socket-path fields. Changes persist between restarts (verified by reading the params via the console).

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/Resources/ui/preferences.ui freecad-addon/space_elevator/preferences.py freecad-addon/InitGui.py
git commit -m "feat(addon): Preferences page for axis/deadzone/LCD settings"
```

### Task 8.2: Button binding editor (button table)

**Files:**
- Modify: `freecad-addon/Resources/ui/preferences.ui` (add button table) — *or* create a separate dialog if simpler
- Create: `freecad-addon/space_elevator/button_dialog.py`
- Modify: `freecad-addon/InitGui.py` (menu entry to open dialog)

For v1 we ship the binding editor as a **separate dialog launched from a menu entry**, not embedded in the Preferences page. Embedding a dynamic-row table inside a static `.ui` file is fiddly; a small Python-built dialog is the lower-risk path.

- [ ] **Step 1: Dialog code**

`freecad-addon/space_elevator/button_dialog.py`:

```python
try:
    from PySide6 import QtWidgets, QtCore
except ImportError:
    from PySide2 import QtWidgets, QtCore

import FreeCAD
import FreeCADGui as Gui

from . import buttons


N_BUTTONS = 31  # SpaceMouse Enterprise has 31 mappable buttons


class BindingsDialog(QtWidgets.QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Space Elevator — Button bindings")
        self.resize(520, 600)

        self._bindings = buttons.Bindings(FreeCAD.ParamGet)
        self._command_names = sorted(Gui.listCommands())

        layout = QtWidgets.QVBoxLayout(self)

        table = QtWidgets.QTableWidget(N_BUTTONS, 2, self)
        table.setHorizontalHeaderLabels(["Button", "Command"])
        table.horizontalHeader().setStretchLastSection(True)
        for i in range(N_BUTTONS):
            table.setItem(i, 0, QtWidgets.QTableWidgetItem(f"#{i}"))
            combo = QtWidgets.QComboBox()
            combo.addItem("(none)", "")
            for name in self._command_names:
                combo.addItem(name, name)
            current = self._bindings.get(i) or ""
            idx = combo.findData(current)
            if idx >= 0:
                combo.setCurrentIndex(idx)
            table.setCellWidget(i, 1, combo)
        self._table = table
        layout.addWidget(table)

        buttons_box = QtWidgets.QDialogButtonBox(
            QtWidgets.QDialogButtonBox.Save | QtWidgets.QDialogButtonBox.Cancel
        )
        buttons_box.accepted.connect(self._save)
        buttons_box.rejected.connect(self.reject)
        layout.addWidget(buttons_box)

    def _save(self):
        for i in range(N_BUTTONS):
            combo = self._table.cellWidget(i, 1)
            self._bindings.set(i, combo.currentData())
        self.accept()


def show():
    dlg = BindingsDialog(Gui.getMainWindow())
    dlg.exec_()
```

- [ ] **Step 2: Menu entry**

Add to `InitGui.py`, after the workbench class:

```python
class _ButtonBindingsCmd:
    def GetResources(self):
        return {"MenuText": "Button bindings…", "ToolTip": "Bind SpaceMouse buttons to FreeCAD commands"}

    def Activated(self):
        from space_elevator.button_dialog import show
        show()

    def IsActive(self):
        return True


Gui.addCommand("SpaceElevator_ButtonBindings", _ButtonBindingsCmd())
```

In `SpaceElevatorWorkbench.Initialize`, register a menu:
```python
def Initialize(self):
    FreeCAD.Console.PrintMessage("Space Elevator: workbench initialized\n")
    from space_elevator import preferences
    preferences.register()
    self.appendMenu("Space Elevator", ["SpaceElevator_ButtonBindings"])
```

- [ ] **Step 3: Manual test**

Restart FreeCAD, switch to Space Elevator workbench. Expected: a "Space Elevator" menu appears with "Button bindings…". Opening it shows a 31-row table; pick `Std_New` for button 0, Save, press button 0 on the SpaceMouse → new doc opens.

- [ ] **Step 4: Commit**

```bash
git add freecad-addon/space_elevator/button_dialog.py freecad-addon/InitGui.py
git commit -m "feat(addon): button bindings dialog"
```

---

## Self-review checklist

- **Spec coverage:**
  - Two-component architecture (daemon + Python addon): Phase 1-2 (daemon) + Phase 3-8 (addon). ✔
  - Motion via libspnav direct + QSocketNotifier: Phase 4. ✔
  - LCD via daemon, optional, degrades gracefully: Phase 7 (`DaemonUnavailable` returns silently). ✔
  - Config in FreeCAD Preferences page: Phase 8.1. ✔
  - Tauri+Vue deleted: Phase 0.2. ✔
  - Cargo workspace with three crates: Phase 0.3 + 1.1. ✔
  - Single-preset/single-binding-map cuts: explicitly stated up-front and acknowledged again in Phase 5/6 task scope.
  - Out-of-scope items (macOS/Windows, Onshape extension, multi-client daemon) intentionally absent.
- **Placeholders:** none. Every step has runnable commands or complete code.
- **Type consistency:** `LcdClient` / `lcd_client.LcdClient` / `LcdHandle` (Rust) used consistently. `Bindings`, `Settings`, `EventPump` used consistently. `RequestPayload::LcdDisplayImage{path}` matches the JSON `cmd: "lcd_display_image", path: "..."` shape via `#[serde(tag="cmd", rename_all="snake_case")]`. Verified.
- **Known unknowns surfaced inline:**
  - `spnav::lcd` packet header is unfinished (Phase 2 intro flags this).
  - PySide6 vs PySide2 — try/except fallback in both `event_pump.py` and `button_dialog.py`.
  - libspnav `max_raw=350` is empirical for the SpaceMouse Enterprise; if the SDK reports differently, expose the cap in Settings later.
