//! Page template rendering — generates background patterns for different page types

use crate::document::PageTemplate;
use crate::ink::Color;

/// Line definition for template rendering
#[derive(Debug, Clone)]
pub struct TemplateLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: Color,
    pub width: f32,
}

/// Dot definition for dotted templates
#[derive(Debug, Clone)]
pub struct TemplateDot {
    pub x: f64,
    pub y: f64,
    pub radius: f32,
    pub color: Color,
}

/// Rendered template elements
#[derive(Debug, Clone)]
pub struct TemplateElements {
    pub lines: Vec<TemplateLine>,
    pub dots: Vec<TemplateDot>,
}

const TEMPLATE_COLOR: Color = Color { r: 0.75, g: 0.85, b: 0.95, a: 0.6 };
const MARGIN_COLOR: Color = Color { r: 0.9, g: 0.3, b: 0.3, a: 0.4 };

/// Generate template elements for a page
pub fn render_template(template: PageTemplate, width: f64, height: f64, spacing: f64) -> TemplateElements {
    match template {
        PageTemplate::Blank => TemplateElements { lines: Vec::new(), dots: Vec::new() },
        PageTemplate::Lined => render_lined(width, height, spacing),
        PageTemplate::Grid => render_grid(width, height, spacing),
        PageTemplate::Dotted => render_dotted(width, height, spacing),
        PageTemplate::Isometric => render_isometric(width, height, spacing),
        PageTemplate::MusicStaff => render_music_staff(width, height),
        PageTemplate::Cornell => render_cornell(width, height, spacing),
    }
}

fn render_lined(width: f64, height: f64, spacing: f64) -> TemplateElements {
    let mut lines = Vec::new();
    let margin = 80.0;
    let top_margin = 100.0;

    // Horizontal lines
    let mut y = top_margin;
    while y < height - 40.0 {
        lines.push(TemplateLine {
            x1: margin, y1: y, x2: width - 40.0, y2: y,
            color: TEMPLATE_COLOR, width: 0.5,
        });
        y += spacing;
    }

    // Left margin line
    lines.push(TemplateLine {
        x1: margin, y1: 40.0, x2: margin, y2: height - 40.0,
        color: MARGIN_COLOR, width: 1.0,
    });

    TemplateElements { lines, dots: Vec::new() }
}

fn render_grid(width: f64, height: f64, spacing: f64) -> TemplateElements {
    let mut lines = Vec::new();
    let margin = 40.0;

    // Horizontal lines
    let mut y = margin;
    while y < height - margin {
        lines.push(TemplateLine {
            x1: margin, y1: y, x2: width - margin, y2: y,
            color: TEMPLATE_COLOR, width: 0.4,
        });
        y += spacing;
    }

    // Vertical lines
    let mut x = margin;
    while x < width - margin {
        lines.push(TemplateLine {
            x1: x, y1: margin, x2: x, y2: height - margin,
            color: TEMPLATE_COLOR, width: 0.4,
        });
        x += spacing;
    }

    TemplateElements { lines, dots: Vec::new() }
}

fn render_dotted(width: f64, height: f64, spacing: f64) -> TemplateElements {
    let mut dots = Vec::new();
    let margin = 40.0;
    let dot_color = Color { r: 0.6, g: 0.7, b: 0.8, a: 0.5 };

    let mut y = margin;
    while y < height - margin {
        let mut x = margin;
        while x < width - margin {
            dots.push(TemplateDot { x, y, radius: 1.0, color: dot_color });
            x += spacing;
        }
        y += spacing;
    }

    TemplateElements { lines: Vec::new(), dots }
}

