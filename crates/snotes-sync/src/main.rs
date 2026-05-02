//! S Notes Sync Daemon — WebDAV/Nextcloud sync with conflict resolution

mod webdav;
mod watcher;
mod dbus_ipc;
mod nextcloud;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("snotes_sync=info")),
        )
        .init();

    tracing::info!("Starting S Notes Sync Daemon");

    // In production:
    // 1. Register D-Bus service
    // 2. Start file watcher for local changes
    // 3. Start WebDAV sync loop
    // 4. Handle conflict resolution

    tracing::info!("Sync daemon ready, waiting for events...");

    // Keep daemon running
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("Sync daemon shutting down");
}
