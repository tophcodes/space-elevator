/// LED state configuration
///
/// Controls the state of the LED on space mouse devices that have one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LedState {
    /// LED is turned off
    Off = 0,
    /// LED is turned on
    On = 1,
    /// LED state is managed automatically by spacenavd
    Auto = 2,
}

impl From<i32> for LedState {
    /// Convert from raw C integer value
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::On,
            _ => Self::Auto,
        }
    }
}