fn render_isometric(width: f64, height: f64, spacing: f64) -> TemplateElements {
    let mut lines = Vec::new();
    let color = Color { r: 0.75, g: 0.85, b: 0.95, a: 0.4 };
    let angle = std::f64::consts::FRAC_PI_6; // 30 degrees

    // Horizontal lines
    let mut y = 0.0;
    while y < height {
        lines.push(TemplateLine { x1: 0.0, y1: y, x2: width, y2: y, color, width: 0.3 });
        y += spacing;
    }

    // Lines going right-down (30°)
    let dx = spacing / angle.tan();
    let mut start_x = -height;
    while start_x < width + height {
        lines.push(TemplateLine {
            x1: start_x, y1: 0.0,
            x2: start_x + height / angle.tan(), y2: height,
            color, width: 0.3,
        });
        start_x += dx;
    }

    // Lines going left-down (-30°)
    start_x = -height;
    while start_x < width + height {
        lines.push(TemplateLine {
            x1: start_x + height / angle.tan(), y1: 0.0,
            x2: start_x, y2: height,
            color, width: 0.3,
        });
        start_x += dx;
    }

    TemplateElements { lines, dots: Vec::new() }
}

fn render_music_staff(width: f64, height: f64) -> TemplateElements {
    let mut lines = Vec::new();
    let staff_spacing = 12.0;
    let staff_gap = 80.0;
    let margin_x = 60.0;
    let margin_top = 80.0;
    let color = Color { r: 0.3, g: 0.3, b: 0.3, a: 0.6 };

    let mut y = margin_top;
    while y + 4.0 * staff_spacing < height - 40.0 {
        // 5 lines per staff
        for line in 0..5 {
            let ly = y + line as f64 * staff_spacing;
            lines.push(TemplateLine {
                x1: margin_x, y1: ly, x2: width - margin_x, y2: ly,
                color, width: 0.6,
            });
        }
        y += 4.0 * staff_spacing + staff_gap;
    }

    TemplateElements { lines, dots: Vec::new() }
}

fn render_cornell(width: f64, height: f64, spacing: f64) -> TemplateElements {
    let mut lines = Vec::new();
    let cue_width = width * 0.3;
    let summary_height = height * 0.15;

    // Vertical divider (cue column)
    lines.push(TemplateLine {
        x1: cue_width, y1: 40.0, x2: cue_width, y2: height - summary_height,
        color: MARGIN_COLOR, width: 1.5,
    });

    // Horizontal divider (summary area)
    lines.push(TemplateLine {
        x1: 40.0, y1: height - summary_height, x2: width - 40.0, y2: height - summary_height,
        color: MARGIN_COLOR, width: 1.5,
    });

    // Ruled lines in the note-taking area
    let mut y = 100.0;
    while y < height - summary_height - 10.0 {
        lines.push(TemplateLine {
            x1: cue_width + 10.0, y1: y, x2: width - 40.0, y2: y,
            color: TEMPLATE_COLOR, width: 0.4,
        });
        y += spacing;
    }

    TemplateElements { lines, dots: Vec::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blank_template() {
        let elems = render_template(PageTemplate::Blank, 1000.0, 1400.0, 30.0);
        assert!(elems.lines.is_empty());
        assert!(elems.dots.is_empty());
    }

    #[test]
    fn test_lined_template() {
        let elems = render_template(PageTemplate::Lined, 1000.0, 1400.0, 30.0);
        assert!(!elems.lines.is_empty());
    }

    #[test]
    fn test_grid_template() {
        let elems = render_template(PageTemplate::Grid, 1000.0, 1400.0, 30.0);
        assert!(elems.lines.len() > 10);
    }

    #[test]
    fn test_dotted_template() {
        let elems = render_template(PageTemplate::Dotted, 1000.0, 1400.0, 30.0);
        assert!(!elems.dots.is_empty());
    }

    #[test]
    fn test_music_staff() {
        let elems = render_template(PageTemplate::MusicStaff, 1000.0, 1400.0, 30.0);
        assert!(elems.lines.len() >= 5);
    }

    #[test]
    fn test_cornell() {
        let elems = render_template(PageTemplate::Cornell, 1000.0, 1400.0, 30.0);
        assert!(elems.lines.len() >= 2);
    }
}
