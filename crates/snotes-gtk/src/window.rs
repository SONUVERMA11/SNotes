//! Main application window

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::canvas_widget;
use crate::themes::ThemeManager;

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
        css_provider.load_from_string(&theme_css);
        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().unwrap(),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Header bar
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("S Notes", "Untitled Notebook");
        header.set_title_widget(Some(&title));

        let undo_btn = gtk4::Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text("Undo (Ctrl+Z)")
            .action_name("win.undo")
            .build();

        let redo_btn = gtk4::Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text("Redo (Ctrl+Shift+Z)")
            .action_name("win.redo")
            .build();

        let export_btn = gtk4::Button::builder()
            .icon_name("document-save-as-symbolic")
            .tooltip_text("Export (Ctrl+Shift+E)")
            .build();

        let menu_btn = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Main Menu")
            .menu_model(&build_app_menu())
            .build();

        header.pack_start(&undo_btn);
        header.pack_start(&redo_btn);
        header.pack_end(&menu_btn);
        header.pack_end(&export_btn);

        // Sidebar
        let sidebar = build_sidebar();

        // Canvas widget (real drawing surface)
        let (canvas, _canvas_state) = canvas_widget::create_canvas_widget();

        // Tool palette bar
        let toolbar = build_toolbar();

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
            .title("S Notes")
            .default_width(1400)
            .default_height(900)
            .content(&main_box)
            .build();

        // Accessibility
        undo_btn.update_property(&[gtk4::accessible::Property::Label("Undo")]);
        redo_btn.update_property(&[gtk4::accessible::Property::Label("Redo")]);
        export_btn.update_property(&[gtk4::accessible::Property::Label("Export")]);

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn build_sidebar() -> gtk4::Box {
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
            .css_classes(vec!["notebook-row".to_string()])
            .build();
        let prefix_label = gtk4::Label::new(Some(color));
        row.add_prefix(&prefix_label);
        list.append(&row);
    }

    sidebar_content.append(&list);

    // Page list section
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
        let row = adw::ActionRow::builder()
            .title(&format!("Page {}", i))
            .subtitle("Blank")
            .build();
        row.add_prefix(&gtk4::Image::from_icon_name("document-page-setup-symbolic"));
        page_list.append(&row);
    }
    sidebar_content.append(&page_list);

    sidebar_box.append(&sidebar_content);
    sidebar_box
}

fn build_toolbar() -> gtk4::Box {
    let toolbar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .halign(gtk4::Align::Center)
        .margin_top(6)
        .margin_bottom(6)
        .css_classes(vec!["toolbar".to_string()])
        .build();

    let tools = vec![
        ("edit-symbolic", "Pen (P)"),
        ("draw-pen-symbolic", "Brush (B)"),
        ("draw-eraser-symbolic", "Eraser (E)"),
        ("draw-highlight-symbolic", "Highlighter (H)"),
        ("edit-select-all-symbolic", "Select (S)"),
        ("shape-rectangle-symbolic", "Shape (R)"),
        ("format-text-bold-symbolic", "Text (T)"),
    ];

    for (icon, tooltip) in &tools {
        let btn = gtk4::ToggleButton::builder()
            .icon_name(*icon)
            .tooltip_text(*tooltip)
            .css_classes(vec!["flat".to_string(), "tool-button".to_string()])
            .build();
        toolbar.append(&btn);
    }

    // Separator
    let sep = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep.set_margin_start(8);
    sep.set_margin_end(8);
    toolbar.append(&sep);

    // Color picker
    let color_btn = gtk4::ColorDialogButton::builder()
        .tooltip_text("Stroke Color")
        .build();
    toolbar.append(&color_btn);

    // Width slider
    let width_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(2.0, 0.5, 20.0, 0.5, 1.0, 0.0))
        .draw_value(false)
        .width_request(120)
        .tooltip_text("Stroke Width")
        .build();
    toolbar.append(&width_scale);

    // Separator
    let sep2 = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep2.set_margin_start(8);
    sep2.set_margin_end(8);
    toolbar.append(&sep2);

    // Zoom controls
    let zoom_out = gtk4::Button::builder()
        .icon_name("zoom-out-symbolic")
        .tooltip_text("Zoom Out")
        .css_classes(vec!["flat".to_string()])
        .build();
    toolbar.append(&zoom_out);

    let zoom_label = gtk4::Label::builder()
        .label("100%")
        .width_chars(5)
        .build();
    toolbar.append(&zoom_label);

    let zoom_in = gtk4::Button::builder()
        .icon_name("zoom-in-symbolic")
        .tooltip_text("Zoom In")
        .css_classes(vec!["flat".to_string()])
        .build();
    toolbar.append(&zoom_in);

    let zoom_fit = gtk4::Button::builder()
        .icon_name("zoom-fit-best-symbolic")
        .tooltip_text("Fit Page")
        .css_classes(vec!["flat".to_string()])
        .build();
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
