/// Button action for special button mappings
///
/// Allows mapping buttons to special functions instead of regular button events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i32)]
pub enum ButtonAction {
    /// No action mapped, use as regular button (default)
    None = 0,
    /// Reset sensitivity to 1.0
    SensitivityReset = 1,
    /// Increase sensitivity
    SensitivityIncrease = 2,
    /// Decrease sensitivity
    SensitivityDecrease = 3,
    /// Disable rotation while button is held down
    DisableRotation = 4,
    /// Disable translation while button is held down
    DisableTranslation = 5,
}

impl ButtonAction {
    /// Convert from raw C integer value
    pub fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::SensitivityReset),
            2 => Some(Self::SensitivityIncrease),
            3 => Some(Self::SensitivityDecrease),
            4 => Some(Self::DisableRotation),
            5 => Some(Self::DisableTranslation),
            _ => None,
        }
    }

    /// Get the raw C integer value
    pub fn to_raw(self) -> i32 {
        self as i32
    }

    /// Check if this action is a sensitivity modifier
    pub fn is_sensitivity_action(&self) -> bool {
        matches!(
            self,
            Self::SensitivityReset | Self::SensitivityIncrease | Self::SensitivityDecrease
        )
    }

    /// Check if this action is a motion disabler
    pub fn is_disable_action(&self) -> bool {
        matches!(self, Self::DisableRotation | Self::DisableTranslation)
    }
}

impl std::fmt::Display for ButtonAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None (regular button)"),
            Self::SensitivityReset => write!(f, "Reset Sensitivity"),
            Self::SensitivityIncrease => write!(f, "Increase Sensitivity"),
            Self::SensitivityDecrease => write!(f, "Decrease Sensitivity"),
            Self::DisableRotation => write!(f, "Disable Rotation (while held)"),
            Self::DisableTranslation => write!(f, "Disable Translation (while held)"),
        }
    }
}
