//! # Input Engine
//!
//! Universal tablet/stylus input abstraction supporting:
//! - libinput for Wacom, Huion, XP-Pen, Gaomon tablets
//! - HID raw fallback for unrecognised devices
//! - Pressure normalisation (0.0–1.0)
//! - Tilt (X/Y), rotation, azimuth
//! - Palm rejection with configurable exclusion zones
//! - Barrel button configurable actions
//! - Hover detection (cursor before contact)

mod device;
mod events;
mod palm_rejection;
mod pressure;
#[cfg(all(target_os = "linux", feature = "libinput"))]
mod libinput_backend;
mod hid_fallback;

pub use device::*;
pub use events::*;
pub use palm_rejection::*;
pub use pressure::*;

use thiserror::Error;

/// Errors that can occur in the input engine
#[derive(Error, Debug)]
pub enum InputError {
    #[error("Failed to initialize input backend: {0}")]
    InitFailed(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Permission denied accessing input device: {0}")]
    PermissionDenied(String),

    #[error("Unsupported device: {0}")]
    UnsupportedDevice(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Input backend error: {0}")]
    BackendError(String),
}

/// Input backend trait — abstracts over libinput and HID raw
pub trait InputBackend: Send {
    /// Initialize the input backend
    fn init(&mut self) -> Result<(), InputError>;

    /// Poll for new input events (non-blocking)
    fn poll_events(&mut self) -> Result<Vec<InputEvent>, InputError>;

    /// Get list of connected input devices
    fn devices(&self) -> &[InputDevice];

    /// Check if a specific device is connected
    fn is_device_connected(&self, device_id: &str) -> bool;

    /// Shutdown the input backend
    fn shutdown(&mut self);
}

/// Configuration for the input engine
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputConfig {
    /// Enable palm rejection
    pub palm_rejection_enabled: bool,

    /// Palm rejection exclusion zones (normalized 0.0-1.0 coordinates)
    pub palm_rejection_zones: Vec<ExclusionZone>,

    /// Barrel button action mapping
    pub barrel_button_actions: std::collections::HashMap<PenButton, BarrelAction>,

    /// Pressure curve adjustment (gamma)
    pub pressure_curve_gamma: f32,

    /// Minimum pressure threshold to register a stroke
    pub pressure_threshold: f32,

    /// Enable hover detection
    pub hover_detection: bool,

    /// Enable stylus-only mode (disable touch input)
    pub stylus_only_mode: bool,

    /// Predictive ink lookahead frames
    pub predictive_lookahead_frames: u32,
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut barrel_actions = std::collections::HashMap::new();
        barrel_actions.insert(PenButton::Primary, BarrelAction::Eraser);
        barrel_actions.insert(PenButton::Secondary, BarrelAction::RightClick);

        Self {
            palm_rejection_enabled: true,
            palm_rejection_zones: vec![],
            barrel_button_actions: barrel_actions,
            pressure_curve_gamma: 1.0,
            pressure_threshold: 0.01,
            hover_detection: true,
            stylus_only_mode: false,
            predictive_lookahead_frames: 2,
        }
    }
}

/// Action to perform when a barrel button is pressed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BarrelAction {
    /// Switch to eraser tool
    Eraser,
    /// Right-click context menu
    RightClick,
    /// Undo last stroke
    Undo,
    /// Redo last stroke
    Redo,
    /// Toggle between pen and highlighter
    ToggleHighlighter,
    /// Color picker (sample color from canvas)
    ColorPicker,
    /// Pan canvas
    PanCanvas,
    /// Custom action (plugin-defined)
    Custom(u32),
    /// No action
    None,
}

/// The main input engine that processes raw device input
pub struct InputEngine {
    config: InputConfig,
    palm_filter: PalmRejectionFilter,
    pressure_normalizer: PressureNormalizer,
    active_devices: Vec<InputDevice>,
    stylus_state: StylusState,
}

impl InputEngine {
    /// Create a new input engine with the given configuration
    pub fn new(config: InputConfig) -> Self {
        let palm_filter = PalmRejectionFilter::new(
            config.palm_rejection_enabled,
            config.palm_rejection_zones.clone(),
        );
        let pressure_normalizer = PressureNormalizer::new(
            config.pressure_curve_gamma,
            config.pressure_threshold,
        );

        Self {
            config,
            palm_filter,
            pressure_normalizer,
            active_devices: Vec::new(),
            stylus_state: StylusState::default(),
        }
    }

