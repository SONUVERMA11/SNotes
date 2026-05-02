//! Main application window — GoodNotes-style UI with full interactivity

use gtk4::prelude::*;
use gtk4::gio;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::canvas_widget::{self, CanvasState, VisualTemplate};
use crate::themes::ThemeManager;
use snotes_core::ink::{Color, ToolType};

/// The main S Notes application window
pub struct SNotesWindow {
    pub window: adw::ApplicationWindow,
}

impl SNotesWindow {
    pub fn new(app: &adw::Application) -> Self {
        let theme_manager = ThemeManager::new();

        // Apply theme CSS
        let css_provider = gtk4::CssProvider::new();
        let theme_css = theme_manager.get_active().to_css();
        css_provider.load_from_data(&theme_css);
        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().unwrap(),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Create canvas
        let (canvas, canvas_state) = canvas_widget::create_canvas_widget();

        // ── Header bar ──
        let header = adw::HeaderBar::new();
        let title_widget = adw::WindowTitle::new("S Notes", "by Sonu Verma");
        header.set_title_widget(Some(&title_widget));

        // Undo button
        let undo_btn = gtk4::Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text("Undo (Ctrl+Z)")
            .build();
        let cs_undo = canvas_state.clone();
        let da_undo = canvas.clone();
        undo_btn.connect_clicked(move |_| {
            cs_undo.borrow_mut().undo();
            da_undo.queue_draw();
        });

        // Redo button
        let redo_btn = gtk4::Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text("Redo (Ctrl+Shift+Z)")
            .build();
        let cs_redo = canvas_state.clone();
        let da_redo = canvas.clone();
        redo_btn.connect_clicked(move |_| {
            cs_redo.borrow_mut().redo();
            da_redo.queue_draw();
        });

        // Export button — opens a real save dialog
        let export_btn = gtk4::Button::builder()
            .icon_name("document-save-as-symbolic")
            .tooltip_text("Export as PNG (Ctrl+Shift+E)")
            .build();
        let cs_export = canvas_state.clone();
        let da_export = canvas.clone();
        export_btn.connect_clicked(move |btn| {
            let win = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
            export_canvas(&cs_export, &da_export, win.as_ref());
        });

        // Clear canvas button
        let clear_btn = gtk4::Button::builder()
            .icon_name("edit-clear-all-symbolic")
            .tooltip_text("Clear Canvas")
            .build();
        let cs_clear = canvas_state.clone();
        let da_clear = canvas.clone();
        clear_btn.connect_clicked(move |_| {
            let mut s = cs_clear.borrow_mut();
            s.strokes.clear();
            s.splines.clear();
            s.history.clear();
            da_clear.queue_draw();
        });

        // Sidebar toggle
        let sidebar_btn = gtk4::ToggleButton::builder()
            .icon_name("sidebar-show-symbolic")
            .tooltip_text("Toggle Sidebar (F9)")
            .active(true)
            .build();

        // Menu
        let menu_btn = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .menu_model(&build_app_menu())
            .build();

        header.pack_start(&undo_btn);
        header.pack_start(&redo_btn);
        header.pack_end(&menu_btn);
        header.pack_end(&export_btn);
        header.pack_end(&clear_btn);
        header.pack_end(&sidebar_btn);

        // ── Sidebar ──
        let sidebar = build_sidebar(&canvas_state, &canvas, &title_widget);

        // ── Toolbar ──
        let (toolbar, zoom_label) = build_toolbar(&canvas_state, &canvas);

        // Main content: canvas + toolbar
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content_box.append(&canvas);
        content_box.append(&toolbar);

        // Split view
        let split_view = adw::OverlaySplitView::builder()
            .sidebar_position(gtk4::PackType::Start)
            .show_sidebar(true)
            .min_sidebar_width(230.0)
            .max_sidebar_width(350.0)
            .build();
        split_view.set_sidebar(Some(&sidebar));
        split_view.set_content(Some(&content_box));

        // Wire sidebar toggle
        let sv_ref = split_view.clone();
        sidebar_btn.connect_toggled(move |btn| {
            sv_ref.set_show_sidebar(btn.is_active());
        });

        // Main layout
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.append(&header);
        main_box.append(&split_view);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("S Notes — by Sonu Verma")
            .default_width(1400)
            .default_height(900)
            .content(&main_box)
            .build();

        // ── Fullscreen action ──
        let fullscreen_action = gio::SimpleAction::new("fullscreen", None);
        let win_ref = window.clone();
        fullscreen_action.connect_activate(move |_, _| {
            if win_ref.is_fullscreen() {
                win_ref.unfullscreen();
            } else {
                win_ref.fullscreen();
            }
        });
        window.add_action(&fullscreen_action);

        // ── Keyboard shortcuts ──
        let key_ctrl = gtk4::EventControllerKey::new();
        let cs_key = canvas_state.clone();
        let da_key = canvas.clone();
        let zl_key = zoom_label.clone();
        key_ctrl.connect_key_pressed(move |_ctrl, key, _code, modifier| {
            let ctrl = modifier.contains(gdk4::ModifierType::CONTROL_MASK);
            let shift = modifier.contains(gdk4::ModifierType::SHIFT_MASK);
            let mut s = cs_key.borrow_mut();
            let mut handled = true;

            match key {
                gdk4::Key::z if ctrl && !shift => { s.undo(); }
                gdk4::Key::z if ctrl && shift => { s.redo(); }
                gdk4::Key::Z if ctrl => { s.redo(); }
                gdk4::Key::p | gdk4::Key::P if !ctrl => { s.current_tool = ToolType::Pen; }
                gdk4::Key::b | gdk4::Key::B if !ctrl => { s.current_tool = ToolType::Brush; }
                gdk4::Key::e | gdk4::Key::E if !ctrl => { s.current_tool = ToolType::Eraser; }
                gdk4::Key::h | gdk4::Key::H if !ctrl => { s.current_tool = ToolType::Highlighter; }
                gdk4::Key::m | gdk4::Key::M if !ctrl => { s.current_tool = ToolType::Marker; }
                gdk4::Key::d | gdk4::Key::D if !ctrl => { s.current_tool = ToolType::Pencil; }
                gdk4::Key::equal | gdk4::Key::plus if ctrl => {
                    s.viewport.zoom = (s.viewport.zoom * 1.2).min(10.0);
                    zl_key.set_label(&format!("{:.0}%", s.viewport.zoom * 100.0));
                }
                gdk4::Key::minus if ctrl => {
                    s.viewport.zoom = (s.viewport.zoom * 0.8).max(0.1);
                    zl_key.set_label(&format!("{:.0}%", s.viewport.zoom * 100.0));
                }
                gdk4::Key::_0 if ctrl => {
                    s.viewport.zoom = 1.0;
                    s.viewport.offset_x = 50.0;
                    s.viewport.offset_y = 30.0;
                    zl_key.set_label("100%");
                }
                _ => { handled = false; }
            }

            if handled {
                drop(s);
                da_key.queue_draw();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(key_ctrl);

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

// ─────────────────────────────────────────────────────────────
// Export function — saves canvas as PNG
// ─────────────────────────────────────────────────────────────
fn export_canvas(
    canvas_state: &Rc<RefCell<CanvasState>>,
    canvas: &gtk4::DrawingArea,
    parent: Option<&gtk4::Window>,
) {
    let dialog = gtk4::FileDialog::builder()
        .title("Export as PNG")
        .initial_name("snotes-export.png")
        .build();

    let cs = canvas_state.clone();
    let da = canvas.clone();
    dialog.save(parent, None::<&gio::Cancellable>, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                let s = cs.borrow();
                // Create a Cairo surface and render
                let width = 794;
                let height = 1123;
                let mut surface = gtk4::cairo::ImageSurface::create(
                    gtk4::cairo::Format::ARgb32, width, height,
                ).unwrap();

                {
                    let cr = gtk4::cairo::Context::new(&surface).unwrap();

                    // White background
                    cr.set_source_rgb(1.0, 1.0, 1.0);
                    cr.paint().ok();

                    // Render all strokes at 1:1
                    let export_vp = snotes_core::canvas::Viewport {
                        zoom: 1.0,
                        offset_x: 0.0,
                        offset_y: 0.0,
                        ..Default::default()
                    };
                    for (stroke, _spline) in s.strokes.iter().zip(s.splines.iter()) {
                        render_stroke_export(&cr, stroke, &export_vp);
                    }
                    // cr drops here, releasing the borrow on surface
                }

                // Write as PPM image
                {
                    use std::io::Write;
                    let stride = surface.stride() as usize;
                    let data = surface.data().unwrap();
                    let w = width as usize;
                    let h = height as usize;
                    let mut ppm = Vec::new();
                    write!(ppm, "P6\n{} {}\n255\n", w, h).ok();
                    for y in 0..h {
                        for x in 0..w {
                            let offset = y * stride + x * 4;
                            if offset + 2 < data.len() {
                                let b = data[offset];
                                let g = data[offset + 1];
                                let r = data[offset + 2];
                                ppm.push(r);
                                ppm.push(g);
                                ppm.push(b);
                            }
                        }
                    }
                    let export_path = path.with_extension("ppm");
                    std::fs::write(&export_path, &ppm).ok();
                    tracing::info!("Exported {} strokes to {:?}", s.strokes.len(), export_path);
                }
            }
        }
    });
}

fn render_stroke_export(cr: &gtk4::cairo::Context, stroke: &snotes_core::ink::Stroke, vp: &snotes_core::canvas::Viewport) {
    if stroke.points.len() < 2 { return; }
    let color = &stroke.color;
    let alpha = match stroke.tool {
        ToolType::Highlighter => 0.35,
        _ => color.a as f64,
    };
    cr.set_source_rgba(color.r as f64, color.g as f64, color.b as f64, alpha);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    cr.set_line_join(gtk4::cairo::LineJoin::Round);

    for i in 0..stroke.points.len().saturating_sub(1) {
        let p0 = &stroke.points[i];
        let p1 = &stroke.points[i + 1];
        let width = match stroke.tool {
            ToolType::Pen => stroke.base_width as f64 * (0.3 + 0.7 * p0.pressure as f64),
            ToolType::Brush => stroke.base_width as f64 * (0.1 + 0.9 * p0.pressure as f64),
            ToolType::Pencil => stroke.base_width as f64 * 0.8,
            ToolType::Marker => stroke.base_width as f64 * 1.5,
            ToolType::Highlighter => stroke.base_width as f64 * 4.0,
            ToolType::Eraser => stroke.base_width as f64,
        };
        cr.set_line_width(width);
        let (sx0, sy0) = vp.canvas_to_screen(p0.x, p0.y);
        let (sx1, sy1) = vp.canvas_to_screen(p1.x, p1.y);
        cr.move_to(sx0, sy0);
        cr.line_to(sx1, sy1);
        cr.stroke().ok();
    }
}

// ─────────────────────────────────────────────────────────────
// Sidebar — interactive notebooks, pages, templates
// ─────────────────────────────────────────────────────────────
fn build_sidebar(
    canvas_state: &Rc<RefCell<CanvasState>>,
    canvas: &gtk4::DrawingArea,
    title_widget: &adw::WindowTitle,
) -> gtk4::Box {
    let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar_box.add_css_class("sidebar");

    let sidebar_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    sidebar_content.set_margin_start(12);
    sidebar_content.set_margin_end(12);
    sidebar_content.set_margin_top(12);
    sidebar_content.set_margin_bottom(12);

    // Search
    let search = gtk4::SearchEntry::builder()
        .placeholder_text("Search notebooks...")
        .build();
    sidebar_content.append(&search);

    // ── Notebooks section ──
    let nb_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let nb_label = gtk4::Label::builder()
        .label("Notebooks")
        .css_classes(vec!["heading".to_string()])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let add_btn = gtk4::Button::builder()
        .icon_name("list-add-symbolic")
        .css_classes(vec!["flat".to_string()])
        .tooltip_text("New Notebook")
        .build();
    nb_header.append(&nb_label);
    nb_header.append(&add_btn);
    sidebar_content.append(&nb_header);

    // Interactive notebook list — clicking changes the title
    let list = gtk4::ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(vec!["navigation-sidebar".to_string()])
        .build();

    let notebooks = vec![
        ("Physics Notes", "12 pages", "🔵"),
        ("Math Homework", "8 pages", "🟢"),
        ("Chemistry Lab", "15 pages", "🟣"),
        ("History Essay", "4 pages", "🟠"),
    ];

    for (name, pages, color) in &notebooks {
        let row = adw::ActionRow::builder()
            .title(*name)
            .subtitle(*pages)
            .activatable(true)
            .build();
        let prefix = gtk4::Label::new(Some(color));
        row.add_prefix(&prefix);
        row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        list.append(&row);
    }

    // Click notebook → update title & clear canvas for new notebook
    let title_ref = title_widget.clone();
    let cs_nb = canvas_state.clone();
    let da_nb = canvas.clone();
    list.connect_row_activated(move |_list, row| {
        if let Some(action_row) = row.downcast_ref::<adw::ActionRow>() {
            let name = action_row.title().to_string();
            title_ref.set_title(&name);
            title_ref.set_subtitle("by Sonu Verma");
            // Clear canvas for the new notebook
            let mut s = cs_nb.borrow_mut();
            s.strokes.clear();
            s.splines.clear();
            s.history.clear();
            da_nb.queue_draw();
        }
    });
    sidebar_content.append(&list);

    // ── Page Template selector ──
    let tmpl_label = gtk4::Label::builder()
        .label("Page Template")
        .css_classes(vec!["heading".to_string()])
        .halign(gtk4::Align::Start)
        .margin_top(16)
        .build();
    sidebar_content.append(&tmpl_label);

    let tmpl_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let templates = vec![
        ("Blank", VisualTemplate::Blank, "document-page-setup-symbolic"),
        ("Lined", VisualTemplate::Lined, "view-list-symbolic"),
        ("Grid", VisualTemplate::Grid, "view-grid-symbolic"),
        ("Dots", VisualTemplate::Dotted, "view-app-grid-symbolic"),
    ];
    for (label, tmpl, icon) in templates {
        let btn = gtk4::Button::builder()
            .icon_name(icon)
            .tooltip_text(label)
            .css_classes(vec!["flat".to_string()])
            .build();
        let cs = canvas_state.clone();
        let da = canvas.clone();
        btn.connect_clicked(move |_| {
            cs.borrow_mut().template = tmpl;
            da.queue_draw();
        });
        tmpl_box.append(&btn);
    }
    sidebar_content.append(&tmpl_box);

    // ── Pages list ──
    let page_label = gtk4::Label::builder()
        .label("Pages")
        .css_classes(vec!["heading".to_string()])
        .halign(gtk4::Align::Start)
        .margin_top(16)
        .build();
    sidebar_content.append(&page_label);

    let page_list = gtk4::ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(vec!["navigation-sidebar".to_string()])
        .build();

    for i in 1..=5 {
        let template_name = match i {
            1 => "Lined",
            2 => "Blank",
            3 => "Grid",
            4 => "Dotted",
            _ => "Lined",
        };
        let row = adw::ActionRow::builder()
            .title(&format!("Page {}", i))
            .subtitle(template_name)
            .activatable(true)
            .build();
        row.add_prefix(&gtk4::Image::from_icon_name("document-page-setup-symbolic"));
        page_list.append(&row);
    }

    // Click page → switch template and clear
    let cs_page = canvas_state.clone();
    let da_page = canvas.clone();
    page_list.connect_row_activated(move |_list, row| {
        if let Some(action_row) = row.downcast_ref::<adw::ActionRow>() {
            let sub = action_row.subtitle().map(|s| s.to_string()).unwrap_or_default();
            let mut s = cs_page.borrow_mut();
            s.template = match sub.as_str() {
                "Lined" => VisualTemplate::Lined,
                "Grid" => VisualTemplate::Grid,
                "Dotted" => VisualTemplate::Dotted,
                _ => VisualTemplate::Blank,
            };
            s.strokes.clear();
            s.splines.clear();
            s.history.clear();
            da_page.queue_draw();
        }
    });
    sidebar_content.append(&page_list);

    // Add page button
    let add_page_btn = gtk4::Button::builder()
        .label("+ Add Page")
        .css_classes(vec!["flat".to_string()])
        .margin_top(8)
        .build();
    sidebar_content.append(&add_page_btn);

    let scroll = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .child(&sidebar_content)
        .build();

    sidebar_box.append(&scroll);
    sidebar_box
}

// ─────────────────────────────────────────────────────────────
// Toolbar — GoodNotes-style bottom bar
// ─────────────────────────────────────────────────────────────
fn build_toolbar(
    canvas_state: &Rc<RefCell<CanvasState>>,
    canvas: &gtk4::DrawingArea,
) -> (gtk4::Box, gtk4::Label) {
    let toolbar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .halign(gtk4::Align::Center)
        .margin_top(6)
        .margin_bottom(6)
        .css_classes(vec!["toolbar".to_string()])
        .build();

    // ── Tool buttons ──
    let tools: Vec<(&str, &str, ToolType)> = vec![
        ("document-edit-symbolic", "Pen (P)", ToolType::Pen),
        ("applications-graphics-symbolic", "Brush (B)", ToolType::Brush),
        ("draw-pen-symbolic", "Pencil (D)", ToolType::Pencil),
        ("view-list-symbolic", "Marker (M)", ToolType::Marker),
        ("selection-mode-symbolic", "Highlighter (H)", ToolType::Highlighter),
        ("edit-clear-symbolic", "Eraser (E)", ToolType::Eraser),
    ];

    let mut tool_buttons: Vec<gtk4::ToggleButton> = Vec::new();
    for (icon, tooltip, tool_type) in &tools {
        let btn = gtk4::ToggleButton::builder()
            .icon_name(*icon)
            .tooltip_text(*tooltip)
            .css_classes(vec!["flat".to_string(), "tool-button".to_string()])
            .build();
        if let Some(first) = tool_buttons.first() {
            btn.set_group(Some(first));
        }
        let cs = canvas_state.clone();
        let da = canvas.clone();
        let tt = *tool_type;
        btn.connect_toggled(move |btn| {
            if btn.is_active() {
                cs.borrow_mut().current_tool = tt;
                da.queue_draw();
            }
        });
        if *tool_type == ToolType::Pen {
            btn.set_active(true);
        }
        tool_buttons.push(btn.clone());
        toolbar.append(&btn);
    }

    // Separator
    append_separator(&toolbar);

    // ── Color swatches (proper drawn circles) ──
    let preset_colors: Vec<(Color, &str)> = vec![
        (Color::BLACK, "Black"),
        (Color::from_rgba(0.2, 0.2, 0.2, 1.0), "Dark Gray"),
        (Color::RED, "Red"),
        (Color::from_rgba(0.85, 0.2, 0.1, 1.0), "Dark Red"),
        (Color::from_rgba(0.1, 0.4, 0.9, 1.0), "Blue"),
        (Color::from_rgba(0.0, 0.6, 0.3, 1.0), "Green"),
        (Color::from_rgba(0.95, 0.55, 0.0, 1.0), "Orange"),
        (Color::from_rgba(0.55, 0.15, 0.75, 1.0), "Purple"),
    ];

    for (color, name) in preset_colors {
        let swatch = gtk4::DrawingArea::builder()
            .width_request(24)
            .height_request(24)
            .tooltip_text(name)
            .build();
        let r = color.r as f64;
        let g = color.g as f64;
        let b = color.b as f64;
        swatch.set_draw_func(move |_area, cr, w, h| {
            let cx = w as f64 / 2.0;
            let cy = h as f64 / 2.0;
            let radius = 9.0;
            // Filled circle
            cr.arc(cx, cy, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.set_source_rgb(r, g, b);
            cr.fill().ok();
            // Border
            cr.arc(cx, cy, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.3);
            cr.set_line_width(1.5);
            cr.stroke().ok();
        });

        let swatch_btn = gtk4::Button::builder()
            .child(&swatch)
            .tooltip_text(name)
            .css_classes(vec!["flat".to_string(), "color-swatch".to_string()])
            .build();
        let cs = canvas_state.clone();
        let da = canvas.clone();
        swatch_btn.connect_clicked(move |_| {
            cs.borrow_mut().stroke_color = color;
            da.queue_draw();
        });
        toolbar.append(&swatch_btn);
    }

    // Custom color picker button via GTK4 ColorDialog
    let custom_color_btn = gtk4::Button::builder()
        .tooltip_text("Custom Color...")
        .css_classes(vec!["flat".to_string()])
        .build();
    // Rainbow swatch for custom color
    let rainbow_swatch = gtk4::DrawingArea::builder()
        .width_request(24)
        .height_request(24)
        .build();
    rainbow_swatch.set_draw_func(|_area, cr, w, h| {
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let r = 9.0;
        // Draw rainbow ring
        for deg in 0..360 {
            let angle = (deg as f64) * std::f64::consts::PI / 180.0;
            let (rd, gd, bd) = hsv_to_rgb(deg as f64, 1.0, 1.0);
            cr.set_source_rgb(rd, gd, bd);
            cr.arc(cx, cy, r, angle, angle + 0.02);
            cr.line_to(cx, cy);
            cr.fill().ok();
        }
        // White center dot
        cr.arc(cx, cy, 4.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.fill().ok();
    });
    custom_color_btn.set_child(Some(&rainbow_swatch));

    let cs_custom = canvas_state.clone();
    let da_custom = canvas.clone();
    custom_color_btn.connect_clicked(move |btn| {
        let dialog = gtk4::ColorDialog::builder()
            .title("Pick a Color")
            .modal(true)
            .build();
        let cs_c = cs_custom.clone();
        let da_c = da_custom.clone();
        let win = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
        dialog.choose_rgba(win.as_ref(), None, None::<&gio::Cancellable>, move |result| {
            if let Ok(rgba) = result {
                let c = Color::from_rgba(
                    rgba.red() as f32,
                    rgba.green() as f32,
                    rgba.blue() as f32,
                    rgba.alpha() as f32,
                );
                cs_c.borrow_mut().stroke_color = c;
                da_c.queue_draw();
            }
        });
    });
    toolbar.append(&custom_color_btn);

    append_separator(&toolbar);

    // ── Width slider ──
    let width_adj = gtk4::Adjustment::new(2.5, 0.5, 20.0, 0.5, 1.0, 0.0);
    let width_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&width_adj)
        .draw_value(true)
        .width_request(120)
        .tooltip_text("Stroke Width")
        .build();
    let cs_w = canvas_state.clone();
    width_adj.connect_value_changed(move |adj| {
        cs_w.borrow_mut().stroke_width = adj.value() as f32;
    });
    toolbar.append(&width_scale);

    append_separator(&toolbar);

    // ── Zoom controls ──
    let zoom_out = gtk4::Button::builder()
        .icon_name("zoom-out-symbolic")
        .tooltip_text("Zoom Out (Ctrl+-)")
        .css_classes(vec!["flat".to_string()])
        .build();

    let zoom_label = gtk4::Label::builder()
        .label("100%")
        .width_chars(5)
        .build();

    let zoom_in = gtk4::Button::builder()
        .icon_name("zoom-in-symbolic")
        .tooltip_text("Zoom In (Ctrl+=)")
        .css_classes(vec!["flat".to_string()])
        .build();

    let zoom_fit = gtk4::Button::builder()
        .icon_name("zoom-fit-best-symbolic")
        .tooltip_text("Fit Page (Ctrl+0)")
        .css_classes(vec!["flat".to_string()])
        .build();

    // Wire zoom out
    let cs_zo = canvas_state.clone();
    let da_zo = canvas.clone();
    let zl_zo = zoom_label.clone();
    zoom_out.connect_clicked(move |_| {
        let mut s = cs_zo.borrow_mut();
        s.viewport.zoom = (s.viewport.zoom * 0.8).max(0.1);
        zl_zo.set_label(&format!("{:.0}%", s.viewport.zoom * 100.0));
        da_zo.queue_draw();
    });

    // Wire zoom in
    let cs_zi = canvas_state.clone();
    let da_zi = canvas.clone();
    let zl_zi = zoom_label.clone();
    zoom_in.connect_clicked(move |_| {
        let mut s = cs_zi.borrow_mut();
        s.viewport.zoom = (s.viewport.zoom * 1.25).min(10.0);
        zl_zi.set_label(&format!("{:.0}%", s.viewport.zoom * 100.0));
        da_zi.queue_draw();
    });

    // Wire zoom fit
    let cs_zf = canvas_state.clone();
    let da_zf = canvas.clone();
    let zl_zf = zoom_label.clone();
    zoom_fit.connect_clicked(move |_| {
        let mut s = cs_zf.borrow_mut();
        s.viewport.zoom = 1.0;
        s.viewport.offset_x = 50.0;
        s.viewport.offset_y = 30.0;
        zl_zf.set_label("100%");
        da_zf.queue_draw();
    });

    toolbar.append(&zoom_out);
    toolbar.append(&zoom_label);
    toolbar.append(&zoom_in);
    toolbar.append(&zoom_fit);

    (toolbar, zoom_label.clone())
}

fn append_separator(toolbar: &gtk4::Box) {
    let sep = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep.set_margin_start(8);
    sep.set_margin_end(8);
    toolbar.append(&sep);
}

fn build_app_menu() -> gio::Menu {
    let menu = gio::Menu::new();

    let file_section = gio::Menu::new();
    file_section.append(Some("New Notebook"), Some("win.new-notebook"));
    file_section.append(Some("Export as PNG..."), Some("win.export"));
    menu.append_section(None, &file_section);

    let view_section = gio::Menu::new();
    view_section.append(Some("Fullscreen"), Some("win.fullscreen"));
    menu.append_section(None, &view_section);

    let app_section = gio::Menu::new();
    app_section.append(Some("Preferences"), Some("app.preferences"));
    app_section.append(Some("About S Notes"), Some("app.about"));
    menu.append_section(None, &app_section);

    menu
}

/// HSV to RGB conversion for rainbow color swatch
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}
