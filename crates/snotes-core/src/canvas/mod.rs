//! Canvas system: infinite canvas, viewport, and multi-layer support

pub mod templates;

use crate::ink::{Stroke, BezierSpline, Color};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Canvas mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasMode {
    /// Infinite canvas with pan/zoom
    Infinite,
    /// Fixed-size page mode
    FixedPage { width: u32, height: u32 },
}

/// Viewport state (pan/zoom)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Viewport {
    pub offset_x: f64,
    pub offset_y: f64,
    pub zoom: f64,
    pub rotation: f64, // degrees
}

impl Default for Viewport {
    fn default() -> Self {
        Self { offset_x: 0.0, offset_y: 0.0, zoom: 1.0, rotation: 0.0 }
    }
}

impl Viewport {
    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, sx: f64, sy: f64) -> (f64, f64) {
        let cx = (sx - self.offset_x) / self.zoom;
        let cy = (sy - self.offset_y) / self.zoom;
        (cx, cy)
    }

    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, cx: f64, cy: f64) -> (f64, f64) {
        let sx = cx * self.zoom + self.offset_x;
        let sy = cy * self.zoom + self.offset_y;
        (sx, sy)
    }

    /// Zoom to a point (keeping that point fixed on screen)
    pub fn zoom_to_point(&mut self, screen_x: f64, screen_y: f64, new_zoom: f64) {
        let (cx, cy) = self.screen_to_canvas(screen_x, screen_y);
        self.zoom = new_zoom.clamp(0.1, 10.0);
        self.offset_x = screen_x - cx * self.zoom;
        self.offset_y = screen_y - cy * self.zoom;
    }

    /// Pan by a delta
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    /// Fit the viewport to show all content
    pub fn fit_to_bounds(&mut self, bounds: (f64, f64, f64, f64), screen_w: f64, screen_h: f64) {
        let (min_x, min_y, max_x, max_y) = bounds;
        let content_w = (max_x - min_x).max(1.0);
        let content_h = (max_y - min_y).max(1.0);
        self.zoom = (screen_w / content_w).min(screen_h / content_h) * 0.9;
        self.offset_x = (screen_w - content_w * self.zoom) / 2.0 - min_x * self.zoom;
        self.offset_y = (screen_h - content_h * self.zoom) / 2.0 - min_y * self.zoom;
    }
}

/// A layer on the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32, // 0.0–1.0
    pub order: i32,
    pub strokes: Vec<Uuid>, // stroke IDs on this layer
}

impl Layer {
    pub fn new(name: &str, order: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
            order,
            strokes: Vec::new(),
        }
    }
}

/// The canvas holds all layers and strokes for a page
pub struct Canvas {
    pub mode: CanvasMode,
    pub viewport: Viewport,
    pub layers: Vec<Layer>,
    pub strokes: Vec<Stroke>,
    pub splines: Vec<(Uuid, BezierSpline)>, // stroke_id -> fitted spline
    pub background_color: Color,
    pub grid_visible: bool,
    pub grid_size: f64,
    pub snap_to_grid: bool,
    pub guides_visible: bool,
}

impl Canvas {
    pub fn new(mode: CanvasMode) -> Self {
        let default_layer = Layer::new("Layer 1", 0);
        Self {
            mode,
            viewport: Viewport::default(),
            layers: vec![default_layer],
            strokes: Vec::new(),
            splines: Vec::new(),
            background_color: Color::WHITE,
            grid_visible: false,
            grid_size: 20.0,
            snap_to_grid: false,
            guides_visible: false,
        }
    }

    /// Add a layer
    pub fn add_layer(&mut self, name: &str) -> Uuid {
        let order = self.layers.len() as i32;
        let layer = Layer::new(name, order);
        let id = layer.id;
        self.layers.push(layer);
        id
    }

    /// Get the active (topmost unlocked visible) layer
    pub fn active_layer(&self) -> Option<&Layer> {
        self.layers
            .iter()
            .filter(|l| l.visible && !l.locked)
            .max_by_key(|l| l.order)
    }

    /// Get the active layer ID
    pub fn active_layer_id(&self) -> Option<Uuid> {
        self.active_layer().map(|l| l.id)
    }

    /// Add a completed stroke to the canvas
    pub fn add_stroke(&mut self, stroke: Stroke, spline: BezierSpline) {
        let layer_id = stroke.layer_id;
        let stroke_id = stroke.id;
        self.strokes.push(stroke);
        self.splines.push((stroke_id, spline));
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) {
            layer.strokes.push(stroke_id);
        }
    }

    /// Remove a stroke by ID
    pub fn remove_stroke(&mut self, stroke_id: Uuid) -> Option<Stroke> {
        let pos = self.strokes.iter().position(|s| s.id == stroke_id)?;
        let stroke = self.strokes.remove(pos);
        self.splines.retain(|(id, _)| *id != stroke_id);
        for layer in &mut self.layers {
            layer.strokes.retain(|id| *id != stroke_id);
        }
        Some(stroke)
    }

    /// Get all visible strokes in render order
    pub fn visible_strokes(&self) -> Vec<&Stroke> {
        let mut result = Vec::new();
        let mut sorted_layers: Vec<&Layer> = self.layers.iter().filter(|l| l.visible).collect();
        sorted_layers.sort_by_key(|l| l.order);

        for layer in sorted_layers {
            for stroke_id in &layer.strokes {
                if let Some(stroke) = self.strokes.iter().find(|s| s.id == *stroke_id) {
                    result.push(stroke);
                }
            }
        }
        result
    }

    /// Snap a point to the grid if snap is enabled
    pub fn snap_point(&self, x: f64, y: f64) -> (f64, f64) {
        if !self.snap_to_grid {
            return (x, y);
        }
        let sx = (x / self.grid_size).round() * self.grid_size;
        let sy = (y / self.grid_size).round() * self.grid_size;
        (sx, sy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_transform() {
        let vp = Viewport { offset_x: 100.0, offset_y: 50.0, zoom: 2.0, rotation: 0.0 };
        let (cx, cy) = vp.screen_to_canvas(200.0, 150.0);
        assert!((cx - 50.0).abs() < 1e-10);
        assert!((cy - 50.0).abs() < 1e-10);
        let (sx, sy) = vp.canvas_to_screen(cx, cy);
        assert!((sx - 200.0).abs() < 1e-10);
        assert!((sy - 150.0).abs() < 1e-10);
    }

    #[test]
    fn test_canvas_layers() {
        let mut canvas = Canvas::new(CanvasMode::Infinite);
        assert_eq!(canvas.layers.len(), 1);
        canvas.add_layer("Layer 2");
        assert_eq!(canvas.layers.len(), 2);
    }
}
