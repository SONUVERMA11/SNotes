//! HID raw fallback for unrecognised tablet devices
//!
//! When libinput/libwacom doesn't recognise a device, this backend
//! reads raw HID reports via udev to provide basic stylus functionality.

use super::{InputBackend, InputDevice, InputError, InputEvent};

/// HID raw fallback input backend
pub struct HidFallbackBackend {
    devices: Vec<InputDevice>,
    initialized: bool,
}

impl HidFallbackBackend {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            initialized: false,
        }
    }
}

impl InputBackend for HidFallbackBackend {
    fn init(&mut self) -> Result<(), InputError> {
        log::info!("Initializing HID raw fallback backend");
        // Scan /dev/hidraw* for tablet-like devices
        self.initialized = true;
        Ok(())
    }

    fn poll_events(&mut self) -> Result<Vec<InputEvent>, InputError> {
        if !self.initialized {
            return Err(InputError::InitFailed("Not initialized".to_string()));
        }
        Ok(Vec::new())
    }

    fn devices(&self) -> &[InputDevice] {
        &self.devices
    }

    fn is_device_connected(&self, device_id: &str) -> bool {
        self.devices.iter().any(|d| d.id == device_id)
    }

    fn shutdown(&mut self) {
        log::info!("Shutting down HID fallback backend");
        self.initialized = false;
        self.devices.clear();
    }
}
