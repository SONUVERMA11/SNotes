//! Settings / Preferences dialog

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

/// Build the preferences window
pub fn build_preferences(app: &adw::Application) -> adw::PreferencesWindow {
    let window = adw::PreferencesWindow::builder()
        .title("S Notes Preferences")
        .application(app)
        .default_width(700)
        .default_height(600)
        .build();

    // ── Appearance Page ───────────────────────────────────────
    let appearance_page = adw::PreferencesPage::builder()
        .title("Appearance")
        .icon_name("preferences-desktop-appearance-symbolic")
        .build();

    let theme_group = adw::PreferencesGroup::builder()
        .title("Theme")
        .description("Choose the application color scheme")
        .build();

    let theme_row = adw::ComboRow::builder()
        .title("Color Scheme")
        .subtitle("Select the visual theme")
        .build();
    let theme_list = gtk4::StringList::new(&["Dark", "Light", "Sepia", "Custom"]);
    theme_row.set_model(Some(&theme_list));
    theme_group.add(&theme_row);

    appearance_page.add(&theme_group);

    // ── Canvas Page ───────────────────────────────────────────
    let canvas_page = adw::PreferencesPage::builder()
        .title("Canvas")
        .icon_name("edit-symbolic")
        .build();

    let drawing_group = adw::PreferencesGroup::builder()
        .title("Drawing")
        .build();

    let pressure_row = adw::SwitchRow::builder()
        .title("Pressure Sensitivity")
        .subtitle("Vary stroke width based on pen pressure")
        .active(true)
        .build();
    drawing_group.add(&pressure_row);

    let palm_row = adw::SwitchRow::builder()
        .title("Palm Rejection")
        .subtitle("Ignore accidental touch input while drawing")
        .active(true)
        .build();
    drawing_group.add(&palm_row);

    let stylus_only_row = adw::SwitchRow::builder()
        .title("Stylus-Only Mode")
        .subtitle("Disable all touch input, use stylus only")
        .active(false)
        .build();
    drawing_group.add(&stylus_only_row);

    let predictive_row = adw::SwitchRow::builder()
        .title("Predictive Ink")
        .subtitle("Reduce latency by predicting stroke trajectory")
        .active(true)
        .build();
    drawing_group.add(&predictive_row);

    let grid_snap_row = adw::SwitchRow::builder()
        .title("Snap to Grid")
        .subtitle("Snap shapes and endpoints to grid intersections")
        .active(false)
        .build();
    drawing_group.add(&grid_snap_row);

    canvas_page.add(&drawing_group);

    // ── Input Page ────────────────────────────────────────────
    let input_page = adw::PreferencesPage::builder()
        .title("Input")
        .icon_name("input-tablet-symbolic")
        .build();

    let barrel_group = adw::PreferencesGroup::builder()
        .title("Barrel Buttons")
        .description("Configure stylus button actions")
        .build();

    let btn1_row = adw::ComboRow::builder()
        .title("Primary Button")
        .subtitle("Bottom barrel button")
        .build();
    let btn1_list = gtk4::StringList::new(&["Eraser", "Undo", "Redo", "Right Click", "Color Picker", "Pan Canvas"]);
    btn1_row.set_model(Some(&btn1_list));
    barrel_group.add(&btn1_row);

    let btn2_row = adw::ComboRow::builder()
        .title("Secondary Button")
        .subtitle("Top barrel button")
        .build();
    let btn2_list = gtk4::StringList::new(&["Right Click", "Eraser", "Undo", "Redo", "Color Picker", "Pan Canvas"]);
    btn2_row.set_model(Some(&btn2_list));
    barrel_group.add(&btn2_row);

    input_page.add(&barrel_group);

    let pressure_group = adw::PreferencesGroup::builder()
        .title("Pressure Curve")
        .build();

    let gamma_row = adw::SpinRow::builder()
        .title("Pressure Gamma")
        .subtitle("< 1.0 = more sensitive, > 1.0 = less sensitive")
        .adjustment(&gtk4::Adjustment::new(1.0, 0.1, 3.0, 0.1, 0.5, 0.0))
        .build();
    pressure_group.add(&gamma_row);

    let threshold_row = adw::SpinRow::builder()
        .title("Minimum Pressure Threshold")
        .subtitle("Ignore input below this pressure level")
        .adjustment(&gtk4::Adjustment::new(0.01, 0.0, 0.5, 0.01, 0.05, 0.0))
        .build();
    pressure_group.add(&threshold_row);

    input_page.add(&pressure_group);

    // ── Sync Page ─────────────────────────────────────────────
    let sync_page = adw::PreferencesPage::builder()
        .title("Sync")
        .icon_name("emblem-synchronizing-symbolic")
        .build();

    let sync_group = adw::PreferencesGroup::builder()
        .title("Cloud Sync")
        .description("Sync notebooks via WebDAV or Nextcloud")
        .build();

    let sync_enabled = adw::SwitchRow::builder()
        .title("Enable Sync")
        .active(false)
        .build();
    sync_group.add(&sync_enabled);

    let server_row = adw::EntryRow::builder()
        .title("Server URL")
        .build();
    sync_group.add(&server_row);

    let user_row = adw::EntryRow::builder()
        .title("Username")
        .build();
    sync_group.add(&user_row);

    sync_page.add(&sync_group);

    // Add all pages
    window.add(&appearance_page);
    window.add(&canvas_page);
    window.add(&input_page);
    window.add(&sync_page);

    window
}
