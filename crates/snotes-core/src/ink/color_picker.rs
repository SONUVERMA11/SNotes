//! Color picker — palette, custom colors, recent colors, and gradients

use serde::{Deserialize, Serialize};

/// Full color picker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPicker {
    pub current: HsvColor,
    pub recent: Vec<super::Color>,
    pub favorites: Vec<super::Color>,
    pub palette: ColorPalette,
    pub max_recent: usize,
}

/// HSV color for the picker wheel
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HsvColor {
    pub h: f32, // 0–360
    pub s: f32, // 0–1
    pub v: f32, // 0–1
    pub a: f32, // 0–1
}

impl HsvColor {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v, a: 1.0 }
    }

    /// Convert HSV to RGB
    pub fn to_rgb(&self) -> super::Color {
        let h = self.h / 60.0;
        let c = self.v * self.s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = self.v - c;

        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        super::Color::from_rgba(r + m, g + m, b + m, self.a)
    }

    /// Convert RGB to HSV
    pub fn from_rgb(color: &super::Color) -> Self {
        let r = color.r;
        let g = color.g;
        let b = color.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta < 1e-6 {
            0.0
        } else if (max - r).abs() < 1e-6 {
            60.0 * (((g - b) / delta) % 6.0)
        } else if (max - g).abs() < 1e-6 {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };

        let s = if max < 1e-6 { 0.0 } else { delta / max };
        let h = if h < 0.0 { h + 360.0 } else { h };

        Self { h, s, v: max, a: color.a }
    }
}

/// Predefined color palettes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub name: String,
    pub colors: Vec<super::Color>,
}

impl ColorPalette {
    /// Default GoodNotes-inspired palette
    pub fn default_palette() -> Self {
        Self {
            name: "Default".to_string(),
            colors: vec![
                super::Color::BLACK,
                super::Color::from_rgba(0.2, 0.2, 0.2, 1.0),     // Dark gray
                super::Color::from_rgba(0.5, 0.5, 0.5, 1.0),     // Gray
                super::Color::from_rgba(0.8, 0.8, 0.8, 1.0),     // Light gray
                super::Color::WHITE,
                super::Color::from_rgba(0.85, 0.2, 0.2, 1.0),    // Red
                super::Color::from_rgba(0.9, 0.5, 0.1, 1.0),     // Orange
                super::Color::from_rgba(0.95, 0.75, 0.1, 1.0),   // Yellow
                super::Color::from_rgba(0.2, 0.7, 0.3, 1.0),     // Green
                super::Color::from_rgba(0.1, 0.6, 0.85, 1.0),    // Blue
                super::Color::from_rgba(0.2, 0.3, 0.7, 1.0),     // Indigo
                super::Color::from_rgba(0.6, 0.2, 0.7, 1.0),     // Purple
                super::Color::from_rgba(0.85, 0.3, 0.5, 1.0),    // Pink
                super::Color::from_rgba(0.4, 0.25, 0.15, 1.0),   // Brown
                super::Color::from_rgba(0.0, 0.5, 0.5, 1.0),     // Teal
                super::Color::from_rgba(0.6, 0.8, 0.2, 1.0),     // Lime
            ],
        }
    }

    /// Pastel palette for highlighting
    pub fn pastel_palette() -> Self {
        Self {
            name: "Pastel".to_string(),
            colors: vec![
                super::Color::from_rgba(1.0, 0.8, 0.8, 0.5),     // Light red
                super::Color::from_rgba(1.0, 0.9, 0.7, 0.5),     // Light orange
                super::Color::from_rgba(1.0, 1.0, 0.7, 0.5),     // Light yellow
                super::Color::from_rgba(0.8, 1.0, 0.8, 0.5),     // Light green
                super::Color::from_rgba(0.7, 0.9, 1.0, 0.5),     // Light blue
                super::Color::from_rgba(0.8, 0.7, 1.0, 0.5),     // Light purple
                super::Color::from_rgba(1.0, 0.8, 0.9, 0.5),     // Light pink
                super::Color::from_rgba(0.7, 1.0, 0.9, 0.5),     // Light teal
            ],
        }
    }

