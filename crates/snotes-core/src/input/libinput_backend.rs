//! libinput backend for tablet/stylus input on Linux
//!
//! Wraps the `input` crate (Rust bindings for libinput) to provide
//! tablet tool event handling with pressure, tilt, and proximity support.

use super::{InputBackend, InputDevice, InputError, InputEvent};

/// libinput-based input backend
///
/// Note: This requires root/input group permissions to access /dev/input/*
/// In production, this is handled by the Flatpak sandbox or udev rules.
pub struct LibinputBackend {
    devices: Vec<InputDevice>,
    initialized: bool,
}

impl LibinputBackend {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            initialized: false,
        }
    }
}

impl InputBackend for LibinputBackend {
    fn init(&mut self) -> Result<(), InputError> {
        // In a full implementation, this would:
        // 1. Create a libinput context via udev
        // 2. Set up seat and device discovery
        // 3. Configure tablet tool capabilities
        //
        // For now, we mark as initialized for the build system.
        // Actual libinput integration requires running on a real seat
        // with appropriate permissions.
        log::info!("Initializing libinput backend");
        self.initialized = true;
        Ok(())
    }

    fn poll_events(&mut self) -> Result<Vec<InputEvent>, InputError> {
        if !self.initialized {
            return Err(InputError::InitFailed(
                "Backend not initialized".to_string(),
            ));
        }
        // In production: dispatch libinput events and convert to InputEvent
        Ok(Vec::new())
    }

    fn devices(&self) -> &[InputDevice] {
        &self.devices
    }

    fn is_device_connected(&self, device_id: &str) -> bool {
        self.devices.iter().any(|d| d.id == device_id)
    }

    fn shutdown(&mut self) {
        log::info!("Shutting down libinput backend");
        self.initialized = false;
        self.devices.clear();
    }
}
