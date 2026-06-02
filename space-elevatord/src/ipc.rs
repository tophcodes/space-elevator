use serde::{Deserialize, Serialize};
use crate::lcd_template::LcdState;

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
    LcdSetState(LcdState),
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

    #[test]
    fn parses_lcd_set_state_request() {
        let raw = r#"{"v":1,"id":9,"cmd":"lcd_set_state","theme":"signal","profile":"FreeCAD","mode":"Sketcher","left":[{"label":"Line","icon":"line","cat":"draw"}],"right":[]}"#;
        let req: Request = serde_json::from_str(raw).unwrap();
        assert_eq!(req.id, 9);
        match req.payload {
            RequestPayload::LcdSetState(s) => {
                assert_eq!(s.profile, "FreeCAD");
                assert_eq!(s.mode, "Sketcher");
                assert_eq!(s.left.len(), 1);
                assert_eq!(s.left[0].label, "Line");
                assert_eq!(s.left[0].cat.as_deref(), Some("draw"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn lcd_set_state_defaults_theme_signal() {
        let raw = r#"{"v":1,"id":1,"cmd":"lcd_set_state"}"#;
        let req: Request = serde_json::from_str(raw).unwrap();
        match req.payload {
            RequestPayload::LcdSetState(s) => assert_eq!(s.theme, crate::lcd_template::Theme::Signal),
            _ => panic!("wrong variant"),
        }
    }
}
