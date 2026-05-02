//! Palm rejection with configurable exclusion zones

use super::events::RawInputEvent;
use serde::{Deserialize, Serialize};

/// Rectangular exclusion zone for palm rejection (normalized 0.0–1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionZone {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
    pub label: String,
}

impl ExclusionZone {
    /// Create a right-edge palm rest zone
    pub fn right_palm(width_fraction: f64) -> Self {
        Self {
            x_min: 1.0 - width_fraction,
            y_min: 0.0,
            x_max: 1.0,
            y_max: 1.0,
            label: "Right palm rest".to_string(),
        }
    }

    /// Create a left-edge palm rest zone
    pub fn left_palm(width_fraction: f64) -> Self {
        Self {
            x_min: 0.0,
            y_min: 0.0,
            x_max: width_fraction,
            y_max: 1.0,
            label: "Left palm rest".to_string(),
        }
    }

    /// Create a bottom-edge palm rest zone
    pub fn bottom_palm(height_fraction: f64) -> Self {
        Self {
            x_min: 0.0,
            y_min: 1.0 - height_fraction,
            x_max: 1.0,
            y_max: 1.0,
            label: "Bottom palm rest".to_string(),
        }
    }

    /// Check if a point falls within this exclusion zone
    pub fn contains(&self, x_norm: f64, y_norm: f64) -> bool {
        x_norm >= self.x_min && x_norm <= self.x_max
            && y_norm >= self.y_min && y_norm <= self.y_max
    }
}

/// Palm rejection filter
pub struct PalmRejectionFilter {
    enabled: bool,
    zones: Vec<ExclusionZone>,
    canvas_width: f64,
    canvas_height: f64,
    /// Tracks active touch IDs that started in exclusion zones
    rejected_touches: Vec<String>,
}

impl PalmRejectionFilter {
    pub fn new(enabled: bool, zones: Vec<ExclusionZone>) -> Self {
        Self {
            enabled,
            zones,
            canvas_width: 1920.0,
            canvas_height: 1080.0,
            rejected_touches: Vec::new(),
        }
    }

    /// Update canvas dimensions for coordinate normalization
    pub fn set_canvas_size(&mut self, width: f64, height: f64) {
        self.canvas_width = width;
        self.canvas_height = height;
    }

    /// Check if a raw input event should be rejected
    pub fn should_reject(&mut self, event: &RawInputEvent) -> bool {
        if !self.enabled {
            return false;
        }

        // Only apply palm rejection to touch events
        if event.source != super::events::InputSource::Touch {
            return false;
        }

        // Normalize coordinates
        let x_norm = event.x / self.canvas_width;
        let y_norm = event.y / self.canvas_height;

        // Check if point is in any exclusion zone
        let in_zone = self.zones.iter().any(|z| z.contains(x_norm, y_norm));

        if in_zone {
            // Track this touch as rejected
            if !self.rejected_touches.contains(&event.device_id) {
                self.rejected_touches.push(event.device_id.clone());
            }
            return true;
        }

        // If this touch started in an exclusion zone, keep rejecting it
        if self.rejected_touches.contains(&event.device_id) {
            // Clean up on touch up
            if event.event_type == super::events::InputEventType::TouchUp {
                self.rejected_touches.retain(|id| id != &event.device_id);
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclusion_zone_contains() {
        let zone = ExclusionZone::right_palm(0.2);
        assert!(zone.contains(0.9, 0.5));
        assert!(!zone.contains(0.5, 0.5));
    }

    #[test]
    fn test_palm_rejection_disabled() {
        let mut filter = PalmRejectionFilter::new(false, vec![]);
        let event = super::super::events::RawInputEvent {
            device_id: "touch-1".to_string(),
            event_type: super::super::events::InputEventType::TouchMotion,
            x: 1800.0, y: 500.0,
            pressure_raw: 0.5,
            tilt_x: None, tilt_y: None, rotation: None, azimuth: None,
            timestamp_us: 0,
            button: None,
            source: super::super::events::InputSource::Touch,
        };
        assert!(!filter.should_reject(&event));
    }
}
