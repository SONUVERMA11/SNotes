//! Lasso selection tool — select, move, scale, rotate, copy/paste strokes

use crate::ink::Stroke;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transform applied to a selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelectionTransform {
    pub translate_x: f64,
    pub translate_y: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation: f64,
    pub pivot_x: f64,
    pub pivot_y: f64,
}

impl Default for SelectionTransform {
    fn default() -> Self {
        Self { translate_x: 0.0, translate_y: 0.0, scale_x: 1.0, scale_y: 1.0, rotation: 0.0, pivot_x: 0.0, pivot_y: 0.0 }
    }
}

impl SelectionTransform {
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        let px = x - self.pivot_x;
        let py = y - self.pivot_y;
        let sx = px * self.scale_x;
        let sy = py * self.scale_y;
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rx = sx * cos_r - sy * sin_r;
        let ry = sx * sin_r + sy * cos_r;
        (rx + self.pivot_x + self.translate_x, ry + self.pivot_y + self.translate_y)
    }
}

/// Selection state
#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub stroke_ids: Vec<Uuid>,
    pub bounds: (f64, f64, f64, f64),
    pub lasso_points: Vec<(f64, f64)>,
    pub transform: SelectionTransform,
}

impl Selection {
    pub fn new() -> Self { Self::default() }

    pub fn select_rect(strokes: &[Stroke], x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let (min_x, min_y) = (x1.min(x2), y1.min(y2));
        let (max_x, max_y) = (x1.max(x2), y1.max(y2));
        let selected: Vec<Uuid> = strokes.iter()
            .filter(|s| s.bounds.0 >= min_x && s.bounds.1 >= min_y && s.bounds.2 <= max_x && s.bounds.3 <= max_y)
            .map(|s| s.id).collect();
        Self {
            stroke_ids: selected, bounds: (min_x, min_y, max_x, max_y),
            lasso_points: Vec::new(),
            transform: SelectionTransform { pivot_x: (min_x + max_x) / 2.0, pivot_y: (min_y + max_y) / 2.0, ..Default::default() },
        }
    }

    pub fn select_lasso(strokes: &[Stroke], lasso: &[(f64, f64)]) -> Self {
        let mut selected = Vec::new();
        let (mut min_x, mut min_y) = (f64::MAX, f64::MAX);
        let (mut max_x, mut max_y) = (f64::MIN, f64::MIN);
        for stroke in strokes {
            let cx = (stroke.bounds.0 + stroke.bounds.2) / 2.0;
            let cy = (stroke.bounds.1 + stroke.bounds.3) / 2.0;
            if point_in_polygon(cx, cy, lasso) {
                selected.push(stroke.id);
                min_x = min_x.min(stroke.bounds.0); min_y = min_y.min(stroke.bounds.1);
                max_x = max_x.max(stroke.bounds.2); max_y = max_y.max(stroke.bounds.3);
            }
        }
        Self {
            stroke_ids: selected, bounds: (min_x, min_y, max_x, max_y),
            lasso_points: lasso.to_vec(),
            transform: SelectionTransform { pivot_x: (min_x + max_x) / 2.0, pivot_y: (min_y + max_y) / 2.0, ..Default::default() },
        }
    }

    pub fn apply_transform(&self, strokes: &mut [Stroke]) {
        for stroke in strokes.iter_mut() {
            if !self.stroke_ids.contains(&stroke.id) { continue; }
            stroke.bounds = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
            for point in &mut stroke.points {
                let (nx, ny) = self.transform.apply(point.x, point.y);
                point.x = nx; point.y = ny;
                stroke.bounds.0 = stroke.bounds.0.min(nx); stroke.bounds.1 = stroke.bounds.1.min(ny);
                stroke.bounds.2 = stroke.bounds.2.max(nx); stroke.bounds.3 = stroke.bounds.3.max(ny);
            }
        }
    }

    pub fn duplicate_strokes(&self, strokes: &[Stroke], offset: (f64, f64)) -> Vec<Stroke> {
        strokes.iter().filter(|s| self.stroke_ids.contains(&s.id)).map(|s| {
            let mut copy = s.clone();
            copy.id = Uuid::new_v4();
            copy.bounds = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
            for p in &mut copy.points {
                p.x += offset.0; p.y += offset.1;
                copy.bounds.0 = copy.bounds.0.min(p.x); copy.bounds.1 = copy.bounds.1.min(p.y);
                copy.bounds.2 = copy.bounds.2.max(p.x); copy.bounds.3 = copy.bounds.3.max(p.y);
            }
            copy
        }).collect()
    }

    pub fn is_empty(&self) -> bool { self.stroke_ids.is_empty() }
    pub fn center(&self) -> (f64, f64) { ((self.bounds.0 + self.bounds.2) / 2.0, (self.bounds.1 + self.bounds.3) / 2.0) }
}

fn point_in_polygon(px: f64, py: f64, polygon: &[(f64, f64)]) -> bool {
    let n = polygon.len();
    if n < 3 { return false; }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) { inside = !inside; }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::{Color, ToolType, StrokePoint};

    #[test]
    fn test_point_in_polygon() {
        let sq = vec![(0.0,0.0),(100.0,0.0),(100.0,100.0),(0.0,100.0)];
        assert!(point_in_polygon(50.0, 50.0, &sq));
        assert!(!point_in_polygon(150.0, 50.0, &sq));
    }

    #[test]
    fn test_rect_selection() {
        let mut s1 = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
        s1.add_point(StrokePoint { x:10.0, y:10.0, pressure:0.5, tilt_x:0.0, tilt_y:0.0, timestamp_us:0 });
        s1.add_point(StrokePoint { x:30.0, y:30.0, pressure:0.5, tilt_x:0.0, tilt_y:0.0, timestamp_us:1000 });
        let sel = Selection::select_rect(&[s1], 0.0, 0.0, 50.0, 50.0);
        assert_eq!(sel.stroke_ids.len(), 1);
    }

    #[test]
    fn test_transform() {
        let t = SelectionTransform { translate_x: 10.0, translate_y: 20.0, ..Default::default() };
        let (nx, ny) = t.apply(5.0, 5.0);
        assert!((nx - 15.0).abs() < 1e-10);
        assert!((ny - 25.0).abs() < 1e-10);
    }
}
