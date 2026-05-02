//! Text tool — add, edit, and render text annotations on the canvas

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A text annotation on a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnnotation {
    pub id: Uuid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: super::Color,
    pub alignment: TextAlignment,
    pub line_spacing: f32,
    pub layer_id: Uuid,
    pub rotation: f64,
    pub opacity: f32,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
}

impl FontWeight {
    pub fn to_pango_weight(&self) -> i32 {
        match self {
            FontWeight::Light => 300,
            FontWeight::Regular => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

impl TextAnnotation {
    pub fn new(x: f64, y: f64, layer_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            x,
            y,
            width: 200.0,
            height: 40.0,
            text: String::new(),
            font_family: "Sans".to_string(),
            font_size: 16.0,
            font_weight: FontWeight::Regular,
            font_style: FontStyle::Normal,
            color: super::Color::BLACK,
            alignment: TextAlignment::Left,
            line_spacing: 1.4,
            layer_id,
            rotation: 0.0,
            opacity: 1.0,
            locked: false,
        }
    }

    /// Set text content
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.auto_resize();
    }

    /// Auto-resize height based on content
    fn auto_resize(&mut self) {
        let line_count = self.text.lines().count().max(1);
        let line_height = self.font_size as f64 * self.line_spacing as f64;
        self.height = line_count as f64 * line_height + 8.0; // 8px padding
    }

    /// Hit test — is a point inside this text box?
    pub fn hit_test(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width
            && py >= self.y && py <= self.y + self.height
    }

    /// Get bounding box (x, y, w, h)
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.width, self.height)
    }

    /// Move to a new position
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    /// Resize the text box
    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width.max(50.0);
        self.height = height.max(20.0);
    }

    /// Get the font description string (for Pango/CSS)
    pub fn font_description(&self) -> String {
        format!(
            "{} {} {} {}px",
            self.font_family,
            match self.font_weight {
                FontWeight::Light => "Light",
                FontWeight::Regular => "Regular",
                FontWeight::Medium => "Medium",
                FontWeight::SemiBold => "SemiBold",
                FontWeight::Bold => "Bold",
                FontWeight::ExtraBold => "ExtraBold",
            },
            match self.font_style {
                FontStyle::Normal => "",
                FontStyle::Italic => "Italic",
                FontStyle::Oblique => "Oblique",
            },
            self.font_size,
        )
    }

    /// Export to SVG <text> element
    pub fn to_svg(&self) -> String {
        let anchor = match self.alignment {
            TextAlignment::Left => "start",
            TextAlignment::Center => "middle",
            TextAlignment::Right => "end",
            TextAlignment::Justify => "start",
        };
        format!(
            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" font-weight="{}" fill="rgba({},{},{},{})" text-anchor="{}" opacity="{}">{}</text>"#,
            self.x, self.y + self.font_size as f64,
            self.font_family, self.font_size,
            self.font_weight.to_pango_weight(),
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            self.color.a,
            anchor,
            self.opacity,
            html_escape(&self.text),
        )
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Text editing state
#[derive(Debug, Clone)]
pub struct TextEditor {
    pub annotation_id: Option<Uuid>,
    pub cursor_pos: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub composing: bool,
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            annotation_id: None,
            cursor_pos: 0,
            selection_start: None,
            selection_end: None,
            composing: false,
        }
    }

    pub fn start_editing(&mut self, annotation_id: Uuid) {
        self.annotation_id = Some(annotation_id);
        self.cursor_pos = 0;
        self.selection_start = None;
        self.selection_end = None;
    }

    pub fn stop_editing(&mut self) {
        self.annotation_id = None;
    }

    pub fn is_editing(&self) -> bool {
        self.annotation_id.is_some()
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_end.is_some()
    }

    /// Insert text at cursor
    pub fn insert(&mut self, text: &mut String, input: &str) {
        if self.has_selection() {
            self.delete_selection(text);
        }
        let pos = self.cursor_pos.min(text.len());
        text.insert_str(pos, input);
        self.cursor_pos = pos + input.len();
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self, text: &mut String) {
        if self.has_selection() {
            self.delete_selection(text);
            return;
        }
        if self.cursor_pos > 0 && self.cursor_pos <= text.len() {
            text.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    /// Delete selected text
    fn delete_selection(&mut self, text: &mut String) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (s, e) = (start.min(end), start.max(end));
            let s = s.min(text.len());
            let e = e.min(text.len());
            text.drain(s..e);
            self.cursor_pos = s;
            self.selection_start = None;
            self.selection_end = None;
        }
    }

    /// Select all text
    pub fn select_all(&mut self, text_len: usize) {
        self.selection_start = Some(0);
        self.selection_end = Some(text_len);
    }
}

impl Default for TextEditor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_annotation() {
        let mut ann = TextAnnotation::new(100.0, 200.0, Uuid::new_v4());
        ann.set_text("Hello, World!");
        assert!(ann.hit_test(150.0, 210.0));
        assert!(!ann.hit_test(50.0, 50.0));
    }

    #[test]
    fn test_text_editor() {
        let mut editor = TextEditor::new();
        let mut text = String::new();

        editor.start_editing(Uuid::new_v4());
        editor.insert(&mut text, "Hello");
        assert_eq!(text, "Hello");
        assert_eq!(editor.cursor_pos, 5);

        editor.insert(&mut text, " World");
        assert_eq!(text, "Hello World");

        editor.backspace(&mut text);
        assert_eq!(text, "Hello Worl");
    }

    #[test]
    fn test_svg_export() {
        let mut ann = TextAnnotation::new(10.0, 20.0, Uuid::new_v4());
        ann.set_text("Test <>&");
        let svg = ann.to_svg();
        assert!(svg.contains("&lt;&gt;&amp;"));
        assert!(svg.contains("font-family"));
    }

    #[test]
    fn test_font_description() {
        let ann = TextAnnotation::new(0.0, 0.0, Uuid::new_v4());
        let desc = ann.font_description();
        assert!(desc.contains("Sans"));
        assert!(desc.contains("16px"));
    }
}
