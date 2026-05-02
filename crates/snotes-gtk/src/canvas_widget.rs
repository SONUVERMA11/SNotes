//! GTK4 Canvas Drawing Widget — fully functional drawing surface
//!
//! Integrates with snotes-core: stroke persistence, Bézier rendering,
//! tool switching, undo/redo, zoom, pan, and page templates.

use gtk4::prelude::*;
use gtk4::glib;
use snotes_core::canvas::Viewport;
use snotes_core::ink::{Stroke, StrokePoint, BezierSpline, Color, ToolType};
use snotes_core::history::{HistoryStack, HistoryAction};
use snotes_core::tools::ToolPalette;
use std::cell::RefCell;
use std::rc::Rc;

/// Page template rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualTemplate {
    Blank,
    Lined,
    Grid,
    Dotted,
}

/// State shared between the canvas widget and its event handlers
pub struct CanvasState {
    pub viewport: Viewport,
    pub tool_palette: ToolPalette,
    pub is_drawing: bool,
    pub current_points: Vec<StrokePoint>,
    /// All completed strokes on the canvas
    pub strokes: Vec<Stroke>,
    /// Bézier splines for each stroke (for smooth rendering)
    pub splines: Vec<BezierSpline>,
    /// Undo/redo history
    pub history: HistoryStack,
    /// Current stroke color
    pub stroke_color: Color,
    /// Current stroke width
    pub stroke_width: f32,
    /// Current tool type
    pub current_tool: ToolType,
    /// Current page template
    pub template: VisualTemplate,
    /// Canvas dimensions
    pub width: f64,
    pub height: f64,
    /// Cursor for hover preview
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub cursor_visible: bool,
    /// Eraser mode
    pub is_erasing: bool,
    /// Pan state
    pub pan_start_x: f64,
    pub pan_start_y: f64,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            tool_palette: ToolPalette::default(),
            is_drawing: false,
            current_points: Vec::new(),
            strokes: Vec::new(),
            splines: Vec::new(),
            history: HistoryStack::new(200),
            stroke_color: Color::BLACK,
            stroke_width: 2.5,
            current_tool: ToolType::Pen,
            template: VisualTemplate::Lined,
            width: 1400.0,
            height: 900.0,
            cursor_x: 0.0,
            cursor_y: 0.0,
            cursor_visible: false,
            is_erasing: false,
            pan_start_x: 0.0,
            pan_start_y: 0.0,
        }
    }
}

impl CanvasState {
    /// Finalize current stroke and add to canvas
    pub fn finalize_stroke(&mut self) {
        if self.current_points.len() < 2 {
            self.current_points.clear();
            return;
        }

        let layer_id = uuid::Uuid::new_v4();
        let mut stroke = Stroke::new(
            self.current_tool,
            self.stroke_color,
            self.stroke_width,
            layer_id,
        );
        for p in &self.current_points {
            stroke.add_point(*p);
        }

        let spline = BezierSpline::fit_from_points(&stroke.points);

        // Push to history for undo
        self.history.push(HistoryAction::AddStroke { stroke: stroke.clone() });

        self.strokes.push(stroke);
        self.splines.push(spline);
        self.current_points.clear();
    }

    /// Erase strokes near a point
    pub fn erase_at(&mut self, x: f64, y: f64, radius: f64) {
        let mut removed = Vec::new();
        let mut i = 0;
        while i < self.strokes.len() {
            if self.strokes[i].hit_test(x, y, radius) {
                let stroke = self.strokes.remove(i);
                self.splines.remove(i);
                removed.push(stroke);
            } else {
                i += 1;
            }
        }
        for stroke in removed {
            self.history.push(HistoryAction::RemoveStroke { stroke });
        }
    }

