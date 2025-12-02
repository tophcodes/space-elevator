/// Event grab mode configuration
///
/// Controls whether the application receives events exclusively or shares
/// them with other applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GrabMode {
    /// Share events with other applications
    Normal = 0,
    /// Exclusively grab the device
    Exclusive = 1,
}

impl From<i32> for GrabMode {
    /// Convert from raw C integer value
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Exclusive,
            _ => Self::Normal,
        }
    }
}
