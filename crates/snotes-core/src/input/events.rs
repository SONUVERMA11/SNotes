//! Input event types and data structures

use serde::{Deserialize, Serialize};

/// Source of an input event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSource {
    Stylus,
    Touch,
    Mouse,
    Keyboard,
}

/// Type of input event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputEventType {
    ProximityIn,
    ProximityOut,
    StylusDown,
    StylusUp,
    StylusMotion,
    ButtonPress,
    ButtonRelease,
    TouchDown,
    TouchUp,
    TouchMotion,
    PinchGesture,
    ScrollGesture,
}

/// Pen/stylus button identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PenButton {
    Primary,
    Secondary,
    Tertiary,
    Eraser,
}

/// Normalized input event from any device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub device_id: String,
    pub event_type: InputEventType,
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub rotation: f32,
    pub azimuth: f32,
    pub timestamp_us: u64,
    pub button: Option<PenButton>,
    pub source: InputSource,
}

/// Raw input event before normalization
#[derive(Debug, Clone)]
pub struct RawInputEvent {
    pub device_id: String,
    pub event_type: InputEventType,
    pub x: f64,
    pub y: f64,
    pub pressure_raw: f32,
    pub tilt_x: Option<f32>,
    pub tilt_y: Option<f32>,
    pub rotation: Option<f32>,
    pub azimuth: Option<f32>,
    pub timestamp_us: u64,
    pub button: Option<PenButton>,
    pub source: InputSource,
}

/// Current state of the stylus
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StylusState {
    pub in_proximity: bool,
    pub in_contact: bool,
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub rotation: f32,
}

impl InputEvent {
    pub fn is_stroke_start(&self) -> bool {
        self.event_type == InputEventType::StylusDown
    }

    pub fn is_stroke_end(&self) -> bool {
        self.event_type == InputEventType::StylusUp
    }

    pub fn is_stroke_motion(&self) -> bool {
        self.event_type == InputEventType::StylusMotion
    }

    pub fn is_hover(&self) -> bool {
        self.event_type == InputEventType::StylusMotion && self.pressure <= 0.0
    }

    pub fn to_stroke_point(&self) -> crate::ink::StrokePoint {
        crate::ink::StrokePoint {
            x: self.x,
            y: self.y,
            pressure: self.pressure,
            tilt_x: self.tilt_x,
            tilt_y: self.tilt_y,
            timestamp_us: self.timestamp_us,
        }
    }
}
