//! GTK4 Application setup

use gtk4::prelude::*;
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
    // Global keyboard shortcuts
    app.set_accels_for_action("win.undo", &["<Ctrl>z"]);
    app.set_accels_for_action("win.redo", &["<Ctrl><Shift>z"]);
    app.set_accels_for_action("win.save", &["<Ctrl>s"]);
    app.set_accels_for_action("win.new-page", &["<Ctrl>n"]);
    app.set_accels_for_action("win.export", &["<Ctrl><Shift>e"]);
    app.set_accels_for_action("win.zoom-in", &["<Ctrl>equal"]);
    app.set_accels_for_action("win.zoom-out", &["<Ctrl>minus"]);
    app.set_accels_for_action("win.zoom-fit", &["<Ctrl>0"]);
    app.set_accels_for_action("win.fullscreen", &["F11"]);
    // Tool shortcuts
    app.set_accels_for_action("win.tool-pen", &["p"]);
    app.set_accels_for_action("win.tool-eraser", &["e"]);
    app.set_accels_for_action("win.tool-highlighter", &["h"]);
    app.set_accels_for_action("win.tool-select", &["s"]);
    app.set_accels_for_action("win.tool-shape", &["r"]);
}

fn setup_actions(_app: &adw::Application) {
    // Application-level actions (preferences, about, quit)
}
