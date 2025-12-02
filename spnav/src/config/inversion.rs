/// Axis inversion configuration builder
///
/// Provides a type-safe way to configure which axes should be inverted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AxisInversion {
    /// Translation X axis is inverted
    pub tx: bool,
    /// Translation Y axis is inverted
    pub ty: bool,
    /// Translation Z axis is inverted
    pub tz: bool,
    /// Rotation X axis is inverted
    pub rx: bool,
    /// Rotation Y axis is inverted
    pub ry: bool,
    /// Rotation Z axis is inverted
    pub rz: bool,
}

impl AxisInversion {
    /// Create a new inversion configuration with all axes normal (not inverted)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a bitmask
    pub fn from_mask(mask: u32) -> Self {
        Self {
            tx: (mask & (1 << 0)) != 0,
            ty: (mask & (1 << 1)) != 0,
            tz: (mask & (1 << 2)) != 0,
            rx: (mask & (1 << 3)) != 0,
            ry: (mask & (1 << 4)) != 0,
            rz: (mask & (1 << 5)) != 0,
        }
    }

    /// Convert to bitmask
    pub fn to_mask(self) -> u32 {
        let mut mask = 0u32;
        if self.tx {
            mask |= 1 << 0;
        }
        if self.ty {
            mask |= 1 << 1;
        }
        if self.tz {
            mask |= 1 << 2;
        }
        if self.rx {
            mask |= 1 << 3;
        }
        if self.ry {
            mask |= 1 << 4;
        }
        if self.rz {
            mask |= 1 << 5;
        }
        mask
    }

    // Builder methods

    /// Set Translation X inversion
    pub fn with_tx(mut self, inverted: bool) -> Self {
        self.tx = inverted;
        self
    }

    /// Set Translation Y inversion
    pub fn with_ty(mut self, inverted: bool) -> Self {
        self.ty = inverted;
        self
    }

    /// Set Translation Z inversion
    pub fn with_tz(mut self, inverted: bool) -> Self {
        self.tz = inverted;
        self
    }

    /// Set Rotation X inversion
    pub fn with_rx(mut self, inverted: bool) -> Self {
        self.rx = inverted;
        self
    }

    /// Set Rotation Y inversion
    pub fn with_ry(mut self, inverted: bool) -> Self {
        self.ry = inverted;
        self
    }

    /// Set Rotation Z inversion
    pub fn with_rz(mut self, inverted: bool) -> Self {
        self.rz = inverted;
        self
    }

    // Utility methods

    /// Invert all translation axes
    pub fn invert_translation(self) -> Self {
        self.with_tx(true).with_ty(true).with_tz(true)
    }

    /// Invert all rotation axes
    pub fn invert_rotation(self) -> Self {
        self.with_rx(true).with_ry(true).with_rz(true)
    }

    /// Invert all axes
    pub fn invert_all(self) -> Self {
        self.invert_translation().invert_rotation()
    }
}
