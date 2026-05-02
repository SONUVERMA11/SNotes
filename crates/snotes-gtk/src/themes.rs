//! Theme system — Dark / Light / Sepia / Custom themes

use serde::{Deserialize, Serialize};

/// Available theme modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    Sepia,
    Custom,
}

/// Theme color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub surface: String,
    pub primary: String,
    pub secondary: String,
    pub text: String,
    pub text_secondary: String,
    pub border: String,
    pub canvas_background: String,
    pub toolbar_background: String,
    pub sidebar_background: String,
    pub accent: String,
    pub error: String,
    pub success: String,
}

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub mode: ThemeMode,
    pub colors: ThemeColors,
    pub canvas_opacity: f32,
    pub font_family: String,
    pub corner_radius: u32,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            mode: ThemeMode::Light,
            colors: ThemeColors {
                background: "#ffffff".to_string(),
                surface: "#f8f9fa".to_string(),
                primary: "#1a73e8".to_string(),
                secondary: "#5f6368".to_string(),
                text: "#202124".to_string(),
                text_secondary: "#5f6368".to_string(),
                border: "#dadce0".to_string(),
                canvas_background: "#ffffff".to_string(),
                toolbar_background: "#f1f3f4".to_string(),
                sidebar_background: "#f8f9fa".to_string(),
                accent: "#1a73e8".to_string(),
                error: "#d93025".to_string(),
                success: "#1e8e3e".to_string(),
            },
            canvas_opacity: 1.0,
            font_family: "Inter, sans-serif".to_string(),
            corner_radius: 8,
        }
    }

    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            mode: ThemeMode::Dark,
            colors: ThemeColors {
                background: "#1e1e2e".to_string(),
                surface: "#2d2d3f".to_string(),
                primary: "#89b4fa".to_string(),
                secondary: "#a6adc8".to_string(),
                text: "#cdd6f4".to_string(),
                text_secondary: "#a6adc8".to_string(),
                border: "#45475a".to_string(),
                canvas_background: "#1e1e2e".to_string(),
                toolbar_background: "#313244".to_string(),
                sidebar_background: "#181825".to_string(),
                accent: "#89b4fa".to_string(),
                error: "#f38ba8".to_string(),
                success: "#a6e3a1".to_string(),
            },
            canvas_opacity: 1.0,
            font_family: "Inter, sans-serif".to_string(),
            corner_radius: 8,
        }
    }

    pub fn sepia() -> Self {
        Self {
            name: "Sepia".to_string(),
            mode: ThemeMode::Sepia,
            colors: ThemeColors {
                background: "#f4ecd8".to_string(),
                surface: "#ece0c8".to_string(),
                primary: "#8b6914".to_string(),
                secondary: "#6b5c3e".to_string(),
                text: "#3b3020".to_string(),
                text_secondary: "#6b5c3e".to_string(),
                border: "#d4c8a8".to_string(),
                canvas_background: "#f9f3e3".to_string(),
                toolbar_background: "#ece0c8".to_string(),
                sidebar_background: "#f0e6d0".to_string(),
                accent: "#8b6914".to_string(),
                error: "#c23616".to_string(),
                success: "#6ab04c".to_string(),
            },
            canvas_opacity: 1.0,
            font_family: "Merriweather, serif".to_string(),
            corner_radius: 6,
        }
    }

    /// Generate professional CSS for the GTK4 application
    pub fn to_css(&self) -> String {
        format!(
            r#"
            @define-color bg_color {bg};
            @define-color surface_color {surface};
            @define-color primary_color {primary};
            @define-color accent_color {accent};
            @define-color text_color {text};
            @define-color border_color {border};

            window {{ background-color: @bg_color; color: @text_color; }}

            headerbar {{
                background-color: {surface};
                border-bottom: 1px solid alpha(@border_color, 0.5);
                min-height: 42px;
            }}
            headerbar button {{
                border-radius: 8px;
                min-width: 32px; min-height: 32px;
                margin: 2px;
                transition: all 150ms ease;
            }}
            headerbar button:hover {{ background-color: alpha(@primary_color, 0.12); }}

            .sidebar {{
                background-color: {sidebar};
                border-right: 1px solid alpha(@border_color, 0.3);
            }}
            .sidebar .heading {{
                font-weight: 700; font-size: 13px;
                letter-spacing: 0.3px; opacity: 0.7;
                text-transform: uppercase;
            }}
            .sidebar searchentry {{
                border-radius: 10px; padding: 6px 12px;
                margin-bottom: 8px;
                background-color: alpha(@surface_color, 0.6);
                border: 1px solid alpha(@border_color, 0.3);
            }}
            .sidebar searchentry:focus {{
                border-color: @primary_color;
            }}
            .navigation-sidebar row {{
                border-radius: 10px; margin: 2px 4px;
                transition: all 150ms ease;
            }}
            .navigation-sidebar row:hover {{
                background-color: alpha(@primary_color, 0.08);
            }}
            .navigation-sidebar row:selected {{
                background-color: alpha(@primary_color, 0.18);
            }}

            .toolbar {{
                background-color: alpha({toolbar}, 0.95);
                border-top: 1px solid alpha(@border_color, 0.4);
                padding: 5px 12px;
                min-height: 44px;
            }}
            .tool-button {{
                border-radius: 10px;
                min-width: 36px; min-height: 36px;
                padding: 4px; margin: 0 1px;
                transition: all 200ms ease;
                border: 2px solid transparent;
            }}
            .tool-button:hover {{
                background-color: alpha(@primary_color, 0.1);
            }}
            .tool-button:checked {{
                background-color: alpha(@primary_color, 0.2);
                color: @primary_color;
                border-color: @primary_color;
                font-weight: 700;
            }}

            .canvas-area {{ background-color: {canvas}; }}

            .toolbar button.flat {{
                border-radius: 8px;
                min-width: 32px; min-height: 32px;
                transition: all 150ms ease;
            }}
            .toolbar button.flat:hover {{
                background-color: alpha(@primary_color, 0.12);
            }}
            scale trough {{
                border-radius: 6px; min-height: 6px;
                background-color: alpha(@border_color, 0.3);
            }}
            scale highlight {{
                border-radius: 6px;
                background-color: @primary_color;
            }}
            scale slider {{
                background-color: @primary_color;
                border-radius: 50%;
                min-width: 16px; min-height: 16px;
                border: 2px solid alpha(white, 0.3);
            }}
            .toolbar separator {{
                background-color: alpha(@border_color, 0.3);
                min-width: 1px; margin: 6px 0;
            }}
            .toolbar label {{
                font-size: 12px; font-weight: 600;
                opacity: 0.8;
            }}
            "#,
            bg = self.colors.background,
            surface = self.colors.surface,
            primary = self.colors.primary,
            accent = self.colors.accent,
            text = self.colors.text,
            border = self.colors.border,
            toolbar = self.colors.toolbar_background,
            sidebar = self.colors.sidebar_background,
            canvas = self.colors.canvas_background,
        )
    }
}

impl Default for Theme {
    fn default() -> Self { Self::dark() }
}

/// Theme manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManager {
    pub active_theme: ThemeMode,
    pub custom_theme: Option<Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self { active_theme: ThemeMode::Dark, custom_theme: None }
    }

    pub fn get_active(&self) -> Theme {
        match self.active_theme {
            ThemeMode::Light => Theme::light(),
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::Sepia => Theme::sepia(),
            ThemeMode::Custom => self.custom_theme.clone().unwrap_or_else(Theme::dark),
        }
    }

    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.active_theme = mode;
    }

    pub fn set_custom(&mut self, theme: Theme) {
        self.custom_theme = Some(theme);
        self.active_theme = ThemeMode::Custom;
    }
}

impl Default for ThemeManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_css_generation() {
        let theme = Theme::dark();
        let css = theme.to_css();
        assert!(css.contains("background-color"));
        assert!(css.contains(&theme.colors.background));
    }

    #[test]
    fn test_theme_manager() {
        let mut tm = ThemeManager::new();
        assert_eq!(tm.active_theme, ThemeMode::Dark);
        tm.set_theme(ThemeMode::Light);
        let t = tm.get_active();
        assert_eq!(t.mode, ThemeMode::Light);
    }
}
