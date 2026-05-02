//! Auto-save system — periodic saves with dirty tracking and crash recovery

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Auto-save configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    /// Enable auto-save
    pub enabled: bool,
    /// Interval between saves (seconds)
    pub interval_secs: u64,
    /// Save on focus lost / app switch
    pub save_on_blur: bool,
    /// Maximum number of backup versions to keep
    pub max_backups: usize,
    /// Backup directory
    pub backup_dir: String,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            save_on_blur: true,
            max_backups: 10,
            backup_dir: "~/.local/share/snotes/backups".to_string(),
        }
    }
}

/// Dirty tracking — knows which pages/notebooks need saving
#[derive(Debug, Clone)]
pub struct DirtyTracker {
    pub dirty_pages: Vec<Uuid>,
    pub dirty_notebooks: Vec<Uuid>,
    pub last_save: Option<Instant>,
    pub last_modify: Option<Instant>,
    pub stroke_count_since_save: u32,
}

impl DirtyTracker {
    pub fn new() -> Self {
        Self {
            dirty_pages: Vec::new(),
            dirty_notebooks: Vec::new(),
            last_save: None,
            last_modify: None,
            stroke_count_since_save: 0,
        }
    }

    /// Mark a page as dirty (needs saving)
    pub fn mark_page_dirty(&mut self, page_id: Uuid) {
        if !self.dirty_pages.contains(&page_id) {
            self.dirty_pages.push(page_id);
        }
        self.last_modify = Some(Instant::now());
        self.stroke_count_since_save += 1;
    }

    /// Mark a notebook as dirty
    pub fn mark_notebook_dirty(&mut self, notebook_id: Uuid) {
        if !self.dirty_notebooks.contains(&notebook_id) {
            self.dirty_notebooks.push(notebook_id);
        }
        self.last_modify = Some(Instant::now());
    }

    /// Check if anything needs saving
    pub fn is_dirty(&self) -> bool {
        !self.dirty_pages.is_empty() || !self.dirty_notebooks.is_empty()
    }

    /// Clear all dirty flags after a successful save
    pub fn mark_saved(&mut self) {
        self.dirty_pages.clear();
        self.dirty_notebooks.clear();
        self.last_save = Some(Instant::now());
        self.stroke_count_since_save = 0;
    }

    /// Should we save now?
    pub fn should_save(&self, config: &AutoSaveConfig) -> bool {
        if !config.enabled || !self.is_dirty() {
            return false;
        }

        // Save if enough time has passed since last save
        if let Some(last) = self.last_save {
            return last.elapsed() >= Duration::from_secs(config.interval_secs);
        }

        // No previous save — save if we have modifications
        self.last_modify.is_some()
    }

    /// Get time until next auto-save (for UI display)
    pub fn time_until_save(&self, config: &AutoSaveConfig) -> Option<Duration> {
        if !config.enabled || !self.is_dirty() {
            return None;
        }
        let interval = Duration::from_secs(config.interval_secs);
        if let Some(last) = self.last_save {
            let elapsed = last.elapsed();
            if elapsed < interval {
                return Some(interval - elapsed);
            }
        }
        Some(Duration::ZERO)
    }
}

impl Default for DirtyTracker {
    fn default() -> Self { Self::new() }
}

/// Auto-save manager
pub struct AutoSaveManager {
    pub config: AutoSaveConfig,
    pub tracker: DirtyTracker,
    backup_dir: PathBuf,
}

impl AutoSaveManager {
    pub fn new(config: AutoSaveConfig) -> Self {
        let backup_dir = PathBuf::from(
            config.backup_dir.replace('~', &std::env::var("HOME").unwrap_or_default())
        );
        Self { config, tracker: DirtyTracker::new(), backup_dir }
    }

    /// Create a backup of a file before overwriting
    pub fn create_backup(&self, source: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.backup_dir)?;

        let filename = source.file_name().unwrap_or_default().to_string_lossy();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup_name = format!("{}.{}.bak", filename, timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        std::fs::copy(source, &backup_path)?;
        log::info!("Created backup: {:?}", backup_path);

        // Prune old backups
        self.prune_backups(&filename)?;

        Ok(backup_path)
    }

    /// Remove excess backups, keeping only max_backups
    fn prune_backups(&self, base_name: &str) -> std::io::Result<()> {
        let mut backups: Vec<_> = std::fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(base_name))
            .collect();

        backups.sort_by_key(|e| std::cmp::Reverse(
            e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        ));

        for excess in backups.iter().skip(self.config.max_backups) {
            let _ = std::fs::remove_file(excess.path());
            log::debug!("Pruned old backup: {:?}", excess.path());
        }

        Ok(())
    }

    /// Get list of available backups for a file
    pub fn list_backups(&self, base_name: &str) -> Vec<PathBuf> {
        std::fs::read_dir(&self.backup_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_name().to_string_lossy().starts_with(base_name))
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Write a crash-recovery lock file
    pub fn write_lock(&self, notebook_id: &Uuid) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.backup_dir)?;
        let lock_path = self.backup_dir.join(format!("{}.lock", notebook_id));
        std::fs::write(&lock_path, format!("{}", std::process::id()))?;
        Ok(lock_path)
    }

    /// Remove the lock file (clean shutdown)
    pub fn remove_lock(&self, notebook_id: &Uuid) {
        let lock_path = self.backup_dir.join(format!("{}.lock", notebook_id));
        let _ = std::fs::remove_file(lock_path);
    }

    /// Check if a previous session crashed (stale lock file)
    pub fn check_crash_recovery(&self, notebook_id: &Uuid) -> Option<PathBuf> {
        let lock_path = self.backup_dir.join(format!("{}.lock", notebook_id));
        if lock_path.exists() {
            log::warn!("Found stale lock file — previous session may have crashed");
            Some(lock_path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_tracker() {
        let mut tracker = DirtyTracker::new();
        assert!(!tracker.is_dirty());

        tracker.mark_page_dirty(Uuid::new_v4());
        assert!(tracker.is_dirty());

        tracker.mark_saved();
        assert!(!tracker.is_dirty());
    }

    #[test]
    fn test_should_save() {
        let config = AutoSaveConfig { interval_secs: 0, ..Default::default() };
        let mut tracker = DirtyTracker::new();

        assert!(!tracker.should_save(&config)); // nothing dirty

        tracker.mark_page_dirty(Uuid::new_v4());
        assert!(tracker.should_save(&config)); // dirty + 0 interval
    }

    #[test]
    fn test_auto_save_disabled() {
        let config = AutoSaveConfig { enabled: false, ..Default::default() };
        let mut tracker = DirtyTracker::new();
        tracker.mark_page_dirty(Uuid::new_v4());
        assert!(!tracker.should_save(&config));
    }
}
