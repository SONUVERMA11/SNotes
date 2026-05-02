//! # Ink Rendering Engine
//!
//! Bézier cubic spline stroke engine with:
//! - Variable stroke width based on pressure + velocity
//! - Predictive ink (2-frame lookahead)
//! - Multiple tool types (pen, brush, pencil, marker, highlighter, eraser)
//! - GPU (Skia) and CPU (Cairo) rendering backends

mod stroke;
mod bezier;
mod stroke_builder;
mod renderer;
mod geometry;
mod eraser;

pub use stroke::*;
pub use bezier::*;
pub use stroke_builder::*;
pub use renderer::*;
pub use geometry::*;
pub use eraser::*;
