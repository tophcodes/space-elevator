/// Sensitivity configuration builder
///
/// Provides a type-safe way to configure axis sensitivities.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sensitivity {
    values: [f32; 6],
}

impl Sensitivity {
    /// Create with default sensitivity (1.0) for all axes
    pub fn new() -> Self {
        Self { values: [1.0; 6] }
    }

    /// Create from raw array [TX, TY, TZ, RX, RY, RZ]
    pub fn from_array(values: [f32; 6]) -> Self {
        Self { values }
    }

    /// Create with same value for all axes
    pub fn uniform(sensitivity: f32) -> Self {
        Self {
            values: [sensitivity; 6],
        }
    }

    /// Create with separate translation and rotation values
    pub fn translation_rotation(translation: f32, rotation: f32) -> Self {
        Self {
            values: [
                translation,
                translation,
                translation,
                rotation,
                rotation,
                rotation,
            ],
        }
    }

    /// Get the raw array [TX, TY, TZ, RX, RY, RZ]
    pub fn as_array(&self) -> [f32; 6] {
        self.values
    }

    // Builder methods for individual axes

    pub fn with_tx(mut self, sensitivity: f32) -> Self {
        self.values[0] = sensitivity;
        self
    }

    pub fn with_ty(mut self, sensitivity: f32) -> Self {
        self.values[1] = sensitivity;
        self
    }

    pub fn with_tz(mut self, sensitivity: f32) -> Self {
        self.values[2] = sensitivity;
        self
    }

    pub fn with_rx(mut self, sensitivity: f32) -> Self {
        self.values[3] = sensitivity;
        self
    }

    pub fn with_ry(mut self, sensitivity: f32) -> Self {
        self.values[4] = sensitivity;
        self
    }

    pub fn with_rz(mut self, sensitivity: f32) -> Self {
        self.values[5] = sensitivity;
        self
    }

    // Query methods

    pub fn tx(&self) -> f32 {
        self.values[0]
    }
    pub fn ty(&self) -> f32 {
        self.values[1]
    }
    pub fn tz(&self) -> f32 {
        self.values[2]
    }
    pub fn rx(&self) -> f32 {
        self.values[3]
    }
    pub fn ry(&self) -> f32 {
        self.values[4]
    }
    pub fn rz(&self) -> f32 {
        self.values[5]
    }

    /// Get translation sensitivities [TX, TY, TZ]
    pub fn translation(&self) -> [f32; 3] {
        [self.values[0], self.values[1], self.values[2]]
    }

    /// Get rotation sensitivities [RX, RY, RZ]
    pub fn rotation(&self) -> [f32; 3] {
        [self.values[3], self.values[4], self.values[5]]
    }

    /// Set all translation axes to the same value
    pub fn with_translation(mut self, sensitivity: f32) -> Self {
        self.values[0] = sensitivity;
        self.values[1] = sensitivity;
        self.values[2] = sensitivity;
        self
    }

    /// Set all rotation axes to the same value
    pub fn with_rotation(mut self, sensitivity: f32) -> Self {
        self.values[3] = sensitivity;
        self.values[4] = sensitivity;
        self.values[5] = sensitivity;
        self
    }

    /// Scale all values by a multiplier
    pub fn scale(mut self, multiplier: f32) -> Self {
        for value in &mut self.values {
            *value *= multiplier;
        }
        self
    }
}

impl Default for Sensitivity {
    fn default() -> Self {
        Self::new()
    }
}
