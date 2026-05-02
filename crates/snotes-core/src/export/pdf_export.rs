//! PDF export — render strokes to PDF with embedded vector paths

use crate::ink::{Stroke, BezierSpline, Color};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfExportError {
    #[error("Export failed: {0}")]
    Failed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Export options for PDF generation
pub struct PdfExportOptions {
    pub output_path: String,
    pub page_width_mm: f64,
    pub page_height_mm: f64,
    pub include_background: bool,
    pub title: String,
    pub author: String,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            output_path: "output.pdf".to_string(),
            page_width_mm: 210.0,  // A4
            page_height_mm: 297.0, // A4
            include_background: true,
            title: "S Notes Export".to_string(),
            author: "S Notes".to_string(),
        }
    }
}

/// Export strokes to a PDF file using printpdf
pub fn export_strokes_to_pdf(
    pages: &[PageExportData],
    options: &PdfExportOptions,
) -> Result<String, PdfExportError> {
    use printpdf::*;

    let (doc, first_page, first_layer) = PdfDocument::new(
        &options.title,
        Mm(options.page_width_mm as f32),
        Mm(options.page_height_mm as f32),
        "Layer 1",
    );

    // Set metadata
    // doc.set_author(&options.author); // not available in all versions

    // Render first page
    if let Some(first) = pages.first() {
        let layer = doc.get_page(first_page).get_layer(first_layer);
        render_strokes_to_layer(&layer, &first.strokes, &first.splines, options);
    }

    // Render additional pages
    for page_data in pages.iter().skip(1) {
        let (page_idx, layer_idx) = doc.add_page(
            Mm(options.page_width_mm as f32),
            Mm(options.page_height_mm as f32),
            "Layer 1",
        );
        let layer = doc.get_page(page_idx).get_layer(layer_idx);
        render_strokes_to_layer(&layer, &page_data.strokes, &page_data.splines, options);
    }

    // Save to file
    let file = std::fs::File::create(&options.output_path)
        .map_err(|e| PdfExportError::Io(e))?;
    let mut writer = std::io::BufWriter::new(file);
    doc.save(&mut writer)
        .map_err(|e| PdfExportError::Failed(e.to_string()))?;

    log::info!("PDF exported to: {}", options.output_path);
    Ok(options.output_path.clone())
}

/// Data for one page to export
pub struct PageExportData {
    pub strokes: Vec<Stroke>,
    pub splines: Vec<BezierSpline>,
    pub width: f64,
    pub height: f64,
}

fn render_strokes_to_layer(
    layer: &printpdf::PdfLayerReference,
    strokes: &[Stroke],
    splines: &[BezierSpline],
    options: &PdfExportOptions,
) {
    use printpdf::*;

    let scale_x = options.page_width_mm / 1191.0; // canvas width to mm
    let scale_y = options.page_height_mm / 1684.0; // canvas height to mm

    for (i, stroke) in strokes.iter().enumerate() {
        let color = printpdf::Color::Rgb(Rgb::new(
            stroke.color.r as f32,
            stroke.color.g as f32,
            stroke.color.b as f32,
            None,
        ));

        layer.set_outline_color(color.clone());
        layer.set_outline_thickness(stroke.base_width as f32 * scale_x as f32);

        if stroke.points.len() < 2 {
            continue;
        }

        // Build path from points
        let mut points = Vec::new();
        let mut first = true;

        for point in &stroke.points {
            let x = Mm((point.x * scale_x) as f32);
            // PDF y-axis is inverted (0 at bottom)
            let y = Mm((options.page_height_mm - point.y * scale_y) as f32);

            if first {
                points.push((Point::new(x, y), false));
                first = false;
            } else {
                points.push((Point::new(x, y), false));
            }
        }

        if points.len() >= 2 {
            let line = Line {
                points,
                is_closed: false,
            };

            layer.add_line(line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_export_options_default() {
        let opts = PdfExportOptions::default();
        assert!((opts.page_width_mm - 210.0).abs() < 0.01);
        assert!((opts.page_height_mm - 297.0).abs() < 0.01);
    }
}
