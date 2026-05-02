//! # S Notes Core
//!
//! Core engine library for S Notes — a Linux-native handwriting & annotation app.
//!
//! This crate provides:
//! - **Input Engine**: Universal tablet/stylus support via libinput with pressure normalization
//! - **Ink Rendering**: Bézier cubic spline stroke engine with GPU (Skia) and CPU (Cairo) backends
//! - **Canvas System**: Infinite canvas with multi-layer support
//! - **Document Model**: Hierarchical notebook/section/page organization
//! - **Storage**: SQLite metadata + LZ4-compressed FlatBuffers for strokes
//! - **Export**: PDF, PNG, SVG output
//! - **Tools**: Pen, brush, pencil, marker, highlighter, eraser, shapes

pub mod input;
pub mod ink;
pub mod canvas;
pub mod document;
pub mod tools;
pub mod shapes;
pub mod history;
pub mod storage;
pub mod export;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::input::{InputEvent, InputDevice, StylusState, PenButton};
    pub use crate::ink::{Stroke, StrokePoint, StrokeBuilder, BezierSpline};
    pub use crate::canvas::{Canvas, Layer, Viewport};
    pub use crate::document::{Library, Notebook, Section, Page, PageTemplate};
    pub use crate::tools::{Tool, ToolSettings};
    pub use crate::ink::ToolType;
    pub use crate::history::{HistoryStack, HistoryAction};
    pub use uuid::Uuid;
}
