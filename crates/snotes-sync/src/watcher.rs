//! Local folder watcher for file-based sync

/// Watches a local directory for changes
pub struct FolderWatcher {
    watch_path: String,
    active: bool,
}

impl FolderWatcher {
    pub fn new(path: &str) -> Self {
        Self { watch_path: path.to_string(), active: false }
    }

    pub fn start(&mut self) {
        tracing::info!("Watching folder: {}", self.watch_path);
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}
