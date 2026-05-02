//! Plugin API — defines the interface available to WASM plugins
//!
//! This is the "guest-side" API contract. Plugins import these functions
//! from the host. The host provides implementations that are sandboxed
//! per-plugin based on granted capabilities.

use serde::{Deserialize, Serialize};

/// API surface exposed to plugins via WASM imports
///
/// Each function checks the plugin's granted capabilities before executing.
pub struct PluginApi;

/// Stroke data exposed to plugins (simplified, read-only view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStrokeData {
    pub id: String,
    pub tool: String,
    pub color_hex: String,
    pub point_count: u32,
    pub length: f64,
    pub bounds: (f64, f64, f64, f64),
}

/// Page info exposed to plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPageInfo {
    pub id: String,
    pub width: f64,
    pub height: f64,
    pub template: String,
    pub stroke_count: u32,
    pub layer_count: u32,
}

/// Toolbar button registered by a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginToolbarButton {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub tooltip: String,
    pub callback_name: String,
}

/// Custom export format registered by a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExportFormat {
    pub name: String,
    pub extension: String,
    pub description: String,
    pub callback_name: String,
}

/// Events that plugins can listen to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// A new stroke was added
    StrokeAdded { stroke_id: String },
    /// A stroke was removed
    StrokeRemoved { stroke_id: String },
    /// The active page changed
    PageChanged { page_id: String },
    /// The active tool changed
    ToolChanged { tool_name: String },
    /// A plugin toolbar button was clicked
    ButtonClicked { button_id: String },
    /// The document was saved
    DocumentSaved,
    /// The document was loaded
    DocumentLoaded,
}

impl PluginApi {
    // ─── Stroke API (requires ReadStrokes) ────────────────────

    /// Get the number of strokes on the current page
    pub fn stroke_count() -> u32 { 0 }

    /// Get simplified data about all strokes on the current page
    pub fn get_strokes() -> Vec<PluginStrokeData> { Vec::new() }

    /// Get a specific stroke by ID
    pub fn get_stroke(_id: &str) -> Option<PluginStrokeData> { None }

    // ─── Page API ─────────────────────────────────────────────

    /// Get info about the current page
    pub fn current_page() -> Option<PluginPageInfo> { None }

    /// Get info about all pages in the current notebook
    pub fn list_pages() -> Vec<PluginPageInfo> { Vec::new() }

    // ─── Tool API ─────────────────────────────────────────────

    /// Get the current tool type as a string
    pub fn current_tool() -> String { "pen".to_string() }

    /// Get the current stroke color as hex
    pub fn current_color() -> String { "#000000".to_string() }

    /// Get the current stroke width
    pub fn current_width() -> f32 { 2.0 }

    // ─── UI API (requires ToolbarExtension) ───────────────────

    /// Register a toolbar button
    pub fn register_toolbar_button(_button: PluginToolbarButton) -> bool { false }

    /// Show a notification toast
    pub fn notify(_message: &str, _duration_ms: u32) {}

    /// Show a dialog with a message
    pub fn show_dialog(_title: &str, _message: &str) {}

    // ─── Export API (requires ExportFormat) ────────────────────

    /// Register a custom export format
    pub fn register_export_format(_format: PluginExportFormat) -> bool { false }

    // ─── Settings API (requires Settings) ─────────────────────

    /// Get a plugin setting
    pub fn get_setting(_key: &str) -> Option<String> { None }

    /// Set a plugin setting
    pub fn set_setting(_key: &str, _value: &str) -> bool { false }

    // ─── Canvas API (requires CanvasOverlay) ──────────────────

    /// Draw a line on the canvas overlay
    pub fn draw_overlay_line(
        _x1: f64, _y1: f64, _x2: f64, _y2: f64,
        _color_hex: &str, _width: f32,
    ) {}

    /// Draw a circle on the canvas overlay
    pub fn draw_overlay_circle(
        _cx: f64, _cy: f64, _radius: f64,
        _color_hex: &str, _width: f32,
    ) {}

    /// Clear the canvas overlay
    pub fn clear_overlay() {}

    // ─── Logging ──────────────────────────────────────────────

    /// Log a message from a plugin (visible in app logs)
    pub fn log(_level: &str, _message: &str) {}
}

/// The WASM import/export contract for plugins
///
/// Plugins must export these functions:
/// ```text
/// extern "C" fn snotes_init() -> i32;          // Called on load, return 0 for success
/// extern "C" fn snotes_on_event(event: *const u8, len: u32);  // Event handler
/// extern "C" fn snotes_shutdown();              // Called on unload
/// ```
///
/// And can import these host functions from the "snotes" module:
/// ```text
/// "snotes"."stroke_count"  -> () -> i32
/// "snotes"."get_strokes"   -> (buf: *mut u8, max_len: i32) -> i32
/// "snotes"."notify"        -> (msg: *const u8, len: i32, duration_ms: i32) -> ()
/// "snotes"."log"           -> (level: i32, msg: *const u8, len: i32) -> ()
/// "snotes"."get_setting"   -> (key: *const u8, key_len: i32, val: *mut u8, max_len: i32) -> i32
/// "snotes"."set_setting"   -> (key: *const u8, key_len: i32, val: *const u8, val_len: i32) -> i32
/// ```
pub struct WasmContract;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_defaults() {
        assert_eq!(PluginApi::stroke_count(), 0);
        assert_eq!(PluginApi::current_tool(), "pen");
        assert_eq!(PluginApi::current_color(), "#000000");
        assert!(PluginApi::current_width() > 0.0);
    }

    #[test]
    fn test_plugin_event_serialization() {
        let event = PluginEvent::StrokeAdded { stroke_id: "abc-123".to_string() };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("abc-123"));

        let parsed: PluginEvent = serde_json::from_str(&json).unwrap();
        if let PluginEvent::StrokeAdded { stroke_id } = parsed {
            assert_eq!(stroke_id, "abc-123");
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn test_manifest_serialization() {
        use crate::host::{PluginManifest, PluginCapability};
        let manifest = PluginManifest {
            name: "my-plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Dev".to_string(),
            description: "A test plugin".to_string(),
            capabilities: vec![PluginCapability::ReadStrokes, PluginCapability::Settings],
            module: "plugin.wasm".to_string(),
            icon: None,
            min_app_version: Some("0.1.0".to_string()),
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("my-plugin"));
        assert!(json.contains("ReadStrokes"));
    }
}
