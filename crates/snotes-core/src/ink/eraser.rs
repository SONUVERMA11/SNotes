//! Eraser system — whole-stroke delete and pixel-level erase with splitting

use super::{Stroke, StrokePoint};
use uuid::Uuid;

/// Result of an erase operation
#[derive(Debug)]
pub enum EraseResult {
    /// No strokes were affected
    NoHit,
    /// Entire strokes were deleted (whole-stroke mode)
    WholeStrokeDeleted {
        deleted_ids: Vec<Uuid>,
    },
    /// Strokes were split (pixel-level erase)
    PixelErased {
        deleted_ids: Vec<Uuid>,
        new_strokes: Vec<Stroke>,
    },
}

/// Perform whole-stroke erase: delete any strokes the eraser path touches
pub fn erase_whole_stroke(
    strokes: &[Stroke],
    eraser_path: &[(f64, f64)],
    eraser_radius: f64,
) -> EraseResult {
    let mut deleted_ids = Vec::new();

    for stroke in strokes {
        for &(ex, ey) in eraser_path {
            if stroke.hit_test(ex, ey, eraser_radius) {
                deleted_ids.push(stroke.id);
                break;
            }
        }
    }

    if deleted_ids.is_empty() {
        EraseResult::NoHit
    } else {
        EraseResult::WholeStrokeDeleted { deleted_ids }
    }
}

/// Perform pixel-level erase: split strokes where the eraser intersects
pub fn erase_pixel_level(
    strokes: &[Stroke],
    eraser_path: &[(f64, f64)],
    eraser_radius: f64,
) -> EraseResult {
    let mut deleted_ids = Vec::new();
    let mut new_strokes = Vec::new();

    for stroke in strokes {
        let segments = split_stroke_by_eraser(stroke, eraser_path, eraser_radius);

        if segments.len() == 1 && segments[0].len() == stroke.points.len() {
            // No intersection, keep original
            continue;
        }

        if segments.is_empty() {
            // Entirely erased
            deleted_ids.push(stroke.id);
            continue;
        }

        // Mark original for deletion and create new sub-strokes
        deleted_ids.push(stroke.id);

        for segment_points in segments {
            if segment_points.len() < 2 {
                continue; // Too short to form a stroke
            }

            let mut new_stroke = Stroke::new(
                stroke.tool,
                stroke.color,
                stroke.base_width,
                stroke.layer_id,
            );
            for point in segment_points {
                new_stroke.add_point(point);
            }
            new_strokes.push(new_stroke);
        }
    }

    if deleted_ids.is_empty() {
        EraseResult::NoHit
    } else {
        EraseResult::PixelErased {
            deleted_ids,
            new_strokes,
        }
    }
}

/// Split a stroke into segments where the eraser doesn't intersect
fn split_stroke_by_eraser(
    stroke: &Stroke,
    eraser_path: &[(f64, f64)],
    eraser_radius: f64,
) -> Vec<Vec<StrokePoint>> {
    let mut segments: Vec<Vec<StrokePoint>> = Vec::new();
    let mut current_segment: Vec<StrokePoint> = Vec::new();

    for point in &stroke.points {
        let erased = eraser_path.iter().any(|&(ex, ey)| {
            let dx = point.x - ex;
            let dy = point.y - ey;
            (dx * dx + dy * dy).sqrt() <= eraser_radius + stroke.base_width as f64 / 2.0
        });

        if erased {
            // Point is erased, finalize current segment if it exists
            if current_segment.len() >= 2 {
                segments.push(std::mem::take(&mut current_segment));
            } else {
                current_segment.clear();
            }
        } else {
            current_segment.push(*point);
        }
    }

    // Don't forget the last segment
    if current_segment.len() >= 2 {
        segments.push(current_segment);
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::{Color, ToolType};

    fn make_horizontal_stroke() -> Stroke {
        let mut stroke = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
        for i in 0..20 {
            stroke.add_point(StrokePoint {
                x: i as f64 * 10.0,
                y: 50.0,
                pressure: 0.5,
                tilt_x: 0.0,
                tilt_y: 0.0,
                timestamp_us: i * 1000,
            });
        }
        stroke
    }

    #[test]
    fn test_whole_stroke_erase_hit() {
        let stroke = make_horizontal_stroke();
        let result = erase_whole_stroke(&[stroke], &[(50.0, 50.0)], 10.0);
        match result {
            EraseResult::WholeStrokeDeleted { deleted_ids } => {
                assert_eq!(deleted_ids.len(), 1);
            }
            _ => panic!("Expected WholeStrokeDeleted"),
        }
    }

    #[test]
    fn test_whole_stroke_erase_miss() {
        let stroke = make_horizontal_stroke();
        let result = erase_whole_stroke(&[stroke], &[(50.0, 200.0)], 5.0);
        assert!(matches!(result, EraseResult::NoHit));
    }

    #[test]
    fn test_pixel_erase_split() {
        let stroke = make_horizontal_stroke();
        // Erase the middle of the stroke
        let result = erase_pixel_level(
            &[stroke],
            &[(100.0, 50.0)],
            15.0,
        );
        match result {
            EraseResult::PixelErased { deleted_ids, new_strokes } => {
                assert_eq!(deleted_ids.len(), 1);
                assert_eq!(new_strokes.len(), 2, "Should split into 2 segments");
            }
            _ => panic!("Expected PixelErased"),
        }
    }

    #[test]
    fn test_pixel_erase_total() {
        let mut stroke = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
        stroke.add_point(StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 });
        stroke.add_point(StrokePoint { x: 5.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 });

        let result = erase_pixel_level(&[stroke], &[(2.5, 0.0)], 10.0);
        match result {
            EraseResult::PixelErased { deleted_ids, new_strokes } => {
                assert_eq!(deleted_ids.len(), 1);
                assert!(new_strokes.is_empty(), "Entire stroke erased");
            }
            _ => panic!("Expected PixelErased"),
        }
    }
}
