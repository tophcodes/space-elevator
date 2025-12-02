/// Axis inversion configuration builder
///
/// Provides a type-safe way to configure which axes should be inverted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AxisInversion {
    tx: bool,
    ty: bool,
    tz: bool,
    rx: bool,
    ry: bool,
    rz: bool,
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

    // Query methods

    /// Check if Translation X is inverted
    pub fn tx(&self) -> bool {
        self.tx
    }

    /// Check if Translation Y is inverted
    pub fn ty(&self) -> bool {
        self.ty
    }

    /// Check if Translation Z is inverted
    pub fn tz(&self) -> bool {
        self.tz
    }

    /// Check if Rotation X is inverted
    pub fn rx(&self) -> bool {
        self.rx
    }

    /// Check if Rotation Y is inverted
    pub fn ry(&self) -> bool {
        self.ry
    }

    /// Check if Rotation Z is inverted
    pub fn rz(&self) -> bool {
        self.rz
    }

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
