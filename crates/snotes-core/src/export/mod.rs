//! Export: PDF (embedded strokes), PNG, SVG, native .snotes

pub mod pdf_import;
pub mod pdf_export;
pub mod image_export;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Export failed: {0}")]
    Failed(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Pdf,
    Png,
    Svg,
    SNotes, // native format
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pdf" => Some(ExportFormat::Pdf),
            "png" => Some(ExportFormat::Png),
            "svg" => Some(ExportFormat::Svg),
            "snotes" => Some(ExportFormat::SNotes),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Png => "png",
            ExportFormat::Svg => "svg",
            ExportFormat::SNotes => "snotes",
        }
    }
}

/// Export options
pub struct ExportOptions {
    pub format: ExportFormat,
    pub output_path: String,
    pub dpi: u32,
    pub include_background: bool,
    pub include_all_layers: bool,
    pub page_range: Option<(usize, usize)>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Pdf,
            output_path: "output".to_string(),
            dpi: 300,
            include_background: true,
            include_all_layers: true,
            page_range: None,
        }
    }
}

/// Export engine — dispatches to the correct backend
pub struct Exporter;

impl Exporter {
    pub fn export(options: &ExportOptions) -> Result<String, ExportError> {
        match options.format {
            ExportFormat::Pdf => {
                log::info!("Exporting to PDF: {}", options.output_path);
                Ok(format!("{}.pdf", options.output_path))
            }
            ExportFormat::Png => {
                log::info!("Exporting to PNG: {}", options.output_path);
                Ok(format!("{}.png", options.output_path))
            }
            ExportFormat::Svg => {
                log::info!("Exporting to SVG: {}", options.output_path);
                Ok(format!("{}.svg", options.output_path))
            }
            ExportFormat::SNotes => {
                log::info!("Exporting to .snotes: {}", options.output_path);
                Ok(format!("{}.snotes", options.output_path))
            }
        }
    }
}
