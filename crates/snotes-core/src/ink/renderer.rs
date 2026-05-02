//! Rendering backends: Skia (GPU) and Cairo (CPU)

use super::{BezierSpline, Stroke, Color};

/// Rendering backend trait
pub trait RenderBackend {
    /// Begin a new frame
    fn begin_frame(&mut self, width: u32, height: u32);

    /// Clear the canvas with a background color
    fn clear(&mut self, color: Color);

    /// Render a complete stroke using its fitted Bézier spline
    fn render_stroke(&mut self, stroke: &Stroke, spline: &BezierSpline);

    /// Render a partial stroke (during live drawing)
    fn render_stroke_live(&mut self, stroke: &Stroke);

    /// Render predicted ink (speculative, lighter opacity)
    fn render_predicted(&mut self, points: &[(f64, f64)], color: Color, width: f32);

    /// End frame and present
    fn end_frame(&mut self);

    /// Get the backend name
    fn name(&self) -> &str;
}

/// Skia GPU rendering backend (primary)
pub struct SkiaBackend {
    initialized: bool,
}

impl SkiaBackend {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn init_gpu(&mut self) -> Result<(), String> {
        // In production: initialize Skia GPU context with OpenGL/Vulkan
        log::info!("Initializing Skia GPU backend");
        self.initialized = true;
        Ok(())
    }
}

impl RenderBackend for SkiaBackend {
    fn begin_frame(&mut self, _width: u32, _height: u32) {
        // Begin Skia surface frame
    }

    fn clear(&mut self, _color: Color) {
        // Clear with skia_safe::Canvas::clear()
    }

    fn render_stroke(&mut self, stroke: &Stroke, spline: &BezierSpline) {
        // In production: build a Skia Path from the Bézier segments,
        // with variable width by constructing an outline mesh from
        // the spline + pressure data
        let _ = (stroke, spline);
    }

    fn render_stroke_live(&mut self, stroke: &Stroke) {
        // Render using raw points for low-latency live preview
        let _ = stroke;
    }

    fn render_predicted(&mut self, _points: &[(f64, f64)], _color: Color, _width: f32) {
        // Render predicted points with reduced opacity
    }

    fn end_frame(&mut self) {
        // Flush and present
    }

    fn name(&self) -> &str {
        "Skia GPU"
    }
}

/// Cairo CPU rendering backend (fallback)
pub struct CairoBackend {
    initialized: bool,
}

impl CairoBackend {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn init(&mut self) -> Result<(), String> {
        log::info!("Initializing Cairo CPU backend");
        self.initialized = true;
        Ok(())
    }
}

impl RenderBackend for CairoBackend {
    fn begin_frame(&mut self, _width: u32, _height: u32) {}

    fn clear(&mut self, _color: Color) {}

    fn render_stroke(&mut self, stroke: &Stroke, spline: &BezierSpline) {
        // In production: use cairo_rs to draw the stroke
        let _ = (stroke, spline);
    }

    fn render_stroke_live(&mut self, stroke: &Stroke) {
        let _ = stroke;
    }

    fn render_predicted(&mut self, _points: &[(f64, f64)], _color: Color, _width: f32) {}

    fn end_frame(&mut self) {}

    fn name(&self) -> &str {
        "Cairo CPU"
    }
}

/// Select the best available rendering backend
pub fn select_backend() -> Box<dyn RenderBackend> {
    // Try GPU first, fall back to CPU
    let mut skia = SkiaBackend::new();
    if skia.init_gpu().is_ok() {
        log::info!("Using Skia GPU rendering backend");
        return Box::new(skia);
    }

    log::warn!("Skia GPU unavailable, falling back to Cairo CPU");
    let mut cairo = CairoBackend::new();
    let _ = cairo.init();
    Box::new(cairo)
}
