//! WebDAV client for sync

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Network error: {0}")]
    Network(String),
}

/// WebDAV sync client
pub struct WebDavClient {
    base_url: String,
    username: String,
    authenticated: bool,
}

impl WebDavClient {
    pub fn new(base_url: &str, username: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            username: username.to_string(),
            authenticated: false,
        }
    }

    pub async fn connect(&mut self) -> Result<(), SyncError> {
        tracing::info!("Connecting to WebDAV: {}", self.base_url);
        self.authenticated = true;
        Ok(())
    }

    pub async fn sync(&self) -> Result<(), SyncError> {
        if !self.authenticated {
            return Err(SyncError::AuthFailed);
        }
        tracing::info!("Syncing with {}", self.base_url);
        Ok(())
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    /// Keep the local version
    KeepLocal,
    /// Keep the remote version
    KeepRemote,
    /// Keep both (create a copy)
    KeepBoth,
    /// Ask the user
    AskUser,
}
