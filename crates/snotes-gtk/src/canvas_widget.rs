//! GTK4 Canvas Drawing Widget — integrates with snotes-core rendering pipeline

use gtk4::prelude::*;
use gtk4::glib;
use snotes_core::canvas::Viewport;
use snotes_core::ink::{StrokePoint, Color};
use snotes_core::tools::ToolPalette;
use std::cell::RefCell;
use std::rc::Rc;

/// State shared between the canvas widget and its event handlers
pub struct CanvasState {
    pub viewport: Viewport,
    pub tool_palette: ToolPalette,
    pub is_drawing: bool,
    pub current_points: Vec<StrokePoint>,
    pub width: f64,
    pub height: f64,
    pub needs_redraw: bool,
    /// Cursor position for hover preview
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub cursor_visible: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            tool_palette: ToolPalette::default(),
            is_drawing: false,
            current_points: Vec::new(),
            width: 1400.0,
            height: 900.0,
            needs_redraw: true,
            cursor_x: 0.0,
            cursor_y: 0.0,
            cursor_visible: false,
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
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let s = state_draw.borrow();

        // Background
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().ok();

        // Page shadow (to show page bounds in infinite canvas mode)
        let page_w = 1191.0 * s.viewport.zoom;
        let page_h = 1684.0 * s.viewport.zoom;
        let px = s.viewport.offset_x;
        let py = s.viewport.offset_y;

        // Drop shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
        cr.rectangle(px + 4.0, py + 4.0, page_w, page_h);
        cr.fill().ok();

        // Page
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(px, py, page_w, page_h);
        cr.fill().ok();

        // Page border
        cr.set_source_rgba(0.8, 0.8, 0.8, 1.0);
        cr.set_line_width(0.5);
        cr.rectangle(px, py, page_w, page_h);
        cr.stroke().ok();

        // Draw current stroke in progress
        if s.is_drawing && s.current_points.len() >= 2 {
            cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
            cr.set_line_width(2.0);
            cr.set_line_cap(gtk4::cairo::LineCap::Round);
            cr.set_line_join(gtk4::cairo::LineJoin::Round);

            let first = &s.current_points[0];
            let (sx, sy) = s.viewport.canvas_to_screen(first.x, first.y);
            cr.move_to(sx, sy);

            for point in &s.current_points[1..] {
                let (sx, sy) = s.viewport.canvas_to_screen(point.x, point.y);
                cr.line_to(sx, sy);
            }
            cr.stroke().ok();
        }

        // Hover cursor
        if s.cursor_visible && !s.is_drawing {
            let tool = s.tool_palette.active_tool();
            let radius = tool.settings.width as f64 * s.viewport.zoom / 2.0;
            cr.set_source_rgba(0.3, 0.3, 0.3, 0.4);
            cr.arc(s.cursor_x, s.cursor_y, radius.max(2.0), 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().ok();
        }

        // Info text (temporary, for development)
        cr.set_source_rgba(0.5, 0.5, 0.5, 0.6);
        cr.set_font_size(11.0);
        cr.move_to(10.0, height as f64 - 10.0);
        let info = format!(
            "Zoom: {:.0}% | Offset: ({:.0}, {:.0}) | Tool: {:?}",
            s.viewport.zoom * 100.0,
            s.viewport.offset_x,
            s.viewport.offset_y,
            s.tool_palette.active_tool().settings.tool_type,
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
        s.is_drawing = true;
        s.current_points.clear();
        s.current_points.push(StrokePoint {
            x: cx, y: cy, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0,
        });
        da_down.queue_draw();
    });

    let state_motion = state.clone();
    let da_motion = drawing_area.clone();
    stylus.connect_motion(move |gesture, x, y| {
        let mut s = state_motion.borrow_mut();
        if s.is_drawing {
            let (cx, cy) = s.viewport.screen_to_canvas(x, y);
            let pressure = gesture.axis(gdk4::AxisUse::Pressure).unwrap_or(0.5);
            s.current_points.push(StrokePoint {
                x: cx, y: cy, pressure: pressure as f32,
                tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0,
            });
        }
        s.cursor_x = x;
        s.cursor_y = y;
        da_motion.queue_draw();
    });

    let state_up = state.clone();
    let da_up = drawing_area.clone();
    stylus.connect_up(move |_gesture, _x, _y| {
        let mut s = state_up.borrow_mut();
        s.is_drawing = false;
        // TODO: finalize stroke via StrokeBuilder, add to canvas
        s.current_points.clear();
        da_up.queue_draw();
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
        s.is_drawing = true;
        s.current_points.clear();
        s.current_points.push(StrokePoint {
            x: cx, y: cy, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0,
        });
        da_drag.queue_draw();
    });

    let state_drag_update = state.clone();
    let da_drag_u = drawing_area.clone();
    drag.connect_drag_update(move |gesture, offset_x, offset_y| {
        let mut s = state_drag_update.borrow_mut();
        if s.is_drawing {
            if let Some((start_x, start_y)) = gesture.start_point() {
                let x = start_x + offset_x;
                let y = start_y + offset_y;
                let (cx, cy) = s.viewport.screen_to_canvas(x, y);
                s.current_points.push(StrokePoint {
                    x: cx, y: cy, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0,
                });
            }
        }
        da_drag_u.queue_draw();
    });

    let state_drag_end = state.clone();
    let da_drag_e = drawing_area.clone();
    drag.connect_drag_end(move |_gesture, _offset_x, _offset_y| {
        let mut s = state_drag_end.borrow_mut();
        s.is_drawing = false;
        s.current_points.clear();
        da_drag_e.queue_draw();
    });

    drawing_area.add_controller(drag);

    // ── Scroll/zoom gesture ───────────────────────────────────
    let scroll = gtk4::EventControllerScroll::new(
        gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE,
    );
    let state_scroll = state.clone();
    let da_scroll = drawing_area.clone();
    scroll.connect_scroll(move |_ctrl, _dx, dy| {
        let mut s = state_scroll.borrow_mut();
        let zoom_delta = if dy < 0.0 { 1.1 } else { 0.9 };
        let new_zoom = (s.viewport.zoom * zoom_delta).clamp(0.1, 10.0);
        let cx = s.width / 2.0;
        let cy = s.height / 2.0;
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

    // Set accessible labels
    drawing_area.update_property(
        &[gtk4::accessible::Property::Label("Drawing Canvas")],
    );

    (drawing_area, state)
}
