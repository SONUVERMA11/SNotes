//! D-Bus IPC interface between snotes-gtk and snotes-sync daemon
//!
//! The sync daemon exposes a D-Bus service at `org.snotes.Sync`
//! The GTK app calls methods on this service to trigger sync operations.

use serde::{Deserialize, Serialize};

/// D-Bus well-known name for the sync daemon
pub const DBUS_NAME: &str = "org.snotes.Sync";
/// D-Bus object path
pub const DBUS_PATH: &str = "/org/snotes/Sync";
/// D-Bus interface name
pub const DBUS_INTERFACE: &str = "org.snotes.Sync1";

/// Sync status reported by the daemon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error(String),
    Offline,
}

/// Sync request from the GTK app to the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    /// Trigger a full sync
    SyncAll,
    /// Sync a specific notebook
    SyncNotebook { notebook_id: String },
    /// Get current sync status
    GetStatus,
    /// Set server configuration
    Configure {
        server_url: String,
        username: String,
        password: String,
    },
    /// Pause sync
    Pause,
    /// Resume sync
    Resume,
}

/// Sync response from the daemon to the GTK app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    /// Sync started
    Started,
    /// Sync completed
    Completed { items_synced: u32 },
    /// Current status
    Status(SyncStatus),
    /// Error occurred
    Error(String),
    /// Configuration updated
    ConfigUpdated,
}

/// D-Bus interface XML definition (for introspection)
pub const DBUS_INTROSPECTION_XML: &str = r#"
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
  <interface name="org.snotes.Sync1">
    <method name="SyncAll">
      <arg name="result" type="s" direction="out"/>
    </method>
    <method name="SyncNotebook">
      <arg name="notebook_id" type="s" direction="in"/>
      <arg name="result" type="s" direction="out"/>
    </method>
    <method name="GetStatus">
      <arg name="status" type="s" direction="out"/>
    </method>
    <method name="Configure">
      <arg name="server_url" type="s" direction="in"/>
      <arg name="username" type="s" direction="in"/>
      <arg name="password" type="s" direction="in"/>
      <arg name="result" type="s" direction="out"/>
    </method>
    <method name="Pause">
      <arg name="result" type="s" direction="out"/>
    </method>
    <method name="Resume">
      <arg name="result" type="s" direction="out"/>
    </method>
    <signal name="SyncProgress">
      <arg name="progress" type="d"/>
      <arg name="message" type="s"/>
    </signal>
    <signal name="SyncCompleted">
      <arg name="items_synced" type="u"/>
    </signal>
    <signal name="SyncError">
      <arg name="error" type="s"/>
    </signal>
  </interface>
</node>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_serialization() {
        let status = SyncStatus::Syncing;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Syncing"));
    }

    #[test]
    fn test_sync_request_serialization() {
        let req = SyncRequest::SyncNotebook { notebook_id: "abc-123".to_string() };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("abc-123"));
    }

    #[test]
    fn test_dbus_constants() {
        assert!(DBUS_NAME.starts_with("org.snotes"));
        assert!(DBUS_PATH.starts_with("/org/snotes"));
    }
}