    /// Monochrome palette for technical drawing
    pub fn monochrome_palette() -> Self {
        Self {
            name: "Monochrome".to_string(),
            colors: (0..=10)
                .map(|i| {
                    let v = i as f32 / 10.0;
                    super::Color::from_rgba(v, v, v, 1.0)
                })
                .collect(),
        }
    }
}

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            current: HsvColor::new(0.0, 0.0, 0.0),
            recent: Vec::new(),
            favorites: Vec::new(),
            palette: ColorPalette::default_palette(),
            max_recent: 20,
        }
    }

    /// Set the current color (also adds to recent)
    pub fn set_color(&mut self, color: super::Color) {
        self.current = HsvColor::from_rgb(&color);
        self.add_to_recent(color);
    }

    /// Get the current RGB color
    pub fn get_color(&self) -> super::Color {
        self.current.to_rgb()
    }

    /// Set hue from the color wheel (0–360)
    pub fn set_hue(&mut self, h: f32) {
        self.current.h = h.clamp(0.0, 360.0);
    }

    /// Set saturation and value from the SV square (0–1)
    pub fn set_sv(&mut self, s: f32, v: f32) {
        self.current.s = s.clamp(0.0, 1.0);
        self.current.v = v.clamp(0.0, 1.0);
    }

    /// Set alpha (0–1)
    pub fn set_alpha(&mut self, a: f32) {
        self.current.a = a.clamp(0.0, 1.0);
    }

    /// Set color from hex string
    pub fn set_hex(&mut self, hex: &str) {
        if let Some(color) = super::Color::from_hex(hex) {
            self.set_color(color);
        }
    }

    /// Add a color to favorites
    pub fn add_favorite(&mut self, color: super::Color) {
        if !self.favorites.iter().any(|c| c == &color) {
            self.favorites.push(color);
        }
    }

    /// Remove a color from favorites
    pub fn remove_favorite(&mut self, index: usize) {
        if index < self.favorites.len() {
            self.favorites.remove(index);
        }
    }

    fn add_to_recent(&mut self, color: super::Color) {
        // Don't duplicate
        self.recent.retain(|c| c != &color);
        self.recent.insert(0, color);
        if self.recent.len() > self.max_recent {
            self.recent.truncate(self.max_recent);
        }
    }

    /// Switch to a different palette
    pub fn set_palette(&mut self, palette: ColorPalette) {
        self.palette = palette;
    }
}

impl Default for ColorPicker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsv_roundtrip() {
        let original = super::super::Color::from_rgba(0.5, 0.3, 0.8, 1.0);
        let hsv = HsvColor::from_rgb(&original);
        let back = hsv.to_rgb();
        assert!((original.r - back.r).abs() < 0.02);
        assert!((original.g - back.g).abs() < 0.02);
        assert!((original.b - back.b).abs() < 0.02);
    }

    #[test]
    fn test_color_picker() {
        let mut picker = ColorPicker::new();
        picker.set_hex("#ff0000");
        let c = picker.get_color();
        assert!((c.r - 1.0).abs() < 0.02);
        assert!(c.g < 0.05);
        assert!(c.b < 0.05);
        assert_eq!(picker.recent.len(), 1);
    }

    #[test]
    fn test_palette() {
        let palette = ColorPalette::default_palette();
        assert!(palette.colors.len() >= 12);
    }

    #[test]
    fn test_favorites() {
        let mut picker = ColorPicker::new();
        let color = super::super::Color::from_rgba(0.1, 0.2, 0.3, 1.0);
        picker.add_favorite(color);
        picker.add_favorite(color); // duplicate
        assert_eq!(picker.favorites.len(), 1);
    }
}
