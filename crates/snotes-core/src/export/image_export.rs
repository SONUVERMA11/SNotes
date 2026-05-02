//! Image export — render strokes to PNG and SVG

use crate::ink::{Stroke, StrokePoint, Color};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageExportError {
    #[error("Export failed: {0}")]
    Failed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image error: {0}")]
    Image(String),
}

/// Export strokes as SVG
pub fn export_strokes_to_svg(
    strokes: &[Stroke],
    width: f64,
    height: f64,
    background: Color,
    output_path: &str,
) -> Result<String, ImageExportError> {
    let mut svg = String::new();

    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
  <rect width="{w}" height="{h}" fill="rgba({r},{g},{b},{a})"/>
"#,
        w = width as u32,
        h = height as u32,
        r = (background.r * 255.0) as u8,
        g = (background.g * 255.0) as u8,
        b = (background.b * 255.0) as u8,
        a = background.a,
    ));

    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }

        let path_data = points_to_svg_path(&stroke.points);
        let color = stroke.color.to_hex();
        let opacity = stroke.color.a;

        svg.push_str(&format!(
            r#"  <path d="{path}" fill="none" stroke="{color}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round" opacity="{opacity}"/>
"#,
            path = path_data,
            color = &color[..7], // strip alpha from hex
            width = stroke.base_width,
            opacity = opacity,
        ));
    }

    svg.push_str("</svg>\n");

    std::fs::write(output_path, &svg)?;
    log::info!("SVG exported to: {}", output_path);
    Ok(output_path.to_string())
}

/// Export strokes as PNG using the image crate
pub fn export_strokes_to_png(
    strokes: &[Stroke],
    width: u32,
    height: u32,
    background: Color,
    scale: f64,
    output_path: &str,
) -> Result<String, ImageExportError> {
    let w = (width as f64 * scale) as u32;
    let h = (height as f64 * scale) as u32;

    let mut imgbuf = image::RgbaImage::new(w, h);

    // Fill background
    let bg = image::Rgba([
        (background.r * 255.0) as u8,
        (background.g * 255.0) as u8,
        (background.b * 255.0) as u8,
        (background.a * 255.0) as u8,
    ]);
    for pixel in imgbuf.pixels_mut() {
        *pixel = bg;
    }

    // Render strokes (simple line rasterization)
    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }

        let color = image::Rgba([
            (stroke.color.r * 255.0) as u8,
            (stroke.color.g * 255.0) as u8,
            (stroke.color.b * 255.0) as u8,
            (stroke.color.a * 255.0) as u8,
        ]);

        for window in stroke.points.windows(2) {
            draw_line_segment(
                &mut imgbuf,
                (window[0].x * scale, window[0].y * scale),
                (window[1].x * scale, window[1].y * scale),
                color,
                (stroke.base_width as f64 * scale).max(1.0) as u32,
            );
        }
    }

    imgbuf.save(output_path)
        .map_err(|e| ImageExportError::Image(e.to_string()))?;

    log::info!("PNG exported to: {} ({}x{})", output_path, w, h);
    Ok(output_path.to_string())
}

/// Convert stroke points to an SVG path data string
fn points_to_svg_path(points: &[StrokePoint]) -> String {
    let mut path = String::new();
    if points.is_empty() {
        return path;
    }

    path.push_str(&format!("M {:.1} {:.1}", points[0].x, points[0].y));

    if points.len() <= 2 {
        for p in &points[1..] {
            path.push_str(&format!(" L {:.1} {:.1}", p.x, p.y));
        }
        return path;
    }

    // Use quadratic Bézier approximation for smoothness
    for i in 1..points.len() - 1 {
        let mid_x = (points[i].x + points[i + 1].x) / 2.0;
        let mid_y = (points[i].y + points[i + 1].y) / 2.0;
        path.push_str(&format!(
            " Q {:.1} {:.1} {:.1} {:.1}",
            points[i].x, points[i].y, mid_x, mid_y
        ));
    }

    let last = points.last().unwrap();
    path.push_str(&format!(" L {:.1} {:.1}", last.x, last.y));

    path
}

/// Simple line rasterization with thickness (Bresenham-based)
fn draw_line_segment(
    img: &mut image::RgbaImage,
    from: (f64, f64),
    to: (f64, f64),
    color: image::Rgba<u8>,
    thickness: u32,
) {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let dist = (dx * dx + dy * dy).sqrt();
    let steps = (dist * 2.0).max(1.0) as u32;
    let half_t = thickness as i32 / 2;

    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let x = (from.0 + dx * t) as i32;
        let y = (from.1 + dy * t) as i32;

        for ox in -half_t..=half_t {
            for oy in -half_t..=half_t {
                let px = x + ox;
                let py = y + oy;
                if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32 {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::ToolType;
    use uuid::Uuid;

    #[test]
    fn test_svg_path_generation() {
        let points = vec![
            StrokePoint { x: 10.0, y: 20.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 },
            StrokePoint { x: 50.0, y: 30.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 },
            StrokePoint { x: 90.0, y: 10.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 2000 },
        ];
        let path = points_to_svg_path(&points);
        assert!(path.starts_with("M 10.0 20.0"));
        assert!(path.contains("Q"));
    }

    #[test]
    fn test_svg_export() {
        let mut stroke = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
        stroke.add_point(StrokePoint { x: 10.0, y: 10.0, pressure: 0.5, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 0 });
        stroke.add_point(StrokePoint { x: 50.0, y: 50.0, pressure: 0.8, tilt_x: 0.0, tilt_y: 0.0, timestamp_us: 1000 });

        let svg = export_strokes_to_svg(&[stroke], 100.0, 100.0, Color::WHITE, "/tmp/snotes_test.svg");
        assert!(svg.is_ok());

        // Clean up
        let _ = std::fs::remove_file("/tmp/snotes_test.svg");
    }
}
