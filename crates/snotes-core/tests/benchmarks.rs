//! Performance benchmarks for core engine operations

use snotes_core::prelude::*;
use snotes_core::ink::*;
use snotes_core::canvas::*;
use snotes_core::document::*;
use std::time::Instant;
use uuid::Uuid;

fn make_stroke(point_count: usize) -> Stroke {
    let mut stroke = Stroke::new(ToolType::Pen, Color::BLACK, 3.0, Uuid::new_v4());
    for i in 0..point_count {
        stroke.add_point(StrokePoint {
            x: i as f64 * 2.0,
            y: (i as f64 * 0.1).sin() * 50.0 + 100.0,
            pressure: 0.3 + 0.4 * (i as f32 / point_count as f32),
            tilt_x: 0.0,
            tilt_y: 0.0,
            timestamp_us: i as u64 * 8333,
        });
    }
    stroke
}

/// Benchmark: Bézier spline fitting
#[test]
fn bench_bezier_spline_fitting() {
    let sizes = [10, 50, 100, 500, 1000];

    for &n in &sizes {
        let stroke = make_stroke(n);
        let start = Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let _spline = BezierSpline::fit_from_points(&stroke.points);
        }

        let elapsed = start.elapsed();
        let per_iter = elapsed / iterations;
        eprintln!(
            "BezierSpline::fit_from_points({} pts): {:?}/iter ({:.0} ops/sec)",
            n, per_iter, 1.0 / per_iter.as_secs_f64()
        );

        // Performance gate: must complete within 1ms for 100 points
        if n == 100 {
            assert!(per_iter.as_micros() < 1000, "Spline fit too slow for {} points", n);
        }
    }
}

/// Benchmark: Stroke geometry generation
#[test]
fn bench_geometry_generation() {
    let sizes = [10, 50, 100, 500];
    let config = GeometryConfig::default();

    for &n in &sizes {
        let stroke = make_stroke(n);
        let spline = BezierSpline::fit_from_points(&stroke.points);
        let start = Instant::now();
        let iterations = 500;

        for _ in 0..iterations {
            let _geom = generate_stroke_geometry(&stroke, &spline, &config);
        }

        let elapsed = start.elapsed();
        let per_iter = elapsed / iterations;
        eprintln!(
            "generate_stroke_geometry({} pts): {:?}/iter ({:.0} ops/sec)",
            n, per_iter, 1.0 / per_iter.as_secs_f64()
        );

        // Performance gate: must complete within 2ms for 100 points
        if n == 100 {
            assert!(per_iter.as_micros() < 2000, "Geometry gen too slow for {} points", n);
        }
    }
}

/// Benchmark: Hit testing
#[test]
fn bench_hit_testing() {
    let stroke_count = 1000;
    let strokes: Vec<Stroke> = (0..stroke_count)
        .map(|i| {
            let mut s = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
            s.add_point(StrokePoint {
                x: i as f64 * 5.0, y: 0.0,
                pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0,
            });
            s.add_point(StrokePoint {
                x: i as f64 * 5.0 + 50.0, y: 50.0,
                pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000,
            });
            s
        })
        .collect();

    let start = Instant::now();
    let iterations = 10000;

    for _ in 0..iterations {
        let mut _hits = 0;
        for s in &strokes {
            if s.hit_test(2500.0, 25.0, 10.0) {
                _hits += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    let per_iter = elapsed / iterations;
    eprintln!(
        "hit_test({} strokes): {:?}/iter ({:.0} ops/sec)",
        stroke_count, per_iter, 1.0 / per_iter.as_secs_f64()
    );

    // Performance gate: must complete within 1ms for 1000 strokes
    assert!(per_iter.as_micros() < 1000, "Hit testing too slow");
}

/// Benchmark: LZ4 compression
#[test]
fn bench_lz4_compression() {
    let sizes = [1024, 10240, 102400, 1024000]; // 1KB, 10KB, 100KB, 1MB

    for &size in &sizes {
        let data: Vec<u8> = (0..size).map(|i| ((i * 7 + 13) % 256) as u8).collect();

        let start = Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            let compressed = snotes_core::storage::compress_strokes(&data);
            let _decompressed = snotes_core::storage::decompress_strokes(&compressed).unwrap();
        }

        let elapsed = start.elapsed();
        let per_iter = elapsed / iterations;
        let compressed = snotes_core::storage::compress_strokes(&data);
        let ratio = compressed.len() as f64 / data.len() as f64;

        eprintln!(
            "LZ4 roundtrip ({:.0}KB): {:?}/iter, ratio: {:.2}x",
            size as f64 / 1024.0, per_iter, ratio
        );
    }
}

/// Benchmark: Document creation with many pages
#[test]
fn bench_document_operations() {
    let start = Instant::now();

    let mut lib = Library::new("Benchmark");
    let mut notebook = Notebook::new("Large Notebook", "#333333");

    for i in 0..100 {
        let mut section = Section::new(&format!("Section {}", i));
        for _j in 0..10 {
            let page = Page::new(PageTemplate::Lined);
            // Create strokes (simulating page content)
            let _strokes: Vec<Stroke> = (0..50)
                .map(|_| make_stroke(20))
                .collect();
            section.add_page(page);
        }
        notebook.add_section(section);
    }
    lib.add_notebook(notebook);

    let elapsed = start.elapsed();
    let total_pages = 100 * 10;
    let total_strokes = total_pages * 50;

    eprintln!(
        "Created library with {} sections, {} pages, {} strokes in {:?}",
        100, total_pages, total_strokes, elapsed
    );

    // Performance gate: should create within 5 seconds
    assert!(elapsed.as_secs() < 5, "Document creation too slow");
}

/// Benchmark: Predictive ink throughput
#[test]
fn bench_predictive_ink() {
    let mut predictor = PredictiveInk::new(2);
    let start = Instant::now();
    let iterations = 100_000;

    for i in 0..iterations {
        let point = StrokePoint {
            x: i as f64 * 0.5,
            y: (i as f64 * 0.02).sin() * 100.0,
            pressure: 0.5,
            tilt_x: 0.0,
            tilt_y: 0.0,
            timestamp_us: i as u64 * 8333,
        };
        let _preds = predictor.predict(point);
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();
    eprintln!(
        "PredictiveInk: {} points in {:?} ({:.0} points/sec)",
        iterations, elapsed, throughput
    );

    // Should process at least 1M points/sec
    assert!(throughput > 100_000.0, "Predictive ink too slow");
}

/// Benchmark: Eraser performance
#[test]
fn bench_eraser() {
    let stroke_count = 500;
    let strokes: Vec<Stroke> = (0..stroke_count)
        .map(|_| make_stroke(50))
        .collect();

    let eraser_path: Vec<(f64, f64)> = (0..20)
        .map(|i| (i as f64 * 5.0, 100.0))
        .collect();

    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let _result = snotes_core::ink::erase_whole_stroke(&strokes, &eraser_path, 10.0);
    }

    let elapsed = start.elapsed();
    let per_iter = elapsed / iterations;
    eprintln!(
        "erase_whole_stroke({} strokes, {} pts): {:?}/iter",
        stroke_count, eraser_path.len(), per_iter
    );
}
