//! S Notes — GTK4/libadwaita frontend

mod app;
mod window;
mod canvas_widget;
mod settings;
mod themes;
mod shortcuts;

use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("snotes=debug,snotes_core=debug")),
        )
        .init();

    tracing::info!("Starting S Notes v{}", env!("CARGO_PKG_VERSION"));

    // Run the GTK application
    let exit_code = app::run();
    std::process::exit(exit_code);
}
