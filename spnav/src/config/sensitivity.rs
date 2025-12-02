/// Sensitivity configuration builder
///
/// Provides a type-safe way to configure axis sensitivities.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sensitivity {
    /// Translation X sensitivity
    pub tx: f32,
    /// Translation Y sensitivity
    pub ty: f32,
    /// Translation Z sensitivity
    pub tz: f32,
    /// Rotation X sensitivity
    pub rx: f32,
    /// Rotation Y sensitivity
    pub ry: f32,
    /// Rotation Z sensitivity
    pub rz: f32,
}

impl Sensitivity {
    /// Create with default sensitivity (1.0) for all axes
    pub fn new() -> Self {
        Self {
            tx: 1.0,
            ty: 1.0,
            tz: 1.0,
            rx: 1.0,
            ry: 1.0,
            rz: 1.0,
        }
    }

    /// Create from raw array [TX, TY, TZ, RX, RY, RZ]
    pub fn from_array(values: [f32; 6]) -> Self {
        Self {
            tx: values[0],
            ty: values[1],
            tz: values[2],
            rx: values[3],
            ry: values[4],
            rz: values[5],
        }
    }

    /// Create with same value for all axes
    pub fn uniform(sensitivity: f32) -> Self {
        Self {
            tx: sensitivity,
            ty: sensitivity,
            tz: sensitivity,
            rx: sensitivity,
            ry: sensitivity,
            rz: sensitivity,
        }
    }

    /// Create with separate translation and rotation values
    pub fn translation_rotation(translation: f32, rotation: f32) -> Self {
        Self {
            tx: translation,
            ty: translation,
            tz: translation,
            rx: rotation,
            ry: rotation,
            rz: rotation,
        }
    }

    /// Get the raw array [TX, TY, TZ, RX, RY, RZ]
    pub fn as_array(&self) -> [f32; 6] {
        [self.tx, self.ty, self.tz, self.rx, self.ry, self.rz]
    }

    // Builder methods for individual axes

    pub fn with_tx(mut self, sensitivity: f32) -> Self {
        self.tx = sensitivity;
        self
    }

    pub fn with_ty(mut self, sensitivity: f32) -> Self {
        self.ty = sensitivity;
        self
    }

    pub fn with_tz(mut self, sensitivity: f32) -> Self {
        self.tz = sensitivity;
        self
    }

    pub fn with_rx(mut self, sensitivity: f32) -> Self {
        self.rx = sensitivity;
        self
    }

    pub fn with_ry(mut self, sensitivity: f32) -> Self {
        self.ry = sensitivity;
        self
    }

    pub fn with_rz(mut self, sensitivity: f32) -> Self {
        self.rz = sensitivity;
        self
    }

    // Utility methods

    /// Get translation sensitivities [TX, TY, TZ]
    pub fn translation(&self) -> [f32; 3] {
        [self.tx, self.ty, self.tz]
    }

    /// Get rotation sensitivities [RX, RY, RZ]
    pub fn rotation(&self) -> [f32; 3] {
        [self.rx, self.ry, self.rz]
    }

    /// Set all translation axes to the same value
    pub fn with_translation(mut self, sensitivity: f32) -> Self {
        self.tx = sensitivity;
        self.ty = sensitivity;
        self.tz = sensitivity;
        self
    }

    /// Set all rotation axes to the same value
    pub fn with_rotation(mut self, sensitivity: f32) -> Self {
        self.rx = sensitivity;
        self.ry = sensitivity;
        self.rz = sensitivity;
        self
    }

    /// Scale all values by a multiplier
    pub fn scale(mut self, multiplier: f32) -> Self {
        self.tx *= multiplier;
        self.ty *= multiplier;
        self.tz *= multiplier;
        self.rx *= multiplier;
        self.ry *= multiplier;
        self.rz *= multiplier;
        self
    }
}

impl Default for Sensitivity {
    fn default() -> Self {
        Self::new()
    }
}
