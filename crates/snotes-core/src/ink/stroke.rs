//! Stroke data model

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single point in a stroke with all metadata
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrokePoint {
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub timestamp_us: u64,
}

impl StrokePoint {
    /// Calculate distance to another point
    pub fn distance_to(&self, other: &StrokePoint) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate velocity between two points (pixels per microsecond)
    pub fn velocity_to(&self, other: &StrokePoint) -> f64 {
        let dist = self.distance_to(other);
        let dt = (other.timestamp_us as f64 - self.timestamp_us as f64).abs();
        if dt > 0.0 { dist / dt } else { 0.0 }
    }

    /// Linearly interpolate between two points
    pub fn lerp(&self, other: &StrokePoint, t: f64) -> StrokePoint {
        StrokePoint {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            pressure: self.pressure + (other.pressure - self.pressure) * t as f32,
            tilt_x: self.tilt_x + (other.tilt_x - self.tilt_x) * t as f32,
            tilt_y: self.tilt_y + (other.tilt_y - self.tilt_y) * t as f32,
            timestamp_us: (self.timestamp_us as f64
                + (other.timestamp_us as f64 - self.timestamp_us as f64) * t)
                as u64,
        }
    }
}

/// RGBA color
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW_HIGHLIGHT: Color = Color { r: 1.0, g: 1.0, b: 0.0, a: 0.4 };

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() < 6 { return None; }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        let a = if hex.len() >= 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0
        } else {
            1.0
        };
        Some(Self { r, g, b, a })
    }

    pub fn to_hex(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }
}

/// Tool type for strokes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    Pen,
    Brush,
    Pencil,
    Marker,
    Highlighter,
    Eraser,
}

/// Eraser mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EraserMode {
    /// Delete entire strokes that are touched
    WholeStroke,
    /// Pixel-level erase (splits strokes)
    PixelErase,
}

/// A complete stroke on the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub id: Uuid,
    pub tool: ToolType,
    pub color: Color,
    pub base_width: f32,
    pub points: Vec<StrokePoint>,
    pub layer_id: Uuid,
    pub timestamp: i64,
    /// Bounding box for spatial indexing (min_x, min_y, max_x, max_y)
    pub bounds: (f64, f64, f64, f64),
}

impl Stroke {
    /// Create a new empty stroke
    pub fn new(tool: ToolType, color: Color, base_width: f32, layer_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool,
            color,
            base_width,
            points: Vec::new(),
            layer_id,
            timestamp: chrono_timestamp(),
            bounds: (f64::MAX, f64::MAX, f64::MIN, f64::MIN),
        }
    }

    /// Add a point and update bounding box
    pub fn add_point(&mut self, point: StrokePoint) {
        self.bounds.0 = self.bounds.0.min(point.x);
        self.bounds.1 = self.bounds.1.min(point.y);
        self.bounds.2 = self.bounds.2.max(point.x);
        self.bounds.3 = self.bounds.3.max(point.y);
        self.points.push(point);
    }

    /// Check if a point is near this stroke (for eraser hit testing)
    pub fn hit_test(&self, x: f64, y: f64, radius: f64) -> bool {
        // Quick bounding box check
        if x < self.bounds.0 - radius || x > self.bounds.2 + radius
            || y < self.bounds.1 - radius || y > self.bounds.3 + radius
        {
            return false;
        }
        // Detailed per-segment check
        for window in self.points.windows(2) {
            let dist = point_to_segment_distance(
                x, y,
                window[0].x, window[0].y,
                window[1].x, window[1].y,
            );
            if dist <= radius + self.base_width as f64 / 2.0 {
                return true;
            }
        }
        false
    }

    /// Calculate the total length of the stroke
    pub fn length(&self) -> f64 {
        self.points
            .windows(2)
            .map(|w| w[0].distance_to(&w[1]))
            .sum()
    }

    /// Get the stroke width at a given point index
    pub fn width_at(&self, index: usize) -> f32 {
        if index >= self.points.len() {
            return self.base_width;
        }
        self.width_at_pressure(self.points[index].pressure)
    }

    /// Get the stroke width for a given pressure value (0.0–1.0)
    pub fn width_at_pressure(&self, pressure: f32) -> f32 {
        match self.tool {
            ToolType::Pen => self.base_width * (0.3 + 0.7 * pressure),
            ToolType::Brush => self.base_width * (0.1 + 0.9 * pressure),
            ToolType::Pencil => self.base_width * (0.5 + 0.5 * pressure),
            ToolType::Marker => self.base_width,
            ToolType::Highlighter => self.base_width * 3.0,
            ToolType::Eraser => self.base_width * 2.0,
        }
    }
}

/// Distance from point (px, py) to line segment (x1,y1)-(x2,y2)
fn point_to_segment_distance(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-10 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stroke_point_distance() {
        let a = StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 };
        let b = StrokePoint { x: 3.0, y: 4.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 };
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_color_hex() {
        let c = Color::from_hex("#ff0000").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_stroke_hit_test() {
        let mut stroke = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
        stroke.add_point(StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 });
        stroke.add_point(StrokePoint { x: 100.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 });
        assert!(stroke.hit_test(50.0, 0.0, 5.0));
        assert!(!stroke.hit_test(50.0, 100.0, 5.0));
    }

    #[test]
    fn test_pressure_width() {
        let stroke = Stroke::new(ToolType::Pen, Color::BLACK, 4.0, Uuid::new_v4());
        // Pen at zero pressure: 0.3 * base
        let mut s = stroke.clone();
        s.add_point(StrokePoint { x: 0.0, y: 0.0, pressure: 0.0, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 });
        assert!((s.width_at(0) - 1.2).abs() < 0.01);
    }
}