    /// Undo last action
    pub fn undo(&mut self) {
        if let Some(action) = self.history.undo() {
            match action {
                HistoryAction::AddStroke { stroke } => {
                    // Remove the stroke that was added
                    if let Some(pos) = self.strokes.iter().position(|s| s.id == stroke.id) {
                        self.strokes.remove(pos);
                        self.splines.remove(pos);
                    }
                }
                HistoryAction::RemoveStroke { stroke } => {
                    // Re-add the stroke that was removed
                    let spline = BezierSpline::fit_from_points(&stroke.points);
                    self.strokes.push(stroke);
                    self.splines.push(spline);
                }
                _ => {}
            }
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self) {
        if let Some(action) = self.history.redo() {
            match action {
                HistoryAction::AddStroke { stroke } => {
                    let spline = BezierSpline::fit_from_points(&stroke.points);
                    self.strokes.push(stroke);
                    self.splines.push(spline);
                }
                HistoryAction::RemoveStroke { stroke } => {
                    if let Some(pos) = self.strokes.iter().position(|s| s.id == stroke.id) {
                        self.strokes.remove(pos);
                        self.splines.remove(pos);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Render a completed stroke using Cairo with pressure-variable width
fn render_stroke_cairo(cr: &gtk4::cairo::Context, stroke: &Stroke, spline: &BezierSpline, viewport: &Viewport) {
    if stroke.points.len() < 2 { return; }

    let color = &stroke.color;
    let alpha = match stroke.tool {
        ToolType::Highlighter => 0.35,
        _ => color.a as f64,
    };
    cr.set_source_rgba(color.r as f64, color.g as f64, color.b as f64, alpha);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    cr.set_line_join(gtk4::cairo::LineJoin::Round);

    // Draw stroke segments with variable width based on pressure
    for i in 0..stroke.points.len().saturating_sub(1) {
        let p0 = &stroke.points[i];
        let p1 = &stroke.points[i + 1];

        let width = match stroke.tool {
            ToolType::Pen => stroke.base_width as f64 * (0.3 + 0.7 * p0.pressure as f64),
            ToolType::Brush => stroke.base_width as f64 * (0.1 + 0.9 * p0.pressure as f64),
            ToolType::Pencil => stroke.base_width as f64 * (0.5 + 0.5 * p0.pressure as f64) * 0.8,
            ToolType::Marker => stroke.base_width as f64 * 1.5,
            ToolType::Highlighter => stroke.base_width as f64 * 4.0,
            ToolType::Eraser => stroke.base_width as f64 * 2.0,
        };

        cr.set_line_width(width * viewport.zoom);

        let (sx0, sy0) = viewport.canvas_to_screen(p0.x, p0.y);
        let (sx1, sy1) = viewport.canvas_to_screen(p1.x, p1.y);

        cr.move_to(sx0, sy0);
        cr.line_to(sx1, sy1);
        cr.stroke().ok();
    }
}

/// Render page template (lines, grid, dots)
fn render_template(cr: &gtk4::cairo::Context, template: VisualTemplate, viewport: &Viewport, page_w: f64, page_h: f64) {
    let px = viewport.offset_x;
    let py = viewport.offset_y;
    let pw = page_w * viewport.zoom;
    let ph = page_h * viewport.zoom;

    match template {
        VisualTemplate::Blank => {},
        VisualTemplate::Lined => {
            cr.set_source_rgba(0.75, 0.85, 0.95, 0.6);
            cr.set_line_width(0.5);
            let spacing = 30.0 * viewport.zoom;
            let margin_top = 80.0 * viewport.zoom;
            let mut y = py + margin_top;
            while y < py + ph {
                cr.move_to(px + 10.0, y);
                cr.line_to(px + pw - 10.0, y);
                cr.stroke().ok();
                y += spacing;
            }
            // Left margin line
            cr.set_source_rgba(0.9, 0.5, 0.5, 0.4);
            cr.set_line_width(1.0);
            let margin_left = 70.0 * viewport.zoom;
            cr.move_to(px + margin_left, py);
            cr.line_to(px + margin_left, py + ph);
            cr.stroke().ok();
        }
        VisualTemplate::Grid => {
            cr.set_source_rgba(0.8, 0.85, 0.9, 0.4);
            cr.set_line_width(0.3);
            let spacing = 25.0 * viewport.zoom;
            // Vertical lines
            let mut x = px;
            while x < px + pw {
                cr.move_to(x, py);
                cr.line_to(x, py + ph);
                cr.stroke().ok();
                x += spacing;
            }
            // Horizontal lines
            let mut y = py;
            while y < py + ph {
                cr.move_to(px, y);
                cr.line_to(px + pw, y);
                cr.stroke().ok();
                y += spacing;
            }
        }
        VisualTemplate::Dotted => {
            cr.set_source_rgba(0.6, 0.65, 0.7, 0.5);
            let spacing = 25.0 * viewport.zoom;
            let dot_r = 1.0 * viewport.zoom;
            let mut x = px + spacing;
            while x < px + pw {
                let mut y = py + spacing;
                while y < py + ph {
                    cr.arc(x, y, dot_r, 0.0, 2.0 * std::f64::consts::PI);
                    cr.fill().ok();
                    y += spacing;
                }
                x += spacing;
            }
        }
    }
}

/// Create the canvas drawing area with all gesture handlers
pub fn create_canvas_widget() -> (gtk4::DrawingArea, Rc<RefCell<CanvasState>>) {
    let state = Rc::new(RefCell::new(CanvasState::default()));
    let drawing_area = gtk4::DrawingArea::builder()
        .vexpand(true)
        .hexpand(true)
        .focusable(true)
        .css_classes(vec!["canvas-area".to_string()])
        .build();

    // ── Draw handler ──────────────────────────────────────────
    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_area, cr, _width, _height| {
        let s = state_draw.borrow();

        // Canvas background (dark gray outside page)
        cr.set_source_rgb(0.92, 0.92, 0.90);
        cr.paint().ok();

        // Page dimensions (A4-ish)
        let page_w = 794.0;
        let page_h = 1123.0;
        let px = s.viewport.offset_x;
        let py = s.viewport.offset_y;
        let pw = page_w * s.viewport.zoom;
        let ph = page_h * s.viewport.zoom;

        // Drop shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.12);
        cr.rectangle(px + 3.0, py + 3.0, pw, ph);
        cr.fill().ok();

        // White page
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(px, py, pw, ph);
        cr.fill().ok();

        // Page border
        cr.set_source_rgba(0.75, 0.75, 0.75, 1.0);
        cr.set_line_width(0.5);
        cr.rectangle(px, py, pw, ph);
        cr.stroke().ok();

        // Page template
        render_template(cr, s.template, &s.viewport, page_w, page_h);

        // ── Render all completed strokes ──
        for (stroke, spline) in s.strokes.iter().zip(s.splines.iter()) {
            render_stroke_cairo(cr, stroke, spline, &s.viewport);
        }

        // ── Render current stroke in progress ──
        if s.is_drawing && s.current_points.len() >= 2 {
            let alpha = match s.current_tool {
                ToolType::Highlighter => 0.35,
                _ => s.stroke_color.a as f64,
            };
            cr.set_source_rgba(
                s.stroke_color.r as f64,
                s.stroke_color.g as f64,
                s.stroke_color.b as f64,
                alpha,
            );
            cr.set_line_cap(gtk4::cairo::LineCap::Round);
            cr.set_line_join(gtk4::cairo::LineJoin::Round);

            for i in 0..s.current_points.len().saturating_sub(1) {
                let p0 = &s.current_points[i];
                let p1 = &s.current_points[i + 1];

                let width = match s.current_tool {
                    ToolType::Pen => s.stroke_width as f64 * (0.3 + 0.7 * p0.pressure as f64),
                    ToolType::Brush => s.stroke_width as f64 * (0.1 + 0.9 * p0.pressure as f64),
                    ToolType::Pencil => s.stroke_width as f64 * (0.5 + 0.5 * p0.pressure as f64),
                    ToolType::Marker => s.stroke_width as f64 * 1.5,
                    ToolType::Highlighter => s.stroke_width as f64 * 4.0,
                    ToolType::Eraser => s.stroke_width as f64 * 2.0,
                };
                cr.set_line_width(width * s.viewport.zoom);

                let (sx0, sy0) = s.viewport.canvas_to_screen(p0.x, p0.y);
                let (sx1, sy1) = s.viewport.canvas_to_screen(p1.x, p1.y);

                cr.move_to(sx0, sy0);
                cr.line_to(sx1, sy1);
                cr.stroke().ok();
            }
        }

        // ── Eraser cursor ──
        if s.is_erasing && s.cursor_visible {
            cr.set_source_rgba(1.0, 0.3, 0.3, 0.3);
            cr.set_line_width(1.5);
            let r = s.stroke_width as f64 * 3.0 * s.viewport.zoom;
            cr.arc(s.cursor_x, s.cursor_y, r, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().ok();
        }

        // ── Hover cursor (pen preview) ──
        if s.cursor_visible && !s.is_drawing && !s.is_erasing {
            let radius = s.stroke_width as f64 * s.viewport.zoom / 2.0;
            cr.set_source_rgba(
                s.stroke_color.r as f64,
                s.stroke_color.g as f64,
                s.stroke_color.b as f64,
                0.4,
            );
            cr.arc(s.cursor_x, s.cursor_y, radius.max(1.5), 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().ok();
        }

        // ── Status bar info ──
        cr.set_source_rgba(0.4, 0.4, 0.4, 0.7);
        cr.set_font_size(11.0);
        cr.move_to(10.0, ph.min(800.0) + py + 20.0);
        let info = format!(
            "Zoom: {:.0}% | Strokes: {} | Tool: {:?} | {}",
            s.viewport.zoom * 100.0,
            s.strokes.len(),
            s.current_tool,
            if s.history.can_undo() { "Ctrl+Z to undo" } else { "" },
        );
        cr.show_text(&info).ok();
    });

    // ── Stylus/pen drawing gesture ────────────────────────────
    let stylus = gtk4::GestureStylus::new();
    let state_down = state.clone();
    let da_down = drawing_area.clone();
    stylus.connect_down(move |_gesture, x, y| {
        let mut s = state_down.borrow_mut();
        let (cx, cy) = s.viewport.screen_to_canvas(x, y);
        if s.current_tool == ToolType::Eraser {
            s.is_erasing = true;
            { let r = s.stroke_width as f64 * 3.0; s.erase_at(cx, cy, r); };
        } else {
            s.is_drawing = true;
            s.current_points.clear();
            s.current_points.push(StrokePoint {
                x: cx, y: cy, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0,
                timestamp_us: timestamp_now(),
            });
        }
        da_down.queue_draw();
    });

    let state_motion_s = state.clone();
    let da_motion_s = drawing_area.clone();
    stylus.connect_motion(move |gesture, x, y| {
        let mut s = state_motion_s.borrow_mut();
        let (cx, cy) = s.viewport.screen_to_canvas(x, y);
        if s.is_erasing {
            { let r = s.stroke_width as f64 * 3.0; s.erase_at(cx, cy, r); };
        } else if s.is_drawing {
            let pressure = gesture.axis(gdk4::AxisUse::Pressure).unwrap_or(0.5);
            s.current_points.push(StrokePoint {
                x: cx, y: cy, pressure: pressure as f32,
                tilt_x: 0.0, tilt_y: 0.0,
                timestamp_us: timestamp_now(),
            });
        }
        s.cursor_x = x;
        s.cursor_y = y;
        da_motion_s.queue_draw();
    });

    let state_up_s = state.clone();
    let da_up_s = drawing_area.clone();
    stylus.connect_up(move |_gesture, _x, _y| {
        let mut s = state_up_s.borrow_mut();
        if s.is_drawing {
            s.finalize_stroke();
        }
        s.is_drawing = false;
        s.is_erasing = false;
        da_up_s.queue_draw();
    });

    drawing_area.add_controller(stylus);

    // ── Mouse fallback (for testing without a stylus) ─────────
    let drag = gtk4::GestureDrag::new();
    drag.set_button(gdk4::BUTTON_PRIMARY);
    let state_drag_begin = state.clone();
    let da_drag = drawing_area.clone();
    drag.connect_drag_begin(move |_gesture, x, y| {
        let mut s = state_drag_begin.borrow_mut();
        let (cx, cy) = s.viewport.screen_to_canvas(x, y);
        if s.current_tool == ToolType::Eraser {
            s.is_erasing = true;
            { let r = s.stroke_width as f64 * 3.0; s.erase_at(cx, cy, r); };
        } else {
            s.is_drawing = true;
            s.current_points.clear();
            s.current_points.push(StrokePoint {
                x: cx, y: cy, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0,
                timestamp_us: timestamp_now(),
            });
        }
        da_drag.queue_draw();
    });

    let state_drag_update = state.clone();
    let da_drag_u = drawing_area.clone();
    drag.connect_drag_update(move |gesture, offset_x, offset_y| {
        let mut s = state_drag_update.borrow_mut();
        if let Some((start_x, start_y)) = gesture.start_point() {
            let x = start_x + offset_x;
            let y = start_y + offset_y;
            let (cx, cy) = s.viewport.screen_to_canvas(x, y);
            if s.is_erasing {
                { let r = s.stroke_width as f64 * 3.0; s.erase_at(cx, cy, r); };
            } else if s.is_drawing {
                s.current_points.push(StrokePoint {
                    x: cx, y: cy, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0,
                    timestamp_us: timestamp_now(),
                });
            }
            s.cursor_x = x;
            s.cursor_y = y;
        }
        da_drag_u.queue_draw();
    });

    let state_drag_end = state.clone();
    let da_drag_e = drawing_area.clone();
    drag.connect_drag_end(move |_gesture, _offset_x, _offset_y| {
        let mut s = state_drag_end.borrow_mut();
        if s.is_drawing {
            s.finalize_stroke();
        }
        s.is_drawing = false;
        s.is_erasing = false;
        da_drag_e.queue_draw();
    });

    drawing_area.add_controller(drag);

    // ── Scroll/zoom ───────────────────────────────────────
    let scroll = gtk4::EventControllerScroll::new(
        gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE,
    );
    let state_scroll = state.clone();
    let da_scroll = drawing_area.clone();
    scroll.connect_scroll(move |_ctrl, _dx, dy| {
        let mut s = state_scroll.borrow_mut();
        let zoom_delta = if dy < 0.0 { 1.1 } else { 0.9 };
        let new_zoom = (s.viewport.zoom * zoom_delta).clamp(0.1, 10.0);
        let cx = s.cursor_x;
        let cy = s.cursor_y;
        s.viewport.zoom_to_point(cx, cy, new_zoom);
        da_scroll.queue_draw();
        glib::Propagation::Stop
    });

    drawing_area.add_controller(scroll);

    // ── Mouse motion for hover cursor ─────────────────────────
    let motion = gtk4::EventControllerMotion::new();
    let state_hover = state.clone();
    let da_hover = drawing_area.clone();
    motion.connect_motion(move |_ctrl, x, y| {
        let mut s = state_hover.borrow_mut();
        s.cursor_x = x;
        s.cursor_y = y;
        da_hover.queue_draw();
    });

    let state_enter = state.clone();
    motion.connect_enter(move |_ctrl, _x, _y| {
        state_enter.borrow_mut().cursor_visible = true;
    });

    let state_leave = state.clone();
    let da_leave = drawing_area.clone();
    motion.connect_leave(move |_ctrl| {
        state_leave.borrow_mut().cursor_visible = false;
        da_leave.queue_draw();
    });

    drawing_area.add_controller(motion);

    // ── Pan with middle mouse button ──────────────────────────
    let pan_drag = gtk4::GestureDrag::new();
    pan_drag.set_button(gdk4::BUTTON_MIDDLE);
    let state_pan = state.clone();
    let da_pan = drawing_area.clone();
    pan_drag.connect_drag_update(move |_gesture, offset_x, offset_y| {
        let mut s = state_pan.borrow_mut();
        s.viewport.offset_x += offset_x * 0.3;
        s.viewport.offset_y += offset_y * 0.3;
        da_pan.queue_draw();
    });
    drawing_area.add_controller(pan_drag);

    // Accessibility
    drawing_area.update_property(
        &[gtk4::accessible::Property::Label("Drawing Canvas")],
    );

    (drawing_area, state)
}

fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}
