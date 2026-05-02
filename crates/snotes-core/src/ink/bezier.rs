//! Bézier cubic spline fitting for smooth stroke rendering

use super::StrokePoint;

/// A cubic Bézier curve segment
#[derive(Debug, Clone, Copy)]
pub struct CubicBezier {
    pub p0: (f64, f64),
    pub p1: (f64, f64), // control point 1
    pub p2: (f64, f64), // control point 2
    pub p3: (f64, f64),
    /// Pressure at start and end for width interpolation
    pub pressure_start: f32,
    pub pressure_end: f32,
}

impl CubicBezier {
    /// Evaluate the curve at parameter t (0.0–1.0)
    pub fn eval(&self, t: f64) -> (f64, f64) {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let x = mt3 * self.p0.0
            + 3.0 * mt2 * t * self.p1.0
            + 3.0 * mt * t2 * self.p2.0
            + t3 * self.p3.0;
        let y = mt3 * self.p0.1
            + 3.0 * mt2 * t * self.p1.1
            + 3.0 * mt * t2 * self.p2.1
            + t3 * self.p3.1;

        (x, y)
    }

    /// Evaluate the tangent (derivative) at parameter t
    pub fn tangent(&self, t: f64) -> (f64, f64) {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        let x = 3.0 * mt2 * (self.p1.0 - self.p0.0)
            + 6.0 * mt * t * (self.p2.0 - self.p1.0)
            + 3.0 * t2 * (self.p3.0 - self.p2.0);
        let y = 3.0 * mt2 * (self.p1.1 - self.p0.1)
            + 6.0 * mt * t * (self.p2.1 - self.p1.1)
            + 3.0 * t2 * (self.p3.1 - self.p2.1);

        (x, y)
    }

    /// Get the normal at parameter t
    pub fn normal(&self, t: f64) -> (f64, f64) {
        let (tx, ty) = self.tangent(t);
        let len = (tx * tx + ty * ty).sqrt();
        if len < 1e-10 {
            return (0.0, 1.0);
        }
        (-ty / len, tx / len)
    }

    /// Interpolate pressure at parameter t
    pub fn pressure_at(&self, t: f64) -> f32 {
        self.pressure_start + (self.pressure_end - self.pressure_start) * t as f32
    }

    /// Approximate arc length using subdivision
    pub fn arc_length(&self, segments: usize) -> f64 {
        let mut length = 0.0;
        let mut prev = self.eval(0.0);
        for i in 1..=segments {
            let t = i as f64 / segments as f64;
            let curr = self.eval(t);
            let dx = curr.0 - prev.0;
            let dy = curr.1 - prev.1;
            length += (dx * dx + dy * dy).sqrt();
            prev = curr;
        }
        length
    }
}

/// Bézier spline — a chain of cubic Bézier segments
#[derive(Debug, Clone)]
pub struct BezierSpline {
    pub segments: Vec<CubicBezier>,
}

impl BezierSpline {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    /// Fit a smooth Bézier spline through a series of stroke points.
    /// Uses Catmull-Rom to cubic Bézier conversion for C1 continuity.
    pub fn fit_from_points(points: &[StrokePoint]) -> Self {
        let mut spline = BezierSpline::new();

        if points.len() < 2 {
            return spline;
        }

        if points.len() == 2 {
            // Simple linear segment
            let p0 = (points[0].x, points[0].y);
            let p3 = (points[1].x, points[1].y);
            let p1 = (
                p0.0 + (p3.0 - p0.0) / 3.0,
                p0.1 + (p3.1 - p0.1) / 3.0,
            );
            let p2 = (
                p0.0 + 2.0 * (p3.0 - p0.0) / 3.0,
                p0.1 + 2.0 * (p3.1 - p0.1) / 3.0,
            );
            spline.segments.push(CubicBezier {
                p0, p1, p2, p3,
                pressure_start: points[0].pressure,
                pressure_end: points[1].pressure,
            });
            return spline;
        }

        // Catmull-Rom to Bézier conversion
        // For each segment between points[i] and points[i+1],
        // we need points[i-1] and points[i+2] for tangent calculation
        for i in 0..points.len() - 1 {
            let p_prev = if i > 0 { &points[i - 1] } else { &points[i] };
            let p_curr = &points[i];
            let p_next = &points[i + 1];
            let p_next2 = if i + 2 < points.len() {
                &points[i + 2]
            } else {
                &points[i + 1]
            };

            // Catmull-Rom tangents (alpha = 0.5 for centripetal)
            let tension = 0.5;
            let t1x = tension * (p_next.x - p_prev.x);
            let t1y = tension * (p_next.y - p_prev.y);
            let t2x = tension * (p_next2.x - p_curr.x);
            let t2y = tension * (p_next2.y - p_curr.y);

            // Convert to cubic Bézier control points
            let cp1 = (
                p_curr.x + t1x / 3.0,
                p_curr.y + t1y / 3.0,
            );
            let cp2 = (
                p_next.x - t2x / 3.0,
                p_next.y - t2y / 3.0,
            );

            spline.segments.push(CubicBezier {
                p0: (p_curr.x, p_curr.y),
                p1: cp1,
                p2: cp2,
                p3: (p_next.x, p_next.y),
                pressure_start: p_curr.pressure,
                pressure_end: p_next.pressure,
            });
        }

        spline
    }

