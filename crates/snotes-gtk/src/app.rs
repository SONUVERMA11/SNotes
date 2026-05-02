//! GTK4 Application setup — with About dialog, Preferences, and all actions

use gtk4::prelude::*;
use gtk4::gio;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::window::SNotesWindow;

const APP_ID: &str = "org.snotes.App";

/// Run the GTK4 application, returns exit code
pub fn run() -> i32 {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.connect_startup(|app| {
        setup_shortcuts(app);
        setup_actions(app);
    });

    app.run().into()
}

fn build_ui(app: &adw::Application) {
    let window = SNotesWindow::new(app);
    window.present();
}

fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.fullscreen", &["F11"]);
}

fn setup_actions(app: &adw::Application) {
    // About action
    let about_action = gio::SimpleAction::new("about", None);
    let app_weak = app.downgrade();
    about_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak.upgrade() {
            let about = adw::AboutWindow::builder()
                .application_name("S Notes")
                .version(env!("CARGO_PKG_VERSION"))
                .developer_name("Sonu Verma")
                .website("https://github.com/SONUVERMA11/SNotes")
                .issue_url("https://github.com/SONUVERMA11/SNotes/issues")
                .license_type(gtk4::License::Gpl30)
                .comments("Linux-native handwriting & annotation app.\nBuilt with Rust, GTK4, and libadwaita.")
                .build();

            about.add_credit_section(Some("Created by"), &["Sonu Verma https://github.com/SONUVERMA11"]);
            about.set_developers(&["Sonu Verma <https://github.com/SONUVERMA11>"]);

            if let Some(win) = app.active_window() {
                about.set_transient_for(Some(&win));
            }
            about.present();
        }
    });
    app.add_action(&about_action);

    // Preferences action
    let prefs_action = gio::SimpleAction::new("preferences", None);
    let app_weak2 = app.downgrade();
    prefs_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak2.upgrade() {
            let prefs = build_preferences_window();
            if let Some(win) = app.active_window() {
                prefs.set_transient_for(Some(&win));
            }
            prefs.present();
        }
    });
    app.add_action(&prefs_action);

    // Quit action
    let quit_action = gio::SimpleAction::new("quit", None);
    let app_weak3 = app.downgrade();
    quit_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak3.upgrade() {
            app.quit();
        }
    });
    app.add_action(&quit_action);
}

fn build_preferences_window() -> adw::PreferencesWindow {
    let window = adw::PreferencesWindow::builder()
        .title("Preferences")
        .default_width(700)
        .default_height(500)
        .build();

    // ── Appearance page ──
    let appearance_page = adw::PreferencesPage::builder()
        .title("Appearance")
        .icon_name("applications-graphics-symbolic")
        .build();

    let theme_group = adw::PreferencesGroup::builder()
        .title("Theme")
        .description("Choose your preferred appearance")
        .build();

    let theme_row = adw::ComboRow::builder()
        .title("Color Scheme")
        .subtitle("App-wide color theme")
        .build();
    let theme_model = gtk4::StringList::new(&["Dark", "Light", "Sepia", "System"]);
    theme_row.set_model(Some(&theme_model));
    theme_group.add(&theme_row);
    appearance_page.add(&theme_group);

    // ── Canvas page ──
    let canvas_page = adw::PreferencesPage::builder()
        .title("Canvas")
        .icon_name("document-page-setup-symbolic")
        .build();

    let canvas_group = adw::PreferencesGroup::builder()
        .title("Drawing")
        .build();

    let predictive_row = adw::SwitchRow::builder()
        .title("Predictive Ink")
        .subtitle("Reduce perceived latency with 2-frame lookahead")
        .active(true)
        .build();
    canvas_group.add(&predictive_row);

    let smooth_row = adw::SwitchRow::builder()
        .title("Stroke Smoothing")
        .subtitle("Apply Bézier curve fitting to strokes")
        .active(true)
        .build();
    canvas_group.add(&smooth_row);

    let pressure_row = adw::SwitchRow::builder()
        .title("Pressure Sensitivity")
        .subtitle("Use stylus pressure for variable width")
        .active(true)
        .build();
    canvas_group.add(&pressure_row);

    canvas_page.add(&canvas_group);

    let grid_group = adw::PreferencesGroup::builder()
        .title("Grid & Snapping")
        .build();

    let snap_row = adw::SwitchRow::builder()
        .title("Snap to Grid")
        .subtitle("Align shapes and text to grid intersections")
        .active(false)
        .build();
    grid_group.add(&snap_row);

    canvas_page.add(&grid_group);

    // ── Input page ──
    let input_page = adw::PreferencesPage::builder()
        .title("Input")
        .icon_name("input-tablet-symbolic")
        .build();

    let input_group = adw::PreferencesGroup::builder()
        .title("Stylus")
        .build();

    let palm_row = adw::SwitchRow::builder()
        .title("Palm Rejection")
        .subtitle("Ignore accidental palm touches")
        .active(true)
        .build();
    input_group.add(&palm_row);

    let barrel_row = adw::ComboRow::builder()
        .title("Barrel Button 1")
        .subtitle("Action when pressing the first stylus button")
        .build();
    let barrel_model = gtk4::StringList::new(&["Eraser", "Undo", "Redo", "Color Picker", "Right Click", "None"]);
    barrel_row.set_model(Some(&barrel_model));
    input_group.add(&barrel_row);

    input_page.add(&input_group);

    // ── Auto-Save page ──
    let save_page = adw::PreferencesPage::builder()
        .title("General")
        .icon_name("preferences-system-symbolic")
        .build();

    let save_group = adw::PreferencesGroup::builder()
        .title("Auto-Save")
        .build();

    let autosave_row = adw::SwitchRow::builder()
        .title("Enable Auto-Save")
        .subtitle("Automatically save your work periodically")
        .active(true)
        .build();
    save_group.add(&autosave_row);

    let interval_row = adw::SpinRow::builder()
        .title("Save Interval (seconds)")
        .subtitle("How often to auto-save")
        .adjustment(&gtk4::Adjustment::new(30.0, 5.0, 300.0, 5.0, 30.0, 0.0))
        .build();
    save_group.add(&interval_row);

    save_page.add(&save_group);

    let info_group = adw::PreferencesGroup::builder()
        .title("About")
        .build();
    let author_row = adw::ActionRow::builder()
        .title("Author")
        .subtitle("Sonu Verma")
        .build();
    author_row.add_suffix(&gtk4::Image::from_icon_name("avatar-default-symbolic"));
    info_group.add(&author_row);

    let github_row = adw::ActionRow::builder()
        .title("GitHub")
        .subtitle("github.com/SONUVERMA11/SNotes")
        .activatable(true)
        .build();
    github_row.add_suffix(&gtk4::Image::from_icon_name("web-browser-symbolic"));
    info_group.add(&github_row);
    save_page.add(&info_group);

    window.add(&appearance_page);
    window.add(&canvas_page);
    window.add(&input_page);
    window.add(&save_page);

    window
}
