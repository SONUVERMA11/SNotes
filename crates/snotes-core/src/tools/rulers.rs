//! Ruler and protractor tools for precise measurement

use serde::{Deserialize, Serialize};

/// Ruler tool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruler {
    pub visible: bool,
    pub start: (f64, f64),
    pub end: (f64, f64),
    pub locked: bool,
}

impl Ruler {
    pub fn new() -> Self {
        Self { visible: false, start: (100.0, 500.0), end: (800.0, 500.0), locked: false }
    }

    /// Get the length of the ruler in pixels
    pub fn length(&self) -> f64 {
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Get the angle of the ruler in degrees
    pub fn angle(&self) -> f64 {
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        dy.atan2(dx).to_degrees()
    }

    /// Snap a point to the ruler's line
    pub fn snap_to_line(&self, x: f64, y: f64, threshold: f64) -> Option<(f64, f64)> {
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        let len_sq = dx * dx + dy * dy;
        if len_sq < 1e-10 { return None; }

        let t = ((x - self.start.0) * dx + (y - self.start.1) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);
        let proj_x = self.start.0 + t * dx;
        let proj_y = self.start.1 + t * dy;

        let dist = ((x - proj_x).powi(2) + (y - proj_y).powi(2)).sqrt();
        if dist <= threshold { Some((proj_x, proj_y)) } else { None }
    }
}

impl Default for Ruler {
    fn default() -> Self { Self::new() }
}

/// Protractor tool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protractor {
    pub visible: bool,
    pub center: (f64, f64),
    pub radius: f64,
    pub rotation: f64,
}

impl Protractor {
    pub fn new() -> Self {
        Self { visible: false, center: (500.0, 500.0), radius: 200.0, rotation: 0.0 }
    }

    /// Get the angle of a point relative to the protractor center
    pub fn angle_at(&self, x: f64, y: f64) -> f64 {
        let dx = x - self.center.0;
        let dy = y - self.center.1;
        let angle = dy.atan2(dx).to_degrees() - self.rotation;
        if angle < 0.0 { angle + 360.0 } else { angle }
    }

    /// Snap a point to the protractor's edge
    pub fn snap_to_edge(&self, x: f64, y: f64, threshold: f64) -> Option<(f64, f64)> {
        let dx = x - self.center.0;
        let dy = y - self.center.1;
        let dist = (dx * dx + dy * dy).sqrt();
        if (dist - self.radius).abs() <= threshold {
            let angle = dy.atan2(dx);
            Some((self.center.0 + self.radius * angle.cos(), self.center.1 + self.radius * angle.sin()))
        } else {
            None
        }
    }
}

impl Default for Protractor {
    fn default() -> Self { Self::new() }
}
