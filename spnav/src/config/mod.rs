pub mod action;
pub mod axis;
pub mod grabmode;
pub mod inversion;
pub mod led;
pub mod sensitivity;

pub use action::ButtonAction;
pub use axis::{DeviceAxis, InputAxis};
pub use grabmode::GrabMode;
pub use inversion::AxisInversion;
pub use led::LedState;
pub use sensitivity::Sensitivity;

use crate::{Error, Result};
use spnav_sys as ffi;
use std::ffi::{CStr, CString};

/// Configuration interface for spacenavd
///
/// **IMPORTANT**: These operations modify global spacenavd settings that affect
/// all applications. Use responsibly, typically only in dedicated configuration
/// tools or for application-specific temporary changes.
///
/// Changes made through this API affect the running daemon immediately but are
/// lost on daemon restart unless [`Config::save`] is called.
///
/// # Configuration Categories
///
/// - **Deadzone**: Noise filtering thresholds per axis
/// - **Sensitivity**: Motion scaling (global and per-axis)
/// - **Axis Mapping**: Remap device axes to input axes
/// - **Axis Inversion**: Reverse direction of axes
/// - **Axis Swapping**: Swap Y and Z axes
/// - **Button Mapping**: Remap button numbers
/// - **LED Control**: Control device LED state
/// - **Grab Mode**: Exclusive vs shared event handling
/// - **Serial Device**: Configure serial port (for serial devices)
/// - **Persistence**: Save/restore/reset configuration
pub struct Config<'a> {
    _spnav: &'a mut crate::SpaceNav,
}

impl<'a> Config<'a> {
    pub(crate) fn new(spnav: &'a mut crate::SpaceNav) -> Self {
        Self { _spnav: spnav }
    }

    /// Set deadzone threshold for a specific device axis
    ///
    /// Motion values below this threshold are discarded as noise.
    ///
    /// # Arguments
    ///
    /// * `axis` - Device axis to configure
    /// * `threshold` - Deadzone threshold (typically 0-100)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::{SpaceNav, config::DeviceAxis};
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    /// cfg.set_deadzone(DeviceAxis::TranslationX, 10)?;
    /// cfg.save()?;  // Make persistent
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_deadzone(&mut self, axis: DeviceAxis, threshold: i32) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_deadzone(axis as i32, threshold) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get current deadzone threshold for an axis
    pub fn get_deadzone(&self, axis: DeviceAxis) -> Result<i32> {
        unsafe {
            let value = ffi::spnav_cfg_get_deadzone(axis as i32);
            if value == -1 {
                return Err(Error::ConfigFailed);
            }
            Ok(value)
        }
    }

    /// Get all deadzone thresholds as an array [TX, TY, TZ, RX, RY, RZ]
    pub fn get_all_deadzones(&self) -> Result<[i32; 6]> {
        let mut deadzones = [0i32; 6];
        for (i, axis) in DeviceAxis::all().iter().enumerate() {
            deadzones[i] = self.get_deadzone(*axis)?;
        }
        Ok(deadzones)
    }

    /// Set all deadzone thresholds at once
    pub fn set_all_deadzones(&mut self, deadzones: [i32; 6]) -> Result<()> {
        for (i, &threshold) in deadzones.iter().enumerate() {
            let axis = DeviceAxis::from_index(i).unwrap();
            self.set_deadzone(axis, threshold)?;
        }
        Ok(())
    }

    /// Set global sensitivity multiplier for all axes
    ///
    /// This is a convenient way to scale all motion equally.
    /// Values > 1.0 increase sensitivity, < 1.0 decrease it.
    ///
    /// # Arguments
    ///
    /// * `sensitivity` - Global multiplier (typically 0.1 to 10.0, 1.0 = normal)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::SpaceNav;
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    /// cfg.set_global_sensitivity(1.5)?;  // 50% more sensitive
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_global_sensitivity(&mut self, sensitivity: f32) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_sens(sensitivity) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get global sensitivity multiplier
    pub fn get_global_sensitivity(&self) -> Result<f32> {
        unsafe {
            let value = ffi::spnav_cfg_get_sens();
            if value < 0.0 {
                return Err(Error::ConfigFailed);
            }
            Ok(value)
        }
    }

