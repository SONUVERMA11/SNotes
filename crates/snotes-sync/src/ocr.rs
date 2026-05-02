//! OCR pipeline — handwriting recognition using Tesseract 5
//!
//! Converts handwritten strokes to searchable text.
//! Requires the `tesseract-dev` system library.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Tesseract not available: {0}")]
    TesseractNotAvailable(String),
    #[error("Recognition failed: {0}")]
    RecognitionFailed(String),
    #[error("Image conversion error: {0}")]
    ImageError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// OCR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    /// Language model (e.g. "eng", "deu", "fra")
    pub language: String,
    /// Page segmentation mode
    pub psm: PageSegMode,
    /// Minimum confidence threshold (0.0–1.0)
    pub min_confidence: f32,
    /// DPI for rasterization before OCR
    pub dpi: u32,
    /// Enable handwriting-specific model
    pub handwriting_mode: bool,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            psm: PageSegMode::SingleBlock,
            min_confidence: 0.3,
            dpi: 300,
            handwriting_mode: true,
        }
    }
}

/// Tesseract page segmentation modes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PageSegMode {
    /// Fully automatic page segmentation
    Auto,
    /// Single column of text
    SingleColumn,
    /// Single block of text
    SingleBlock,
    /// Single line
    SingleLine,
    /// Single word
    SingleWord,
    /// Single character
    SingleChar,
    /// Sparse text with OSD
    SparseText,
}

impl PageSegMode {
    pub fn to_tesseract_value(&self) -> i32 {
        match self {
            PageSegMode::Auto => 3,
            PageSegMode::SingleColumn => 4,
            PageSegMode::SingleBlock => 6,
            PageSegMode::SingleLine => 7,
            PageSegMode::SingleWord => 8,
            PageSegMode::SingleChar => 10,
            PageSegMode::SparseText => 11,
        }
    }
}

/// OCR result for a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// Full recognized text
    pub text: String,
    /// Per-word results with bounding boxes
    pub words: Vec<OcrWord>,
    /// Overall confidence (0.0–1.0)
    pub confidence: f32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// A recognized word with position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub confidence: f32,
    /// Bounding box: (x, y, width, height) in page coordinates
    pub bbox: (f64, f64, f64, f64),
}

/// OCR engine interface
pub struct OcrEngine {
    config: OcrConfig,
    available: bool,
}

impl OcrEngine {
    /// Create a new OCR engine (checks if Tesseract is available)
    pub fn new(config: OcrConfig) -> Self {
        // Check if tesseract is available on the system
        let available = check_tesseract_available();
        if !available {
            log::warn!("Tesseract OCR not found. OCR features disabled.");
        }
        Self { config, available }
    }

    /// Check if OCR is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Recognize text from a rasterized page image
    pub fn recognize_from_image(&self, image_path: &str) -> Result<OcrResult, OcrError> {
        if !self.available {
            return Err(OcrError::TesseractNotAvailable(
                "Tesseract is not installed. Install with: sudo apt install tesseract-ocr".to_string()
            ));
        }

        let start = std::time::Instant::now();

        // Shell out to tesseract CLI for simplicity
        // In production: use tesseract-rs bindings for direct API access
        let output = std::process::Command::new("tesseract")
            .arg(image_path)
            .arg("stdout")
            .arg("-l")
            .arg(&self.config.language)
            .arg("--psm")
            .arg(self.config.psm.to_tesseract_value().to_string())
            .arg("--dpi")
            .arg(self.config.dpi.to_string())
            .output()
            .map_err(|e| OcrError::TesseractNotAvailable(e.to_string()))?;

        if !output.status.success() {
            return Err(OcrError::RecognitionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        let elapsed = start.elapsed().as_millis() as u64;

        // For word-level results, we'd use tesseract's TSV/HOCR output
        // This is simplified for the initial implementation
        let words: Vec<OcrWord> = text
            .split_whitespace()
            .map(|w| OcrWord {
                text: w.to_string(),
                confidence: 0.8, // placeholder — real impl parses TSV
                bbox: (0.0, 0.0, 0.0, 0.0),
            })
            .collect();

        Ok(OcrResult {
            text: text.trim().to_string(),
            confidence: 0.8,
            words,
            processing_time_ms: elapsed,
        })
    }

    /// Recognize text from strokes by first rasterizing them
    pub fn recognize_from_strokes(
        &self,
        strokes: &[snotes_core::ink::Stroke],
        page_width: f64,
        page_height: f64,
    ) -> Result<OcrResult, OcrError> {
        if !self.available {
            return Err(OcrError::TesseractNotAvailable(
                "Tesseract is not installed".to_string()
            ));
        }

        // Rasterize strokes to a temporary PNG
        let tmp_path = format!("/tmp/snotes_ocr_{}.png", uuid::Uuid::new_v4());

        snotes_core::export::image_export::export_strokes_to_png(
            strokes,
            page_width as u32,
            page_height as u32,
            snotes_core::ink::Color::WHITE,
            self.config.dpi as f64 / 144.0, // scale relative to canvas DPI
            &tmp_path,
        ).map_err(|e| OcrError::ImageError(e.to_string()))?;

        let result = self.recognize_from_image(&tmp_path);

        // Clean up temp file
        let _ = std::fs::remove_file(&tmp_path);

        result
    }

    /// Update configuration
    pub fn set_config(&mut self, config: OcrConfig) {
        self.config = config;
    }
}

/// Check if tesseract CLI is available
fn check_tesseract_available() -> bool {
    std::process::Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get list of available tesseract languages
pub fn list_tesseract_languages() -> Vec<String> {
    let output = std::process::Command::new("tesseract")
        .arg("--list-langs")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .skip(1) // First line is header
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_config_default() {
        let config = OcrConfig::default();
        assert_eq!(config.language, "eng");
        assert!(config.handwriting_mode);
        assert_eq!(config.dpi, 300);
    }

    #[test]
    fn test_psm_values() {
        assert_eq!(PageSegMode::Auto.to_tesseract_value(), 3);
        assert_eq!(PageSegMode::SingleLine.to_tesseract_value(), 7);
        assert_eq!(PageSegMode::SingleWord.to_tesseract_value(), 8);
    }

    #[test]
    fn test_ocr_engine_creation() {
        let engine = OcrEngine::new(OcrConfig::default());
        // May or may not be available — just test creation doesn't panic
        let _ = engine.is_available();
    }
}
