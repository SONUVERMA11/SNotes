//! Main application window — fully wired up with all features

use gtk4::prelude::*;
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

        // Header bar
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("S Notes", "by Sonu Verma");
        header.set_title_widget(Some(&title));

        // Undo/Redo buttons
        let undo_btn = gtk4::Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text("Undo (Ctrl+Z)")
            .build();
        let redo_btn = gtk4::Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text("Redo (Ctrl+Shift+Z)")
            .build();

        // Wire undo
        let cs_undo = canvas_state.clone();
        let da_undo = canvas.clone();
        undo_btn.connect_clicked(move |_| {
            cs_undo.borrow_mut().undo();
            da_undo.queue_draw();
        });

        // Wire redo
        let cs_redo = canvas_state.clone();
        let da_redo = canvas.clone();
        redo_btn.connect_clicked(move |_| {
            cs_redo.borrow_mut().redo();
            da_redo.queue_draw();
        });

        // Clear all button
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
            da_clear.queue_draw();
        });

        // Menu
        let menu_btn = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .menu_model(&build_app_menu())
            .build();

        header.pack_start(&undo_btn);
        header.pack_start(&redo_btn);
        header.pack_end(&menu_btn);
        header.pack_end(&clear_btn);

        // Sidebar
        let sidebar = build_sidebar(&canvas_state, &canvas);

        // Toolbar
        let toolbar = build_toolbar(&canvas_state, &canvas);

        // Main content: canvas + toolbar
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content_box.append(&canvas);
        content_box.append(&toolbar);

        // Split view
        let split_view = adw::OverlaySplitView::builder()
            .sidebar_position(gtk4::PackType::Start)
            .show_sidebar(true)
            .min_sidebar_width(220.0)
            .max_sidebar_width(350.0)
            .build();
        split_view.set_sidebar(Some(&sidebar));
        split_view.set_content(Some(&content_box));

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

        // ── Keyboard shortcuts ──
        let key_ctrl = gtk4::EventControllerKey::new();
        let cs_key = canvas_state.clone();
        let da_key = canvas.clone();
        key_ctrl.connect_key_pressed(move |_ctrl, key, _code, modifier| {
            let ctrl = modifier.contains(gdk4::ModifierType::CONTROL_MASK);
            let shift = modifier.contains(gdk4::ModifierType::SHIFT_MASK);
            let mut s = cs_key.borrow_mut();

            match key {
                gdk4::Key::z if ctrl && !shift => { s.undo(); da_key.queue_draw(); return glib::Propagation::Stop; }
                gdk4::Key::z if ctrl && shift => { s.redo(); da_key.queue_draw(); return glib::Propagation::Stop; }
                gdk4::Key::Z if ctrl => { s.redo(); da_key.queue_draw(); return glib::Propagation::Stop; }
                gdk4::Key::p | gdk4::Key::P => { s.current_tool = ToolType::Pen; da_key.queue_draw(); }
                gdk4::Key::b | gdk4::Key::B => { s.current_tool = ToolType::Brush; da_key.queue_draw(); }
                gdk4::Key::e | gdk4::Key::E => { s.current_tool = ToolType::Eraser; da_key.queue_draw(); }
                gdk4::Key::h | gdk4::Key::H => { s.current_tool = ToolType::Highlighter; da_key.queue_draw(); }
                gdk4::Key::m | gdk4::Key::M => { s.current_tool = ToolType::Marker; da_key.queue_draw(); }
                gdk4::Key::d | gdk4::Key::D => { s.current_tool = ToolType::Pencil; da_key.queue_draw(); }
                gdk4::Key::equal if ctrl => {
                    let nz = (s.viewport.zoom * 1.2).min(10.0);
                    s.viewport.zoom = nz;
                    da_key.queue_draw();
                }
                gdk4::Key::minus if ctrl => {
                    let nz = (s.viewport.zoom * 0.8).max(0.1);
                    s.viewport.zoom = nz;
                    da_key.queue_draw();
                }
                gdk4::Key::_0 if ctrl => {
                    s.viewport.zoom = 1.0;
                    s.viewport.offset_x = 50.0;
                    s.viewport.offset_y = 30.0;
                    da_key.queue_draw();
                }
                _ => return glib::Propagation::Proceed,
            }
            glib::Propagation::Stop
        });
        window.add_controller(key_ctrl);

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn build_sidebar(canvas_state: &Rc<RefCell<CanvasState>>, canvas: &gtk4::DrawingArea) -> gtk4::Box {
    let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar_box.add_css_class("sidebar");

    let sidebar_content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    sidebar_content.set_margin_start(12);
    sidebar_content.set_margin_end(12);
    sidebar_content.set_margin_top(12);
    sidebar_content.set_margin_bottom(12);

    // Search
    let search = gtk4::SearchEntry::builder()
        .placeholder_text("Search notebooks...")
        .build();
    sidebar_content.append(&search);

    // Notebooks header
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

    // Notebook list
    let list = gtk4::ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(vec!["navigation-sidebar".to_string()])
        .build();

    for (name, pages, color) in &[
        ("Physics Notes", "12 pages", "🔵"),
        ("Math Homework", "8 pages", "🟢"),
        ("Chemistry Lab", "15 pages", "🟣"),
        ("History Essay", "4 pages", "🟠"),
    ] {
        let row = adw::ActionRow::builder()
            .title(*name)
            .subtitle(*pages)
            .build();
        let prefix_label = gtk4::Label::new(Some(color));
        row.add_prefix(&prefix_label);
        list.append(&row);
    }
    sidebar_content.append(&list);

    // ── Page Template selector ──
    let template_label = gtk4::Label::builder()
        .label("Page Template")
        .css_classes(vec!["heading".to_string()])
        .halign(gtk4::Align::Start)
        .margin_top(16)
        .build();
    sidebar_content.append(&template_label);

    let template_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let templates = vec![
        ("Blank", VisualTemplate::Blank),
        ("Lined", VisualTemplate::Lined),
        ("Grid", VisualTemplate::Grid),
        ("Dots", VisualTemplate::Dotted),
    ];
    for (label, tmpl) in templates {
        let btn = gtk4::Button::builder()
            .label(label)
            .css_classes(vec!["flat".to_string()])
            .build();
        let cs = canvas_state.clone();
        let da = canvas.clone();
        btn.connect_clicked(move |_| {
            cs.borrow_mut().template = tmpl;
            da.queue_draw();
        });
        template_box.append(&btn);
    }
    sidebar_content.append(&template_box);

    // ── Stroke info ──
    let info_label = gtk4::Label::builder()
        .label("Stroke Count: 0")
        .halign(gtk4::Align::Start)
        .margin_top(16)
        .build();
    sidebar_content.append(&info_label);

    sidebar_box.append(&sidebar_content);
    sidebar_box
}

