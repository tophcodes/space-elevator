use crate::ipc::{Request, RequestPayload, Response};
use crate::lcd_handle::LcdHandle;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{info, warn};

pub fn socket_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/tmp/space-elevator-{}", whoami_uid()));
    PathBuf::from(dir).join("space-elevator.sock")
}

fn whoami_uid() -> u32 {
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

    let lcd = LcdHandle::new();

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

async fn handle_client(stream: UnixStream, lcd: LcdHandle) -> std::io::Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half).lines();

    while let Some(line) = reader.next_line().await? {
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => dispatch(req, &lcd).await,
            Err(e) => Response::err(0, format!("invalid request: {e}")),
        };
        let mut buf = serde_json::to_vec(&response).expect("serialise");
        buf.push(b'\n');
        write_half.write_all(&buf).await?;
    }
    Ok(())
}

async fn dispatch(req: Request, lcd: &LcdHandle) -> Response {
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
        RequestPayload::LcdDisplaySvg { svg } => match crate::svg_render::render_to_rgb888(&svg) {
            Ok(rgb) => match lcd.display_rgb888(&rgb).await {
                Ok(()) => Response::ok(req.id),
                Err(e) => Response::err(req.id, e.to_string()),
            },
            Err(e) => Response::err(req.id, e),
        },
        RequestPayload::LcdSetState(state) => {
            let svg = crate::lcd_template::render(&state);
            match crate::svg_render::render_to_rgb888(&svg) {
                Ok(rgb) => match lcd.display_rgb888(&rgb).await {
                    Ok(()) => Response::ok(req.id),
                    Err(e) => Response::err(req.id, e.to_string()),
                },
                Err(e) => Response::err(req.id, e),
            }
        }
    }
}

fn load_rgb888(path: &str) -> Result<Vec<u8>, String> {
    let img = image::open(path).map_err(|e| format!("image open: {e}"))?;
    let rgb = img.into_rgb8();
    if rgb.width() != 640 || rgb.height() != 150 {
        return Err(format!(
            "image must be 640x150, got {}x{}",
            rgb.width(),
            rgb.height()
        ));
    }
    Ok(rgb.into_raw())
}
