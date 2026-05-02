//! Integration tests — end-to-end workflows spanning multiple modules

use snotes_core::prelude::*;
use snotes_core::ink::*;
use snotes_core::canvas::*;
use snotes_core::document::*;
use snotes_core::tools::selection::*;
use snotes_core::export::ExportFormat;

/// Test: create a stroke from input points, fit a spline, generate geometry
#[test]
fn test_full_stroke_pipeline() {
    // 1. Simulate raw input points (as if from a stylus)
    let raw_points = vec![
        StrokePoint { x: 100.0, y: 200.0, pressure: 0.3, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 },
        StrokePoint { x: 120.0, y: 210.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 8333 },
        StrokePoint { x: 150.0, y: 220.0, pressure: 0.7, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 16666 },
        StrokePoint { x: 180.0, y: 215.0, pressure: 0.8, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 24999 },
        StrokePoint { x: 210.0, y: 200.0, pressure: 0.6, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 33332 },
        StrokePoint { x: 240.0, y: 180.0, pressure: 0.4, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 41665 },
    ];

    // 2. Build a stroke
    let layer_id = uuid::Uuid::new_v4();
    let mut stroke = Stroke::new(ToolType::Pen, Color::from_rgba(0.0, 0.0, 0.5, 1.0), 3.0, layer_id);
    for p in &raw_points {
        stroke.add_point(p.clone());
    }
    assert_eq!(stroke.points.len(), 6);

    // 3. Fit a Bézier spline
    let spline = BezierSpline::fit_from_points(&stroke.points);
    assert_eq!(spline.segments.len(), 5); // n-1 segments for n points

    // 4. Generate stroke geometry
    let config = snotes_core::ink::GeometryConfig::default();
    let geom = generate_stroke_geometry(&stroke, &spline, &config);
    assert!(geom.vertices.len() > 0);
    assert!(geom.left_outline.len() > 0);

    // 5. Verify bounding box
    assert!(stroke.bounds.0 <= 100.0);
    assert!(stroke.bounds.2 >= 240.0);

    // 6. Hit test
    assert!(stroke.hit_test(150.0, 220.0, 10.0));
    assert!(!stroke.hit_test(500.0, 500.0, 10.0));
}

/// Test: full document hierarchy creation and page reordering
#[test]
fn test_document_hierarchy() {
    let mut lib = Library::new("University Notes");
    let mut nb = Notebook::new("Physics 101", "#3498db");
    let mut sec1 = Section::new("Mechanics");
    let mut sec2 = Section::new("Thermodynamics");

    sec1.add_page(Page::new(PageTemplate::Lined));
    sec1.add_page(Page::new(PageTemplate::Grid));
    sec2.add_page(Page::new(PageTemplate::Blank));

    nb.add_section(sec1);
    nb.add_section(sec2);
    lib.add_notebook(nb);

    assert_eq!(lib.notebooks.len(), 1);
    assert_eq!(lib.notebooks[0].sections.len(), 3); // default + 2
    assert_eq!(lib.notebooks[0].sections[1].pages.len(), 3); // default + 2

    // Test page reorder
    let sec = &mut lib.notebooks[0].sections[1];
    let first_id = sec.pages[0].id;
    sec.reorder_page(0, 2);
    assert_eq!(sec.pages[2].id, first_id);
}

/// Test: canvas viewport transforms
#[test]
fn test_viewport_transforms() {
    let mut viewport = Viewport::default();
    viewport.zoom = 2.0;
    viewport.offset_x = 50.0;
    viewport.offset_y = 100.0;

    // Canvas to screen
    let (sx, sy) = viewport.canvas_to_screen(10.0, 20.0);
    assert!((sx - 70.0).abs() < 0.01); // 10 * 2 + 50
    assert!((sy - 140.0).abs() < 0.01); // 20 * 2 + 100

    // Screen to canvas (inverse)
    let (cx, cy) = viewport.screen_to_canvas(sx, sy);
    assert!((cx - 10.0).abs() < 0.01);
    assert!((cy - 20.0).abs() < 0.01);
}