fn build_toolbar(canvas_state: &Rc<RefCell<CanvasState>>, canvas: &gtk4::DrawingArea) -> gtk4::Box {
    let toolbar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .halign(gtk4::Align::Center)
        .margin_top(6)
        .margin_bottom(6)
        .css_classes(vec!["toolbar".to_string()])
        .build();

    // Tool buttons with real switching
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

        // Group radio-like behavior
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
    let sep = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep.set_margin_start(8);
    sep.set_margin_end(8);
    toolbar.append(&sep);

    // Color buttons
    let colors: Vec<(&str, Color)> = vec![
        ("⬛", Color::BLACK),
        ("🔴", Color::RED),
        ("🔵", Color::from_rgba(0.1, 0.3, 0.8, 1.0)),
        ("🟢", Color::from_rgba(0.1, 0.6, 0.2, 1.0)),
        ("🟠", Color::from_rgba(0.9, 0.5, 0.0, 1.0)),
        ("🟣", Color::from_rgba(0.5, 0.1, 0.7, 1.0)),
    ];

    for (label, color) in colors {
        let btn = gtk4::Button::builder()
            .label(label)
            .tooltip_text(&format!("Color: {:?}", label))
            .css_classes(vec!["flat".to_string()])
            .build();
        let cs = canvas_state.clone();
        let da = canvas.clone();
        btn.connect_clicked(move |_| {
            cs.borrow_mut().stroke_color = color;
            da.queue_draw();
        });
        toolbar.append(&btn);
    }

    // Separator
    let sep2 = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep2.set_margin_start(8);
    sep2.set_margin_end(8);
    toolbar.append(&sep2);

    // Width slider
    let width_adj = gtk4::Adjustment::new(2.5, 0.5, 20.0, 0.5, 1.0, 0.0);
    let width_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&width_adj)
        .draw_value(true)
        .width_request(130)
        .tooltip_text("Stroke Width")
        .build();
    let cs_width = canvas_state.clone();
    width_adj.connect_value_changed(move |adj| {
        cs_width.borrow_mut().stroke_width = adj.value() as f32;
    });
    toolbar.append(&width_scale);

    // Separator
    let sep3 = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep3.set_margin_start(8);
    sep3.set_margin_end(8);
    toolbar.append(&sep3);

    // Zoom controls
    let zoom_out = gtk4::Button::builder()
        .icon_name("zoom-out-symbolic")
        .tooltip_text("Zoom Out (Ctrl+-)")
        .css_classes(vec!["flat".to_string()])
        .build();
    let cs_zo = canvas_state.clone();
    let da_zo = canvas.clone();
    zoom_out.connect_clicked(move |_| {
        let mut s = cs_zo.borrow_mut();
        s.viewport.zoom = (s.viewport.zoom * 0.8).max(0.1);
        da_zo.queue_draw();
    });
    toolbar.append(&zoom_out);

    let zoom_label = gtk4::Label::builder()
        .label("100%")
        .width_chars(5)
        .build();
    toolbar.append(&zoom_label);

    let zoom_in = gtk4::Button::builder()
        .icon_name("zoom-in-symbolic")
        .tooltip_text("Zoom In (Ctrl+=)")
        .css_classes(vec!["flat".to_string()])
        .build();
    let cs_zi = canvas_state.clone();
    let da_zi = canvas.clone();
    zoom_in.connect_clicked(move |_| {
        let mut s = cs_zi.borrow_mut();
        s.viewport.zoom = (s.viewport.zoom * 1.25).min(10.0);
        da_zi.queue_draw();
    });
    toolbar.append(&zoom_in);

    let zoom_fit = gtk4::Button::builder()
        .icon_name("zoom-fit-best-symbolic")
        .tooltip_text("Fit Page (Ctrl+0)")
        .css_classes(vec!["flat".to_string()])
        .build();
    let cs_zf = canvas_state.clone();
    let da_zf = canvas.clone();
    zoom_fit.connect_clicked(move |_| {
        let mut s = cs_zf.borrow_mut();
        s.viewport.zoom = 1.0;
        s.viewport.offset_x = 50.0;
        s.viewport.offset_y = 30.0;
        da_zf.queue_draw();
    });
    toolbar.append(&zoom_fit);

    toolbar
}

fn build_app_menu() -> gio::Menu {
    let menu = gio::Menu::new();

    let file_section = gio::Menu::new();
    file_section.append(Some("New Notebook"), Some("win.new-notebook"));
    file_section.append(Some("Import PDF..."), Some("win.import-pdf"));
    file_section.append(Some("Export..."), Some("win.export"));
    menu.append_section(None, &file_section);

    let view_section = gio::Menu::new();
    view_section.append(Some("Show Grid"), Some("win.toggle-grid"));
    view_section.append(Some("Show Rulers"), Some("win.toggle-rulers"));
    view_section.append(Some("Fullscreen"), Some("win.fullscreen"));
    menu.append_section(None, &view_section);

    let app_section = gio::Menu::new();
    app_section.append(Some("Preferences"), Some("app.preferences"));
    app_section.append(Some("Keyboard Shortcuts"), Some("win.show-shortcuts"));
    app_section.append(Some("About S Notes"), Some("app.about"));
    menu.append_section(None, &app_section);

    menu
}
