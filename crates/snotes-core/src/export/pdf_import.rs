//! PDF import — load PDFs as page backgrounds for annotation

use crate::document::{Page, PageTemplate};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum PdfImportError {
    #[error("Failed to open PDF: {0}")]
    OpenFailed(String),
    #[error("Invalid PDF: {0}")]
    InvalidPdf(String),
    #[error("Page {0} not found in PDF (total: {1})")]
    PageNotFound(u32, u32),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Metadata extracted from a PDF document
#[derive(Debug, Clone)]
pub struct PdfInfo {
    pub path: String,
    pub page_count: u32,
    pub title: Option<String>,
    pub author: Option<String>,
    pub pages: Vec<PdfPageInfo>,
}

/// Per-page info from a PDF
#[derive(Debug, Clone)]
pub struct PdfPageInfo {
    pub index: u32,
    pub width_pt: f64,
    pub height_pt: f64,
}

impl PdfPageInfo {
    /// Convert PDF points to pixels at a given DPI
    pub fn width_px(&self, dpi: f64) -> f64 {
        self.width_pt * dpi / 72.0
    }

    pub fn height_px(&self, dpi: f64) -> f64 {
        self.height_pt * dpi / 72.0
    }
}

/// Import a PDF file and extract metadata
pub fn import_pdf(path: &str) -> Result<PdfInfo, PdfImportError> {
    let data = std::fs::read(path)
        .map_err(|e| PdfImportError::OpenFailed(e.to_string()))?;

    let doc = lopdf::Document::load_mem(&data)
        .map_err(|e| PdfImportError::InvalidPdf(e.to_string()))?;

    let page_count = doc.get_pages().len() as u32;

    // Extract document metadata
    let title = extract_info_string(&doc, b"Title");
    let author = extract_info_string(&doc, b"Author");

    // Extract per-page dimensions
    let mut pages = Vec::new();
    for (page_num, _page_id) in doc.get_pages() {
        let (width, height) = get_page_dimensions(&doc, page_num).unwrap_or((612.0, 792.0)); // Letter default
        pages.push(PdfPageInfo {
            index: page_num,
            width_pt: width,
            height_pt: height,
        });
    }

    pages.sort_by_key(|p| p.index);

    Ok(PdfInfo {
        path: path.to_string(),
        page_count,
        title,
        author,
        pages,
    })
}

/// Create S Notes pages from a PDF (one page per PDF page)
pub fn pdf_to_pages(pdf_info: &PdfInfo, dpi: f64) -> Vec<Page> {
    let mut pages = Vec::new();

    for pdf_page in &pdf_info.pages {
        let mut page = Page::new_with_size(
            PageTemplate::Blank,
            pdf_page.width_px(dpi),
            pdf_page.height_px(dpi),
        );
        page.background_pdf_page = Some(pdf_page.index);
        page.background_image = Some(pdf_info.path.clone());
        pages.push(page);
    }

    pages
}

fn extract_info_string(doc: &lopdf::Document, key: &[u8]) -> Option<String> {
    let trailer = &doc.trailer;
    let info_ref = trailer.get(b"Info").ok()?;
    let info_id = info_ref.as_reference().ok()?;
    let info_dict = doc.get_dictionary(info_id).ok()?;
    let value = info_dict.get(key).ok()?;
    match value {
        lopdf::Object::String(bytes, _) => String::from_utf8(bytes.clone()).ok(),
        _ => None,
    }
}

fn get_page_dimensions(doc: &lopdf::Document, page_num: u32) -> Option<(f64, f64)> {
    let pages = doc.get_pages();
    let page_id = pages.get(&page_num)?;
    let page_dict = doc.get_dictionary(*page_id).ok()?;

    // Try MediaBox first, then CropBox
    let media_box = page_dict
        .get(b"MediaBox")
        .or_else(|_| page_dict.get(b"CropBox"))
        .ok()?;

    if let lopdf::Object::Array(arr) = media_box {
        if arr.len() >= 4 {
            let x1 = obj_to_f64(&arr[0]).unwrap_or(0.0);
            let y1 = obj_to_f64(&arr[1]).unwrap_or(0.0);
            let x2 = obj_to_f64(&arr[2]).unwrap_or(612.0);
            let y2 = obj_to_f64(&arr[3]).unwrap_or(792.0);
            return Some(((x2 - x1).abs(), (y2 - y1).abs()));
        }
    }

    None
}

fn obj_to_f64(obj: &lopdf::Object) -> Option<f64> {
    match obj {
        lopdf::Object::Real(f) => Some(*f as f64),
        lopdf::Object::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

/// Render a PDF page to a PNG image (for display as background)
pub fn render_pdf_page_to_png(
    _path: &str,
    _page_num: u32,
    _dpi: f64,
    output_path: &str,
) -> Result<(), PdfImportError> {
    // In production: use poppler-rs or mupdf to rasterize the PDF page
    // For now, this is a stub that creates a placeholder
    log::info!("Rendering PDF page to: {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_page_dimensions() {
        let info = PdfPageInfo { index: 1, width_pt: 612.0, height_pt: 792.0 };
        // At 144 DPI: 612 * 144/72 = 1224
        assert!((info.width_px(144.0) - 1224.0).abs() < 0.01);
        assert!((info.height_px(144.0) - 1584.0).abs() < 0.01);
    }

    #[test]
    fn test_pdf_to_pages() {
        let pdf_info = PdfInfo {
            path: "test.pdf".to_string(),
            page_count: 3,
            title: Some("Test PDF".to_string()),
            author: None,
            pages: vec![
                PdfPageInfo { index: 1, width_pt: 612.0, height_pt: 792.0 },
                PdfPageInfo { index: 2, width_pt: 612.0, height_pt: 792.0 },
                PdfPageInfo { index: 3, width_pt: 842.0, height_pt: 595.0 }, // A4 landscape
            ],
        };
        let pages = pdf_to_pages(&pdf_info, 144.0);
        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].background_pdf_page, Some(1));
        assert!(pages[2].width > pages[2].height); // landscape
    }
}
