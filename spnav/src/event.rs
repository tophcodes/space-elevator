use crate::error::{Error, Result};
use spnav_sys as ffi;

/// A space mouse event
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Event {
    /// 6-DOF motion event
    Motion(MotionEvent),

    /// Button press or release
    Button(ButtonEvent),
}

/// Motion data from a 6-DOF device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MotionEvent {
    /// Translation along the x axis
    pub x: i32,
    /// Translation along the y axis
    pub y: i32,
    /// Translation along the z axis
    pub z: i32,
    /// Rotation around the x axis
    pub rx: i32,
    /// Rotation around the y axis
    pub ry: i32,
    /// Rotation around the z axis
    pub rz: i32,
    /// Period (timestamp) of the event
    pub period: u32,
}

impl MotionEvent {
    /// Get translation as [x, y, z]
    pub fn translation(&self) -> [i32; 3] {
        [self.x, self.y, self.z]
    }

    /// Get rotation as [rx, ry, rz]
    pub fn rotation(&self) -> [i32; 3] {
        [self.rx, self.ry, self.rz]
    }

    /// Check if this is a zero motion event (all axes are zero)
    ///
    /// Returns true if all translation and rotation values are zero.
    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0 && self.z == 0 && self.rx == 0 && self.ry == 0 && self.rz == 0
    }
}

/// Button event from a 6-DOF device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ButtonEvent {
    /// Button number (0-based)
    pub button: u32,
    /// True if pressed, false if released
    pub pressed: bool,
}

impl Event {
    /// Convert from raw FFI event
    pub(crate) fn from_raw(raw: ffi::spnav_event) -> Result<Self> {
        unsafe {
            match raw.type_ as u32 {
                ffi::SPNAV_EVENT_MOTION => {
                    let motion = raw.motion;
                    Ok(Event::Motion(MotionEvent {
                        x: motion.x,
                        y: motion.y,
                        z: motion.z,
                        rx: motion.rx,
                        ry: motion.ry,
                        rz: motion.rz,
                        period: motion.period,
                    }))
                }
                ffi::SPNAV_EVENT_BUTTON => {
                    let button = raw.button;
                    Ok(Event::Button(ButtonEvent {
                        button: button.bnum as u32,
                        pressed: button.press != 0,
                    }))
                }
                unknown => Err(Error::UnknownEventType(unknown as i32)),
            }
        }
    }
}
