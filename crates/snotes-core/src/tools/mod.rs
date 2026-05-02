//! Tool implementations

pub mod selection;
pub mod rulers;

use crate::ink::{Color, ToolType, EraserMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSettings {
    pub tool_type: ToolType,
    pub color: Color,
    pub width: f32,
    pub opacity: f32,
    pub eraser_mode: EraserMode,
    pub pressure_sensitive: bool,
    pub tilt_sensitive: bool,
    pub velocity_sensitive: bool,
}

impl Default for ToolSettings {
    fn default() -> Self {
        Self {
            tool_type: ToolType::Pen, color: Color::BLACK, width: 2.0,
            opacity: 1.0, eraser_mode: EraserMode::WholeStroke,
            pressure_sensitive: true, tilt_sensitive: false, velocity_sensitive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub settings: ToolSettings,
    pub preset_name: Option<String>,
}

impl Tool {
    pub fn pen(color: Color, width: f32) -> Self {
        Self { settings: ToolSettings { tool_type: ToolType::Pen, color, width, pressure_sensitive: true, ..Default::default() }, preset_name: None }
    }
    pub fn highlighter(color: Color) -> Self {
        Self { settings: ToolSettings { tool_type: ToolType::Highlighter, color: Color { a: 0.4, ..color }, width: 12.0, opacity: 0.4, pressure_sensitive: false, ..Default::default() }, preset_name: None }
    }
    pub fn eraser(mode: EraserMode, width: f32) -> Self {
        Self { settings: ToolSettings { tool_type: ToolType::Eraser, color: Color::WHITE, width, eraser_mode: mode, ..Default::default() }, preset_name: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPalette {
    pub tools: Vec<Tool>,
    pub active_index: usize,
    pub recent_colors: Vec<Color>,
}

impl Default for ToolPalette {
    fn default() -> Self {
        Self {
            tools: vec![
                Tool::pen(Color::BLACK, 2.0),
                Tool::pen(Color::BLUE, 2.0),
                Tool::highlighter(Color::YELLOW_HIGHLIGHT),
                Tool::eraser(EraserMode::WholeStroke, 10.0),
            ],
            active_index: 0,
            recent_colors: vec![Color::BLACK, Color::BLUE, Color::RED],
        }
    }
}

impl ToolPalette {
    pub fn active_tool(&self) -> &Tool { &self.tools[self.active_index] }
    pub fn select_tool(&mut self, index: usize) {
        if index < self.tools.len() { self.active_index = index; }
    }
}
