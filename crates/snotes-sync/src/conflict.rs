//! Conflict resolution — handles sync conflicts between local and remote versions

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// Keep the local version, discard remote
    KeepLocal,
    /// Keep the remote version, discard local
    KeepRemote,
    /// Keep the newest version (by timestamp)
    KeepNewest,
    /// Keep both — rename the older one with a conflict suffix
    KeepBoth,
    /// Ask the user (shows a dialog)
    AskUser,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        ConflictStrategy::KeepBoth
    }
}

/// A detected sync conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub id: String,
    pub file_path: String,
    pub local_modified: i64,
    pub remote_modified: i64,
    pub local_size: u64,
    pub remote_size: u64,
    pub local_etag: Option<String>,
    pub remote_etag: Option<String>,
    pub resolved: bool,
    pub resolution: Option<ConflictStrategy>,
}

impl SyncConflict {
    pub fn new(file_path: &str, local_modified: i64, remote_modified: i64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: file_path.to_string(),
            local_modified,
            remote_modified,
            local_size: 0,
            remote_size: 0,
            local_etag: None,
            remote_etag: None,
            resolved: false,
            resolution: None,
        }
    }

    /// Auto-resolve using the given strategy
    pub fn auto_resolve(&mut self, strategy: ConflictStrategy) -> ConflictAction {
        self.resolved = true;
        self.resolution = Some(strategy);

        match strategy {
            ConflictStrategy::KeepLocal => ConflictAction::UploadLocal,
            ConflictStrategy::KeepRemote => ConflictAction::DownloadRemote,
            ConflictStrategy::KeepNewest => {
                if self.local_modified >= self.remote_modified {
                    ConflictAction::UploadLocal
                } else {
                    ConflictAction::DownloadRemote
                }
            }
            ConflictStrategy::KeepBoth => {
                ConflictAction::KeepBothRename {
                    conflict_suffix: format!("conflict-{}", chrono_ish_timestamp()),
                }
            }
            ConflictStrategy::AskUser => ConflictAction::Pending,
        }
    }
}

/// Action to take after resolving a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictAction {
    /// Upload local version to remote
    UploadLocal,
    /// Download remote version to local
    DownloadRemote,
    /// Keep both with a renamed conflict copy
    KeepBothRename { conflict_suffix: String },
    /// Not yet resolved — waiting for user input
    Pending,
}

/// Conflict resolver manages a queue of conflicts
pub struct ConflictResolver {
    pub default_strategy: ConflictStrategy,
    pub conflicts: Vec<SyncConflict>,
}

impl ConflictResolver {
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self {
            default_strategy: strategy,
            conflicts: Vec::new(),
        }
    }

    /// Detect if a conflict exists between local and remote
    pub fn detect_conflict(
        &mut self,
        file_path: &str,
        local_modified: i64,
        remote_modified: i64,
        local_etag: Option<&str>,
        remote_etag: Option<&str>,
    ) -> Option<&SyncConflict> {
        // No conflict if etags match
        if let (Some(le), Some(re)) = (local_etag, remote_etag) {
            if le == re {
                return None;
            }
        }

        // Conflict if both modified since last sync
        if local_modified != remote_modified {
            let conflict = SyncConflict {
                id: uuid::Uuid::new_v4().to_string(),
                file_path: file_path.to_string(),
                local_modified,
                remote_modified,
                local_size: 0,
                remote_size: 0,
                local_etag: local_etag.map(String::from),
                remote_etag: remote_etag.map(String::from),
                resolved: false,
                resolution: None,
            };
            self.conflicts.push(conflict);
            return self.conflicts.last();
        }

        None
    }

    /// Auto-resolve all pending conflicts with the default strategy
    pub fn resolve_all(&mut self) -> Vec<(String, ConflictAction)> {
        let strategy = self.default_strategy;
        self.conflicts
            .iter_mut()
            .filter(|c| !c.resolved)
            .map(|c| {
                let action = c.auto_resolve(strategy);
                (c.file_path.clone(), action)
            })
            .collect()
    }

    /// Resolve a specific conflict
    pub fn resolve_one(&mut self, conflict_id: &str, strategy: ConflictStrategy) -> Option<ConflictAction> {
        self.conflicts
            .iter_mut()
            .find(|c| c.id == conflict_id)
            .map(|c| c.auto_resolve(strategy))
    }

    /// Get all unresolved conflicts
    pub fn unresolved(&self) -> Vec<&SyncConflict> {
        self.conflicts.iter().filter(|c| !c.resolved).collect()
    }

    /// Get the conflict-renamed path for a file
    pub fn conflict_path(original: &Path, suffix: &str) -> PathBuf {
        let stem = original.file_stem().unwrap_or_default().to_string_lossy();
        let ext = original.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        original.with_file_name(format!("{} ({}){}", stem, suffix, ext))
    }

    /// Clear resolved conflicts
    pub fn clear_resolved(&mut self) {
        self.conflicts.retain(|c| !c.resolved);
    }
}

/// Generate a timestamp-like string for conflict suffixes
fn chrono_ish_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple formatting without chrono dependency
    let secs = now % 60;
    let mins = (now / 60) % 60;
    let hours = (now / 3600) % 24;
    let days = now / 86400;
    format!("{}-{:02}{:02}{:02}", days, hours, mins, secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_detection() {
        let mut resolver = ConflictResolver::new(ConflictStrategy::KeepBoth);
        let conflict = resolver.detect_conflict("notebook.snotes", 1000, 2000, None, None);
        assert!(conflict.is_some());
    }

    #[test]
    fn test_no_conflict_same_etag() {
        let mut resolver = ConflictResolver::new(ConflictStrategy::KeepBoth);
        let conflict = resolver.detect_conflict(
            "notebook.snotes", 1000, 2000, Some("abc"), Some("abc"),
        );
        assert!(conflict.is_none());
    }

    #[test]
    fn test_resolve_keep_newest() {
        let mut conflict = SyncConflict::new("test.snotes", 2000, 1000);
        let action = conflict.auto_resolve(ConflictStrategy::KeepNewest);
        assert!(matches!(action, ConflictAction::UploadLocal)); // local is newer
    }

    #[test]
    fn test_resolve_keep_both() {
        let mut conflict = SyncConflict::new("test.snotes", 1000, 2000);
        let action = conflict.auto_resolve(ConflictStrategy::KeepBoth);
        assert!(matches!(action, ConflictAction::KeepBothRename { .. }));
    }

    #[test]
    fn test_conflict_path_rename() {
        let path = Path::new("/home/user/docs/notes.snotes");
        let renamed = ConflictResolver::conflict_path(path, "conflict-12345");
        assert!(renamed.to_string_lossy().contains("conflict-12345"));
        assert!(renamed.to_string_lossy().ends_with(".snotes"));
    }

    #[test]
    fn test_resolve_all() {
        let mut resolver = ConflictResolver::new(ConflictStrategy::KeepLocal);
        resolver.detect_conflict("a.snotes", 1000, 2000, None, None);
        resolver.detect_conflict("b.snotes", 3000, 1000, None, None);

        let actions = resolver.resolve_all();
        assert_eq!(actions.len(), 2);
        assert!(resolver.unresolved().is_empty());
    }
}
