/// Device axis identifiers
///
/// Represents the 6 physical axes of a 6-DOF space mouse device.
/// These are the raw axes as they come from the hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i32)]
pub enum DeviceAxis {
    TranslationX = 0,
    TranslationY = 1,
    TranslationZ = 2,
    RotationX = 3,
    RotationY = 4,
    RotationZ = 5,
}

impl DeviceAxis {
    /// Create from 0-based axis index
    ///
    /// # Arguments
    ///
    /// * `index` - Axis index (0-5)
    ///
    /// # Returns
    ///
    /// The corresponding device axis, or `None` if index is out of range.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::TranslationX),
            1 => Some(Self::TranslationY),
            2 => Some(Self::TranslationZ),
            3 => Some(Self::RotationX),
            4 => Some(Self::RotationY),
            5 => Some(Self::RotationZ),
            _ => None,
        }
    }

    /// Get all device axes in order [TX, TY, TZ, RX, RY, RZ]
    ///
    /// Returns an array containing all six device axes in the standard order.
    pub fn all() -> [Self; 6] {
        [
            Self::TranslationX,
            Self::TranslationY,
            Self::TranslationZ,
            Self::RotationX,
            Self::RotationY,
            Self::RotationZ,
        ]
    }
}

/// Input axis identifiers (logical axes after remapping)
///
/// These represent the logical axes that applications receive after
/// any device-to-input axis mapping has been applied by spacenavd.
/// These use the short TX/TY/TZ/RX/RY/RZ naming convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i32)]
pub enum InputAxis {
    TX = 0,
    TY = 1,
    TZ = 2,
    RX = 3,
    RY = 4,
    RZ = 5,
}
