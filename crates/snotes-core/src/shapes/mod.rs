//! Shape recognition: circle, rectangle, triangle, arrow, line

use crate::ink::StrokePoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecognizedShape {
    Line { start: (f64, f64), end: (f64, f64) },
    Circle { center: (f64, f64), radius: f64 },
    Rectangle { x: f64, y: f64, width: f64, height: f64 },
    Triangle { p1: (f64, f64), p2: (f64, f64), p3: (f64, f64) },
    Arrow { start: (f64, f64), end: (f64, f64) },
}

/// Attempt to recognize a shape from stroke points
pub fn recognize_shape(points: &[StrokePoint], tolerance: f64) -> Option<RecognizedShape> {
    if points.len() < 3 { return None; }

    // Check for line first (simplest)
    if is_line(points, tolerance) {
        let start = (points.first()?.x, points.first()?.y);
        let end = (points.last()?.x, points.last()?.y);
        return Some(RecognizedShape::Line { start, end });
    }

    // Check for circle
    if let Some(shape) = detect_circle(points, tolerance) {
        return Some(shape);
    }

    // Check for rectangle
    if let Some(shape) = detect_rectangle(points, tolerance) {
        return Some(shape);
    }

    None
}

fn is_line(points: &[StrokePoint], tolerance: f64) -> bool {
    if points.len() < 2 { return false; }
    let start = &points[0];
    let end = &points[points.len() - 1];
    let line_len = start.distance_to(end);
    if line_len < 10.0 { return false; }

    let max_deviation = points.iter().map(|p| {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let t = ((p.x - start.x) * dx + (p.y - start.y) * dy) / (dx * dx + dy * dy);
        let t = t.clamp(0.0, 1.0);
        let proj_x = start.x + t * dx;
        let proj_y = start.y + t * dy;
        ((p.x - proj_x).powi(2) + (p.y - proj_y).powi(2)).sqrt()
    }).fold(0.0_f64, f64::max);

    max_deviation / line_len < tolerance
}

fn detect_circle(points: &[StrokePoint], tolerance: f64) -> Option<RecognizedShape> {
    // Compute centroid
    let n = points.len() as f64;
    let cx = points.iter().map(|p| p.x).sum::<f64>() / n;
    let cy = points.iter().map(|p| p.y).sum::<f64>() / n;

    // Average radius
    let avg_r = points.iter()
        .map(|p| ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt())
        .sum::<f64>() / n;

    // Check variance
    let variance = points.iter()
        .map(|p| {
            let r = ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt();
            ((r - avg_r) / avg_r).abs()
        })
        .sum::<f64>() / n;

    // Check if stroke is closed
    let first = &points[0];
    let last = &points[points.len() - 1];
    let closure = first.distance_to(last) / avg_r;

    if variance < tolerance && closure < 0.3 {
        Some(RecognizedShape::Circle { center: (cx, cy), radius: avg_r })
    } else {
        None
    }
}

fn detect_rectangle(points: &[StrokePoint], tolerance: f64) -> Option<RecognizedShape> {
    // Find bounding box
    let min_x = points.iter().map(|p| p.x).fold(f64::MAX, f64::min);
    let min_y = points.iter().map(|p| p.y).fold(f64::MAX, f64::min);
    let max_x = points.iter().map(|p| p.x).fold(f64::MIN, f64::max);
    let max_y = points.iter().map(|p| p.y).fold(f64::MIN, f64::max);

    let w = max_x - min_x;
    let h = max_y - min_y;
    if w < 10.0 || h < 10.0 { return None; }

    // Check if points are close to the bounding box edges
    let max_deviation = points.iter().map(|p| {
        let dx = (p.x - min_x).min(max_x - p.x);
        let dy = (p.y - min_y).min(max_y - p.y);
        dx.min(dy)
    }).fold(0.0_f64, f64::max);

    let perimeter = 2.0 * (w + h);
    if max_deviation / perimeter < tolerance {
        Some(RecognizedShape::Rectangle { x: min_x, y: min_y, width: w, height: h })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_detection() {
        let points: Vec<StrokePoint> = (0..20).map(|i| StrokePoint {
            x: i as f64 * 10.0, y: i as f64 * 10.0 + (i as f64 * 0.1).sin(),
            pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: i * 1000,
        }).collect();
        assert!(is_line(&points, 0.05));
    }
}