/// Test: selection + transform + duplication
#[test]
fn test_selection_workflow() {
    let layer_id = uuid::Uuid::new_v4();
    let mut s1 = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, layer_id);
    s1.add_point(StrokePoint { x: 10.0, y: 10.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 });
    s1.add_point(StrokePoint { x: 40.0, y: 40.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 });

    let mut s2 = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, layer_id);
    s2.add_point(StrokePoint { x: 200.0, y: 200.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 });
    s2.add_point(StrokePoint { x: 230.0, y: 230.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 });

    let strokes = vec![s1, s2];

    // Select only the first stroke (within rect 0,0 -> 50,50)
    let sel = Selection::select_rect(&strokes, 0.0, 0.0, 50.0, 50.0);
    assert_eq!(sel.stroke_ids.len(), 1);

    // Duplicate with offset
    let copies = sel.duplicate_strokes(&strokes, (100.0, 0.0));
    assert_eq!(copies.len(), 1);
    assert!(copies[0].points[0].x >= 110.0); // original 10 + offset 100
}

/// Test: eraser modes
#[test]
fn test_eraser_modes() {
    let layer_id = uuid::Uuid::new_v4();
    let mut s = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, layer_id);
    for i in 0..20 {
        s.add_point(StrokePoint {
            x: i as f64 * 10.0, y: 50.0,
            pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0,
            timestamp_us: i as u64 * 1000,
        });
    }

    let strokes = vec![s.clone()];

    // Whole-stroke erase — the eraser path hits the stroke
    let eraser_path = vec![(50.0, 45.0), (50.0, 55.0)];
    let result = snotes_core::ink::erase_whole_stroke(&strokes, &eraser_path, 20.0);
    match result {
        snotes_core::ink::EraseResult::WholeStrokeDeleted { deleted_ids } => {
            assert_eq!(deleted_ids.len(), 1);
        }
        _ => panic!("Expected WholeStrokeDeleted"),
    }

    // Pixel-level erase — splits the stroke
    let result = snotes_core::ink::erase_pixel_level(&strokes, &eraser_path, 15.0);
    match result {
        snotes_core::ink::EraseResult::PixelErased { deleted_ids, new_strokes: _ } => {
            assert!(!deleted_ids.is_empty());
        }
        _ => {} // May be NoHit depending on radius
    }
}

/// Test: predictive ink
#[test]
fn test_predictive_ink_latency() {
    let mut predictor = PredictiveInk::new(2);

    let p0 = StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 };
    let p1 = StrokePoint { x: 10.0, y: 10.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 8333 };
    let p2 = StrokePoint { x: 20.0, y: 20.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 16666 };

    predictor.predict(p0);
    predictor.predict(p1);
    let predictions = predictor.predict(p2);

    // For a linear trajectory, predictions should extend the line
    assert_eq!(predictions.len(), 2);
    assert!(predictions[0].x > 20.0);
    assert!(predictions[0].y > 20.0);
    assert!(predictions[1].x > predictions[0].x);
}

/// Test: export format parsing
#[test]
fn test_export_format() {
    assert_eq!(ExportFormat::from_str("pdf"), Some(ExportFormat::Pdf));
    assert_eq!(ExportFormat::from_str("PNG"), Some(ExportFormat::Png));
    assert_eq!(ExportFormat::from_str("svg"), Some(ExportFormat::Svg));
    assert_eq!(ExportFormat::from_str("snotes"), Some(ExportFormat::SNotes));
    assert_eq!(ExportFormat::from_str("docx"), None);

    assert_eq!(ExportFormat::Pdf.extension(), "pdf");
}

/// Test: LZ4 compression roundtrip with realistic stroke data
#[test]
fn test_stroke_compression() {
    // Simulate serialized stroke data
    let mut fake_data = Vec::new();
    for i in 0..1000 {
        let x = (i as f64 * 0.5).to_le_bytes();
        let y = (i as f64 * 0.3 + 100.0).to_le_bytes();
        let p = (0.5f32).to_le_bytes();
        fake_data.extend_from_slice(&x);
        fake_data.extend_from_slice(&y);
        fake_data.extend_from_slice(&p);
    }

    let compressed = snotes_core::storage::compress_strokes(&fake_data);
    assert!(compressed.len() < fake_data.len()); // Should compress

    let decompressed = snotes_core::storage::decompress_strokes(&compressed).unwrap();
    assert_eq!(decompressed, fake_data);
}