    /// Set per-axis sensitivity multipliers for all 6 input axes
    ///
    /// Array order: [TX, TY, TZ, RX, RY, RZ]
    ///
    /// Each value is a multiplier: 1.0 = default, > 1.0 = more sensitive, < 1.0 = less sensitive
    ///
    /// # Arguments
    ///
    /// * `sensitivities` - Array of 6 multipliers, one per input axis
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::SpaceNav;
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    ///
    /// // Make rotation twice as sensitive as translation
    /// cfg.set_axis_sensitivities([1.0, 1.0, 1.0, 2.0, 2.0, 2.0])?;
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_axis_sensitivities(&mut self, sensitivities: [f32; 6]) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_axis_sens(sensitivities.as_ptr()) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get per-axis sensitivity multipliers for all 6 input axes
    ///
    /// Returns array in order: [TX, TY, TZ, RX, RY, RZ]
    pub fn get_axis_sensitivities(&self) -> Result<[f32; 6]> {
        let mut sensitivities = [0.0f32; 6];
        unsafe {
            if ffi::spnav_cfg_get_axis_sens(sensitivities.as_mut_ptr()) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(sensitivities)
    }

    /// Set sensitivity for a specific input axis
    ///
    /// This is a convenience wrapper around `set_axis_sensitivities` that
    /// preserves other axes' values.
    ///
    /// # Arguments
    ///
    /// * `axis` - Input axis to modify (TX, TY, TZ, RX, RY, RZ)
    /// * `sensitivity` - Multiplier for this axis
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::{SpaceNav, config::InputAxis};
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    /// cfg.set_input_axis_sensitivity(InputAxis::RZ, 2.0)?;
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_input_axis_sensitivity(&mut self, axis: InputAxis, sensitivity: f32) -> Result<()> {
        let mut sensitivities = self.get_axis_sensitivities()?;
        sensitivities[axis as usize] = sensitivity;
        self.set_axis_sensitivities(sensitivities)
    }

    /// Get sensitivity for a specific input axis
    pub fn get_input_axis_sensitivity(&self, axis: InputAxis) -> Result<f32> {
        let sensitivities = self.get_axis_sensitivities()?;
        Ok(sensitivities[axis as usize])
    }

    /// Set axis sensitivities using a builder
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::{SpaceNav, config::Sensitivity};
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    ///
    /// let sens = Sensitivity::new()
    ///     .with_translation(0.8)
    ///     .with_rotation(1.5)
    ///     .with_rz(2.0);  // Extra sensitive Z rotation
    ///
    /// cfg.set_sensitivities(sens)?;
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_sensitivities(&mut self, sensitivity: Sensitivity) -> Result<()> {
        self.set_axis_sensitivities(sensitivity.as_array())
    }

    /// Get axis sensitivities as a structured type
    pub fn get_sensitivities(&self) -> Result<Sensitivity> {
        let values = self.get_axis_sensitivities()?;
        Ok(Sensitivity::from_array(values))
    }

    /// Set axis inversion using a bitmask
    ///
    /// Each bit corresponds to an input axis (after mapping):
    /// - Bit 0: TX (Translation X)
    /// - Bit 1: TY (Translation Y)
    /// - Bit 2: TZ (Translation Z)
    /// - Bit 3: RX (Rotation X)
    /// - Bit 4: RY (Rotation Y)
    /// - Bit 5: RZ (Rotation Z)
    ///
    /// # Arguments
    ///
    /// * `mask` - Bitmask of axes to invert (bit set = inverted)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::SpaceNav;
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    ///
    /// // Invert Y and Z translation axes (bits 1 and 2)
    /// cfg.set_invert_raw(0b000110)?;
    ///
    /// // Or use the builder:
    /// cfg.set_invert(AxisInversion::new()
    ///     .with_ty(true)
    ///     .with_tz(true))?;
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_invert_raw(&mut self, mask: u32) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_invert(mask as i32) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get current axis inversion bitmask
    pub fn get_invert_raw(&self) -> Result<u32> {
        unsafe {
            let value = ffi::spnav_cfg_get_invert();
            if value == -1 {
                return Err(Error::ConfigFailed);
            }
            Ok(value as u32)
        }
    }

    /// Set axis inversions using a builder
    pub fn set_invert(&mut self, inversion: AxisInversion) -> Result<()> {
        self.set_invert_raw(inversion.to_mask())
    }

    /// Get current axis inversions as a structured type
    pub fn get_invert(&self) -> Result<AxisInversion> {
        let mask = self.get_invert_raw()?;
        Ok(AxisInversion::from_mask(mask))
    }

    /// Map a device button to an input button
    ///
    /// # Arguments
    ///
    /// * `device_button` - Physical button number on device
    /// * `input_button` - Logical button number to map to
    pub fn set_button_map(&mut self, device_button: u32, input_button: u32) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_bnmap(device_button as i32, input_button as i32) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get the current input button mapping for a device button
    pub fn get_button_map(&self, device_button: u32) -> Result<u32> {
        unsafe {
            let value = ffi::spnav_cfg_get_bnmap(device_button as i32);
            if value == -1 {
                return Err(Error::ConfigFailed);
            }
            Ok(value as u32)
        }
    }

    /// Set LED state
    pub fn set_led(&mut self, state: LedState) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_led(state as i32) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get current LED state
    pub fn get_led(&self) -> Result<LedState> {
        unsafe {
            let value = ffi::spnav_cfg_get_led();
            if value == -1 {
                return Err(Error::ConfigFailed);
            }
            Ok(LedState::from(value))
        }
    }

    /// Set event grab mode
    ///
    /// Exclusive grab prevents other applications from receiving events.
    pub fn set_grab(&mut self, mode: GrabMode) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_grab(mode as i32) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get current grab mode
    pub fn get_grab(&self) -> Result<GrabMode> {
        unsafe {
            let value = ffi::spnav_cfg_get_grab();
            if value == -1 {
                return Err(Error::ConfigFailed);
            }
            Ok(GrabMode::from(value))
        }
    }

    /// Enable or disable swapping of Y and Z axes
    ///
    /// When enabled, both translation and rotation Y and Z axes are swapped.
    /// This is useful for devices that have different axis conventions.
    ///
    /// # Arguments
    ///
    /// * `swap` - true to swap Y/Z axes, false for normal operation
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::SpaceNav;
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    /// cfg.set_swap_yz(true)?;  // Swap Y and Z
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_swap_yz(&mut self, swap: bool) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_swapyz(swap as i32) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Check if Y and Z axes are currently swapped
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Y and Z axes are swapped
    /// * `Ok(false)` - Y and Z axes are normal
    /// * `Err(_)` - Query failed
    pub fn get_swap_yz(&self) -> Result<bool> {
        unsafe {
            let value = ffi::spnav_cfg_get_swapyz();
            match value {
                -1 => Err(Error::ConfigFailed),
                0 => Ok(false),
                _ => Ok(true),
            }
        }
    }

    /// Set serial device path
    ///
    /// Only needed for serial SpaceMouse devices.
    pub fn set_serial(&mut self, path: &str) -> Result<()> {
        let c_path = CString::new(path).map_err(|_| Error::InvalidString)?;
        unsafe {
            if ffi::spnav_cfg_set_serial(c_path.as_ptr()) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get current serial device path
    pub fn get_serial(&self) -> Result<String> {
        const BUF_SIZE: usize = 256;
        let mut buf = vec![0u8; BUF_SIZE];

        unsafe {
            let len = ffi::spnav_cfg_get_serial(buf.as_mut_ptr() as *mut i8, BUF_SIZE as i32);
            if len == -1 {
                return Err(Error::ConfigFailed);
            }

            let c_str = CStr::from_ptr(buf.as_ptr() as *const i8);
            Ok(c_str.to_string_lossy().into_owned())
        }
    }

    /// Map a device button to a special action
    ///
    /// This allows buttons to trigger special functions instead of regular
    /// button events. For example, mapping a button to temporarily disable
    /// rotation while held, or to reset sensitivity.
    ///
    /// # Arguments
    ///
    /// * `button` - Device button number (0 to `device_buttons()` - 1)
    /// * `action` - Action to map to this button
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::{SpaceNav, config::ButtonAction};
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    ///
    /// // Button 0 disables rotation while held (useful for panning)
    /// cfg.set_button_action(0, ButtonAction::DisableRotation)?;
    ///
    /// // Button 1 resets sensitivity to default
    /// cfg.set_button_action(1, ButtonAction::SensitivityReset)?;
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_button_action(&mut self, button: u32, action: ButtonAction) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_set_bnaction(button as i32, action.to_raw()) == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Get the action currently mapped to a device button
    ///
    /// # Arguments
    ///
    /// * `button` - Device button number (0 to `device_buttons()` - 1)
    ///
    /// # Returns
    ///
    /// The action mapped to this button, or an error if the query failed.
    pub fn get_button_action(&self, button: u32) -> Result<ButtonAction> {
        unsafe {
            let value = ffi::spnav_cfg_get_bnaction(button as i32);
            if value == -1 {
                return Err(Error::ConfigFailed);
            }
            ButtonAction::from_raw(value).ok_or(Error::UnknownButtonAction(value))
        }
    }

    /// Clear button action mapping (return to regular button)
    ///
    /// Convenience method equivalent to `set_button_action(button, ButtonAction::None)`
    pub fn clear_button_action(&mut self, button: u32) -> Result<()> {
        self.set_button_action(button, ButtonAction::None)
    }

    /// Get all button action mappings
    ///
    /// Returns a map of button numbers to their actions.
    /// Only buttons with non-None actions are included.
    ///
    /// # Note
    ///
    /// This requires querying the device for button count.
    pub fn get_all_button_actions(&self) -> Result<std::collections::HashMap<u32, ButtonAction>> {
        use std::collections::HashMap;

        // Get button count from device info
        let button_count = unsafe {
            let count = ffi::spnav_dev_buttons();
            if count == -1 {
                return Err(Error::QueryFailed);
            }
            count as u32
        };

        let mut actions = HashMap::new();
        for button in 0..button_count {
            let action = self.get_button_action(button)?;
            if action != ButtonAction::None {
                actions.insert(button, action);
            }
        }

        Ok(actions)
    }

    /// Set multiple button actions at once
    ///
    /// # Arguments
    ///
    /// * `mappings` - Iterator of (button_number, action) pairs
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::{SpaceNav, config::ButtonAction};
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    ///
    /// let mappings = [
    ///     (0, ButtonAction::DisableRotation),
    ///     (1, ButtonAction::DisableTranslation),
    ///     (2, ButtonAction::SensitivityReset),
    /// ];
    ///
    /// cfg.set_button_actions(mappings)?;
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn set_button_actions<I>(&mut self, mappings: I) -> Result<()>
    where
        I: IntoIterator<Item = (u32, ButtonAction)>,
    {
        for (button, action) in mappings {
            self.set_button_action(button, action)?;
        }
        Ok(())
    }

    /// Save current configuration to the spacenavd config file
    ///
    /// This makes all configuration changes persistent across daemon restarts.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spnav::{SpaceNav, config::DeviceAxis};
    /// # let mut spacenav = SpaceNav::connect()?;
    /// let mut cfg = spacenav.config();
    /// cfg.set_deadzone(DeviceAxis::TranslationX, 10)?;
    /// cfg.set_input_axis_sensitivity(InputAxis::RZ, 2.0)?;
    /// cfg.save()?;  // Write to /etc/spnavrc
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn save(&mut self) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_save() == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Restore configuration from the config file
    ///
    /// Reverts all runtime changes to the values defined in the config file.
    pub fn restore(&mut self) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_restore() == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }

    /// Reset all settings to default values
    pub fn reset(&mut self) -> Result<()> {
        unsafe {
            if ffi::spnav_cfg_reset() == -1 {
                return Err(Error::ConfigFailed);
            }
        }
        Ok(())
    }
}
