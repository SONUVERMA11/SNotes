//! Stroke geometry generation — converts Bézier splines + pressure into
//! variable-width outline meshes suitable for GPU/CPU rendering.
//!
//! This is the core of what makes strokes look good: each stroke becomes
//! a filled polygon whose width varies with pressure and velocity.

use super::{BezierSpline, Stroke, StrokePoint, Color, ToolType};

/// A triangle in the stroke mesh (for GPU rendering)
#[derive(Debug, Clone, Copy)]
pub struct StrokeVertex {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Generated stroke geometry ready for rendering
#[derive(Debug, Clone)]
pub struct StrokeGeometry {
    /// Left outline of the stroke
    pub left_outline: Vec<(f64, f64)>,
    /// Right outline of the stroke
    pub right_outline: Vec<(f64, f64)>,
    /// Width at each sample point
    pub widths: Vec<f32>,
    /// Triangle strip vertices (for GPU)
    pub vertices: Vec<StrokeVertex>,
    /// Start cap points
    pub start_cap: Vec<(f64, f64)>,
    /// End cap points
    pub end_cap: Vec<(f64, f64)>,
}

/// Configuration for stroke geometry generation
#[derive(Debug, Clone)]
pub struct GeometryConfig {
    /// Number of subdivisions per Bézier segment
    pub subdivisions: usize,
    /// Minimum stroke width in pixels
    pub min_width: f32,
    /// Maximum stroke width in pixels
    pub max_width: f32,
    /// Velocity damping factor (0.0 = no velocity effect, 1.0 = full)
    pub velocity_damping: f32,
    /// Enable round caps
    pub round_caps: bool,
    /// Cap resolution (number of segments in round cap)
    pub cap_resolution: usize,
}

impl Default for GeometryConfig {
    fn default() -> Self {
        Self {
            subdivisions: 8,
            min_width: 0.3,
            max_width: 50.0,
            velocity_damping: 0.3,
            round_caps: true,
            cap_resolution: 8,
        }
    }
}

/// Generate variable-width stroke geometry from a Bézier spline
pub fn generate_stroke_geometry(
    stroke: &Stroke,
    spline: &BezierSpline,
    config: &GeometryConfig,
) -> StrokeGeometry {
    let mut left_outline = Vec::new();
    let mut right_outline = Vec::new();
    let mut widths = Vec::new();

    if spline.segments.is_empty() {
        return StrokeGeometry {
            left_outline,
            right_outline,
            widths,
            vertices: Vec::new(),
            start_cap: Vec::new(),
            end_cap: Vec::new(),
        };
    }

    // Sample points along the spline with normals and widths
    let mut samples: Vec<SamplePoint> = Vec::new();

    for (seg_idx, segment) in spline.segments.iter().enumerate() {
        let num_steps = config.subdivisions;
        let start = if seg_idx == 0 { 0 } else { 1 }; // avoid duplicate at joins

        for step in start..=num_steps {
            let t = step as f64 / num_steps as f64;
            let (px, py) = segment.eval(t);
            let (nx, ny) = segment.normal(t);
            let pressure = segment.pressure_at(t);

            // Calculate width from pressure and tool type
            let base_w = stroke.width_at_pressure(pressure);

            // Apply velocity damping if we have enough samples
            let velocity_factor = if samples.len() >= 2 {
                let prev = &samples[samples.len() - 1];
                let dx = px - prev.x;
                let dy = py - prev.y;
                let dist = (dx * dx + dy * dy).sqrt();
                // Higher velocity → thinner stroke (pen dynamics)
                let vel_normalized = (dist * 0.01).min(1.0);
                1.0 - config.velocity_damping as f64 * vel_normalized
            } else {
                1.0
            };

            let width = (base_w as f64 * velocity_factor)
                .clamp(config.min_width as f64, config.max_width as f64);

            samples.push(SamplePoint {
                x: px,
                y: py,
                nx,
                ny,
                width: width as f32,
                pressure,
            });
        }
    }

    // Generate outlines from samples
    for sample in &samples {
        let half_w = sample.width as f64 / 2.0;
        left_outline.push((
            sample.x + sample.nx * half_w,
            sample.y + sample.ny * half_w,
        ));
        right_outline.push((
            sample.x - sample.nx * half_w,
            sample.y - sample.ny * half_w,
        ));
        widths.push(sample.width);
    }

    // Generate caps
    let start_cap = if config.round_caps && !samples.is_empty() {
        generate_round_cap(&samples[0], true, config.cap_resolution)
    } else {
        Vec::new()
    };

    let end_cap = if config.round_caps && !samples.is_empty() {
        generate_round_cap(samples.last().unwrap(), false, config.cap_resolution)
    } else {
        Vec::new()
    };

    // Generate triangle strip vertices for GPU rendering
    let vertices = generate_triangle_strip(&left_outline, &right_outline, &stroke.color);

    StrokeGeometry {
        left_outline,
        right_outline,
        widths,
        vertices,
        start_cap,
        end_cap,
    }
}

/// Generate geometry directly from raw points (for live preview during drawing)
pub fn generate_live_geometry(
    points: &[StrokePoint],
    tool: ToolType,
    color: Color,
    base_width: f32,
    config: &GeometryConfig,
) -> StrokeGeometry {
    let mut left_outline = Vec::new();
    let mut right_outline = Vec::new();
    let mut widths = Vec::new();

    if points.len() < 2 {
        return StrokeGeometry {
            left_outline,
            right_outline,
            widths,
            vertices: Vec::new(),
            start_cap: Vec::new(),
            end_cap: Vec::new(),
        };
    }

    for i in 0..points.len() {
        let p = &points[i];

        // Calculate normal from neighboring points
        let (nx, ny) = if i == 0 {
            let dx = points[1].x - points[0].x;
            let dy = points[1].y - points[0].y;
            let len = (dx * dx + dy * dy).sqrt().max(1e-10);
            (-dy / len, dx / len)
        } else if i == points.len() - 1 {
            let dx = points[i].x - points[i - 1].x;
            let dy = points[i].y - points[i - 1].y;
            let len = (dx * dx + dy * dy).sqrt().max(1e-10);
            (-dy / len, dx / len)
        } else {
            let dx = points[i + 1].x - points[i - 1].x;
            let dy = points[i + 1].y - points[i - 1].y;
            let len = (dx * dx + dy * dy).sqrt().max(1e-10);
            (-dy / len, dx / len)
        };

        // Width from pressure
        let width = match tool {
            ToolType::Pen => base_width * (0.3 + 0.7 * p.pressure),
            ToolType::Brush => base_width * (0.1 + 0.9 * p.pressure),
            ToolType::Pencil => base_width * (0.5 + 0.5 * p.pressure),
            ToolType::Marker => base_width,
            ToolType::Highlighter => base_width * 3.0,
            ToolType::Eraser => base_width * 2.0,
        };

        // Apply velocity damping
        let width = if i >= 2 && config.velocity_damping > 0.0 {
            let vel = points[i - 1].velocity_to(p) * 1e6; // px/sec
            let vel_factor = 1.0 - config.velocity_damping as f64 * (vel * 0.001).min(1.0);
            (width as f64 * vel_factor).clamp(config.min_width as f64, config.max_width as f64) as f32
        } else {
            width
        };

        let half_w = width as f64 / 2.0;
        left_outline.push((p.x + nx * half_w, p.y + ny * half_w));
        right_outline.push((p.x - nx * half_w, p.y - ny * half_w));
        widths.push(width);
    }

    let vertices = generate_triangle_strip(&left_outline, &right_outline, &color);

    StrokeGeometry {
        left_outline,
        right_outline,
        widths,
        vertices,
        start_cap: Vec::new(),
        end_cap: Vec::new(),
    }
}

#[derive(Debug)]
struct SamplePoint {
    x: f64,
    y: f64,
    nx: f64,
    ny: f64,
    width: f32,
    #[allow(dead_code)]
    pressure: f32,
}

fn generate_round_cap(sample: &SamplePoint, is_start: bool, resolution: usize) -> Vec<(f64, f64)> {
    let half_w = sample.width as f64 / 2.0;
    let mut cap = Vec::with_capacity(resolution + 1);
    let base_angle = sample.ny.atan2(sample.nx);
    let angle_range = std::f64::consts::PI;

    let start_angle = if is_start {
        base_angle + std::f64::consts::FRAC_PI_2
    } else {
        base_angle - std::f64::consts::FRAC_PI_2
    };

    for i in 0..=resolution {
        let t = i as f64 / resolution as f64;
        let angle = start_angle + angle_range * t;
        cap.push((
            sample.x + angle.cos() * half_w,
            sample.y + angle.sin() * half_w,
        ));
    }
    cap
}

fn generate_triangle_strip(
    left: &[(f64, f64)],
    right: &[(f64, f64)],
    color: &Color,
) -> Vec<StrokeVertex> {
    let mut vertices = Vec::with_capacity(left.len() * 2);
    let n = left.len().min(right.len());

    for i in 0..n {
        vertices.push(StrokeVertex {
            x: left[i].0 as f32,
            y: left[i].1 as f32,
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        });
        vertices.push(StrokeVertex {
            x: right[i].0 as f32,
            y: right[i].1 as f32,
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        });
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::BezierSpline;
    use uuid::Uuid;

    fn make_test_stroke() -> (Stroke, BezierSpline) {
        let mut stroke = Stroke::new(ToolType::Pen, Color::BLACK, 4.0, Uuid::new_v4());
        let points = vec![
            StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 },
            StrokePoint { x: 50.0, y: 20.0, pressure: 0.8, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 8000 },
            StrokePoint { x: 100.0, y: 0.0, pressure: 0.6, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 16000 },
            StrokePoint { x: 150.0, y: 30.0, pressure: 0.3, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 24000 },
        ];
        for p in &points {
            stroke.add_point(*p);
        }
        let spline = BezierSpline::fit_from_points(&points);
        (stroke, spline)
    }

    #[test]
    fn test_geometry_generation() {
        let (stroke, spline) = make_test_stroke();
        let config = GeometryConfig::default();
        let geom = generate_stroke_geometry(&stroke, &spline, &config);

        assert!(!geom.left_outline.is_empty());
        assert!(!geom.right_outline.is_empty());
        assert_eq!(geom.left_outline.len(), geom.right_outline.len());
        assert_eq!(geom.vertices.len(), geom.left_outline.len() * 2);
    }

    #[test]
    fn test_live_geometry() {
        let points = vec![
            StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 },
            StrokePoint { x: 30.0, y: 10.0, pressure: 0.7, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 8000 },
            StrokePoint { x: 60.0, y: 0.0, pressure: 0.4, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 16000 },
        ];
        let config = GeometryConfig::default();
        let geom = generate_live_geometry(&points, ToolType::Pen, Color::BLACK, 3.0, &config);

        assert_eq!(geom.left_outline.len(), 3);
        assert_eq!(geom.right_outline.len(), 3);
    }

    #[test]
    fn test_variable_width() {
        let (stroke, spline) = make_test_stroke();
        let config = GeometryConfig { velocity_damping: 0.0, ..Default::default() };
        let geom = generate_stroke_geometry(&stroke, &spline, &config);

        // Widths should vary with pressure
        let min_w = geom.widths.iter().cloned().fold(f32::MAX, f32::min);
        let max_w = geom.widths.iter().cloned().fold(f32::MIN, f32::max);
        assert!(max_w > min_w, "Width should vary with pressure");
    }
}