    /// Get total arc length of the spline
    pub fn total_length(&self) -> f64 {
        self.segments.iter().map(|s| s.arc_length(16)).sum()
    }

    /// Evaluate the spline at a global parameter t (0.0–1.0 across all segments)
    pub fn eval_global(&self, t: f64) -> Option<(f64, f64)> {
        if self.segments.is_empty() {
            return None;
        }
        let n = self.segments.len() as f64;
        let scaled = t * n;
        let idx = (scaled as usize).min(self.segments.len() - 1);
        let local_t = scaled - idx as f64;
        Some(self.segments[idx].eval(local_t.clamp(0.0, 1.0)))
    }
}

impl Default for BezierSpline {
    fn default() -> Self {
        Self::new()
    }
}

/// Predictive ink: extrapolate future points for latency reduction
pub struct PredictiveInk {
    lookahead_frames: u32,
    history: Vec<StrokePoint>,
}

impl PredictiveInk {
    pub fn new(lookahead_frames: u32) -> Self {
        Self {
            lookahead_frames,
            history: Vec::new(),
        }
    }

    /// Add a new point and return predicted future points
    pub fn predict(&mut self, point: StrokePoint) -> Vec<StrokePoint> {
        self.history.push(point);

        // Need at least 3 points for quadratic extrapolation
        if self.history.len() < 3 {
            return Vec::new();
        }

        let n = self.history.len();
        let p0 = &self.history[n - 3];
        let p1 = &self.history[n - 2];
        let p2 = &self.history[n - 1];

        // Quadratic extrapolation
        let mut predictions = Vec::new();
        for frame in 1..=self.lookahead_frames {
            let t = frame as f64;
            // Simple quadratic extrapolation from last 3 points
            let ax = p0.x - 2.0 * p1.x + p2.x;
            let _bx = -3.0 * p0.x + 4.0 * p1.x - p2.x; // not used, simplify
            let pred_x = p2.x + t * (p2.x - p1.x) + 0.5 * t * t * ax;
            let ay = p0.y - 2.0 * p1.y + p2.y;
            let pred_y = p2.y + t * (p2.y - p1.y) + 0.5 * t * t * ay;

            predictions.push(StrokePoint {
                x: pred_x,
                y: pred_y,
                pressure: p2.pressure, // maintain last pressure
                tilt_x: p2.tilt_x,
                tilt_y: p2.tilt_y,
                timestamp_us: p2.timestamp_us + (frame as u64 * 8333), // ~120fps
            });
        }

        // Keep history bounded
        if self.history.len() > 10 {
            self.history.drain(0..self.history.len() - 10);
        }

        predictions
    }

    /// Reset prediction state (on stroke end)
    pub fn reset(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_eval_endpoints() {
        let bezier = CubicBezier {
            p0: (0.0, 0.0), p1: (0.0, 1.0),
            p2: (1.0, 1.0), p3: (1.0, 0.0),
            pressure_start: 0.5, pressure_end: 0.5,
        };
        let start = bezier.eval(0.0);
        assert!((start.0 - 0.0).abs() < 1e-10);
        assert!((start.1 - 0.0).abs() < 1e-10);
        let end = bezier.eval(1.0);
        assert!((end.0 - 1.0).abs() < 1e-10);
        assert!((end.1 - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_spline_fit() {
        let points = vec![
            StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 },
            StrokePoint { x: 10.0, y: 5.0, pressure: 0.6, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 },
            StrokePoint { x: 20.0, y: 0.0, pressure: 0.7, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 2000 },
            StrokePoint { x: 30.0, y: 5.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 3000 },
        ];
        let spline = BezierSpline::fit_from_points(&points);
        assert_eq!(spline.segments.len(), 3);
    }

    #[test]
    fn test_predictive_ink() {
        let mut predictor = PredictiveInk::new(2);
        let p0 = StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 };
        let p1 = StrokePoint { x: 10.0, y: 10.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 8333 };
        let p2 = StrokePoint { x: 20.0, y: 20.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 16666 };

        assert!(predictor.predict(p0).is_empty());
        assert!(predictor.predict(p1).is_empty());
        let preds = predictor.predict(p2);
        assert_eq!(preds.len(), 2);
        // Linear motion should predict linearly
        assert!(preds[0].x > 20.0);
        assert!(preds[0].y > 20.0);
    }
}