    /// Process a raw input event and return a normalized event
    pub fn process_event(&mut self, raw_event: RawInputEvent) -> Option<InputEvent> {
        // Apply palm rejection
        if self.palm_filter.should_reject(&raw_event) {
            return None;
        }

        // Stylus-only mode check
        if self.config.stylus_only_mode && raw_event.source == InputSource::Touch {
            return None;
        }

        // Normalize pressure
        let pressure = self
            .pressure_normalizer
            .normalize(raw_event.pressure_raw);

        // Build normalized event
        let event = InputEvent {
            device_id: raw_event.device_id,
            event_type: raw_event.event_type,
            x: raw_event.x,
            y: raw_event.y,
            pressure,
            tilt_x: raw_event.tilt_x.unwrap_or(0.0),
            tilt_y: raw_event.tilt_y.unwrap_or(0.0),
            rotation: raw_event.rotation.unwrap_or(0.0),
            azimuth: raw_event.azimuth.unwrap_or(0.0),
            timestamp_us: raw_event.timestamp_us,
            button: raw_event.button,
            source: raw_event.source,
        };

        // Update stylus state
        self.update_stylus_state(&event);

        Some(event)
    }

    /// Get the current stylus state
    pub fn stylus_state(&self) -> &StylusState {
        &self.stylus_state
    }

    /// Get the barrel action for a button press
    pub fn barrel_action(&self, button: PenButton) -> BarrelAction {
        self.config
            .barrel_button_actions
            .get(&button)
            .copied()
            .unwrap_or(BarrelAction::None)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: InputConfig) {
        self.palm_filter = PalmRejectionFilter::new(
            config.palm_rejection_enabled,
            config.palm_rejection_zones.clone(),
        );
        self.pressure_normalizer = PressureNormalizer::new(
            config.pressure_curve_gamma,
            config.pressure_threshold,
        );
        self.config = config;
    }

    /// Register a new device
    pub fn register_device(&mut self, device: InputDevice) {
        if !self.active_devices.iter().any(|d| d.id == device.id) {
            log::info!("Registered input device: {} ({})", device.name, device.id);
            self.active_devices.push(device);
        }
    }

    /// Unregister a device
    pub fn unregister_device(&mut self, device_id: &str) {
        self.active_devices.retain(|d| d.id != device_id);
        log::info!("Unregistered input device: {}", device_id);
    }

    /// Get all active devices
    pub fn active_devices(&self) -> &[InputDevice] {
        &self.active_devices
    }

    fn update_stylus_state(&mut self, event: &InputEvent) {
        match event.event_type {
            InputEventType::ProximityIn => {
                self.stylus_state.in_proximity = true;
                self.stylus_state.in_contact = false;
            }
            InputEventType::ProximityOut => {
                self.stylus_state.in_proximity = false;
                self.stylus_state.in_contact = false;
            }
            InputEventType::StylusDown => {
                self.stylus_state.in_contact = true;
            }
            InputEventType::StylusUp => {
                self.stylus_state.in_contact = false;
            }
            InputEventType::StylusMotion => {
                self.stylus_state.x = event.x;
                self.stylus_state.y = event.y;
                self.stylus_state.pressure = event.pressure;
                self.stylus_state.tilt_x = event.tilt_x;
                self.stylus_state.tilt_y = event.tilt_y;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_engine_creation() {
        let config = InputConfig::default();
        let engine = InputEngine::new(config);
        assert!(engine.active_devices().is_empty());
        assert!(!engine.stylus_state().in_proximity);
    }

    #[test]
    fn test_palm_rejection() {
        let config = InputConfig {
            palm_rejection_enabled: true,
            stylus_only_mode: true,
            ..Default::default()
        };
        let mut engine = InputEngine::new(config);

        // Touch events should be rejected in stylus-only mode
        let touch_event = RawInputEvent {
            device_id: "test".to_string(),
            event_type: InputEventType::StylusMotion,
            x: 100.0,
            y: 100.0,
            pressure_raw: 0.5,
            tilt_x: None,
            tilt_y: None,
            rotation: None,
            azimuth: None,
            timestamp_us: 0,
            button: None,
            source: InputSource::Touch,
        };

        assert!(engine.process_event(touch_event).is_none());
    }

    #[test]
    fn test_barrel_action_mapping() {
        let config = InputConfig::default();
        let engine = InputEngine::new(config);
        assert_eq!(engine.barrel_action(PenButton::Primary), BarrelAction::Eraser);
        assert_eq!(engine.barrel_action(PenButton::Secondary), BarrelAction::RightClick);
    }

    #[test]
    fn test_device_registration() {
        let config = InputConfig::default();
        let mut engine = InputEngine::new(config);

        let device = InputDevice {
            id: "wacom-001".to_string(),
            name: "Wacom Intuos Pro".to_string(),
            vendor_id: 0x056a,
            product_id: 0x0357,
            device_type: DeviceType::Tablet,
            capabilities: DeviceCapabilities {
                has_pressure: true,
                has_tilt: true,
                has_rotation: true,
                has_buttons: true,
                pressure_range: (0, 8192),
                tilt_range: (-60.0, 60.0),
                max_resolution: (5080, 5080),
            },
        };

        engine.register_device(device);
        assert_eq!(engine.active_devices().len(), 1);
        assert_eq!(engine.active_devices()[0].name, "Wacom Intuos Pro");
    }
}
