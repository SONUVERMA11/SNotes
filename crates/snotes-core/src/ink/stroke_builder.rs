//! Stroke builder — constructs strokes from input events in real-time

use super::{BezierSpline, PredictiveInk, Stroke, StrokePoint, Color, ToolType};
use uuid::Uuid;

/// Builds a stroke incrementally from input events
pub struct StrokeBuilder {
    current_stroke: Option<Stroke>,
    raw_points: Vec<StrokePoint>,
    predictor: PredictiveInk,
    smoothing_window: usize,
    min_distance: f64,
}

impl StrokeBuilder {
    pub fn new(lookahead_frames: u32) -> Self {
        Self {
            current_stroke: None,
            raw_points: Vec::new(),
            predictor: PredictiveInk::new(lookahead_frames),
            smoothing_window: 3,
            min_distance: 1.0, // minimum pixel distance between points
        }
    }

    /// Begin a new stroke
    pub fn begin_stroke(
        &mut self,
        tool: ToolType,
        color: Color,
        width: f32,
        layer_id: Uuid,
        initial_point: StrokePoint,
    ) {
        let mut stroke = Stroke::new(tool, color, width, layer_id);
        stroke.add_point(initial_point);
        self.raw_points.clear();
        self.raw_points.push(initial_point);
        self.predictor.reset();
        self.current_stroke = Some(stroke);
    }

    /// Add a point to the current stroke
    pub fn add_point(&mut self, point: StrokePoint) -> Option<StrokeUpdate> {
        if self.current_stroke.is_none() {
            return None;
        }

        // Distance filter: skip points that are too close
        if let Some(last) = self.raw_points.last() {
            if last.distance_to(&point) < self.min_distance {
                return None;
            }
        }

        // Apply smoothing (moving average) — before mutable borrow of current_stroke
        let smoothed = self.smooth_point(&point);
        self.raw_points.push(point);

        // Now borrow current_stroke mutably
        let stroke = self.current_stroke.as_mut().unwrap();
        stroke.add_point(smoothed);

        // Get predictive points
        let predictions = self.predictor.predict(smoothed);

        Some(StrokeUpdate {
            new_point: smoothed,
            predicted_points: predictions,
        })
    }

    /// Finish the current stroke and return the completed Bézier spline
    pub fn finish_stroke(&mut self) -> Option<(Stroke, BezierSpline)> {
        let stroke = self.current_stroke.take()?;
        let spline = BezierSpline::fit_from_points(&stroke.points);
        self.raw_points.clear();
        self.predictor.reset();
        Some((stroke, spline))
    }

    /// Cancel the current stroke
    pub fn cancel_stroke(&mut self) {
        self.current_stroke = None;
        self.raw_points.clear();
        self.predictor.reset();
    }

    /// Check if currently building a stroke
    pub fn is_building(&self) -> bool {
        self.current_stroke.is_some()
    }

    /// Get the current in-progress spline for live rendering
    pub fn current_spline(&self) -> Option<BezierSpline> {
        let stroke = self.current_stroke.as_ref()?;
        if stroke.points.len() < 2 {
            return None;
        }
        Some(BezierSpline::fit_from_points(&stroke.points))
    }

    fn smooth_point(&self, point: &StrokePoint) -> StrokePoint {
        if self.raw_points.len() < self.smoothing_window {
            return *point;
        }

        let n = self.raw_points.len();
        let window_start = n.saturating_sub(self.smoothing_window);
        let window = &self.raw_points[window_start..];

        let mut x = point.x;
        let mut y = point.y;
        let mut pressure = point.pressure;
        let count = window.len() as f64 + 1.0;

        for p in window {
            x += p.x;
            y += p.y;
            pressure += p.pressure;
        }

        StrokePoint {
            x: x / count,
            y: y / count,
            pressure: pressure / count as f32,
            tilt_x: point.tilt_x,
            tilt_y: point.tilt_y,
            timestamp_us: point.timestamp_us,
        }
    }
}

/// Update information when a new point is added
#[derive(Debug)]
pub struct StrokeUpdate {
    pub new_point: StrokePoint,
    pub predicted_points: Vec<StrokePoint>,
}
