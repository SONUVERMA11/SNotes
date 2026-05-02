//! Plugin host — loads, sandboxes, and manages WASM plugins via Wasmtime
//!
//! Plugins run in a WASM sandbox with a capability-based API.
//! They can: read strokes, add toolbar buttons, register export formats,
//! and listen to document events. They CANNOT: access filesystem directly,
//! make network calls, or modify other plugins' state.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin metadata (from plugin manifest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Requested capabilities
    pub capabilities: Vec<PluginCapability>,
    /// WASM module filename
    pub module: String,
    /// Icon path (relative to plugin dir)
    pub icon: Option<String>,
    /// Minimum S Notes version
    pub min_app_version: Option<String>,
}

/// Capabilities a plugin can request
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    /// Read stroke data on the current page
    ReadStrokes,
    /// Modify strokes (add/remove)
    WriteStrokes,
    /// Add items to the toolbar
    ToolbarExtension,
    /// Register a custom export format
    ExportFormat,
    /// Listen to document events (page change, stroke add/remove)
    DocumentEvents,
    /// Access settings storage (key-value, sandboxed per-plugin)
    Settings,
    /// Custom rendering on the canvas overlay
    CanvasOverlay,
}

/// State of a loaded plugin
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Loading,
    Active,
    Paused,
    Error(String),
    Unloaded,
}

/// Info about a loaded plugin
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub path: PathBuf,
    pub granted_capabilities: Vec<PluginCapability>,
    /// Plugin's sandboxed settings
    pub settings: HashMap<String, String>,
}

/// Manages WASM plugin lifecycle
pub struct PluginHost {
    plugins: Vec<LoadedPlugin>,
    plugin_dir: PathBuf,
    /// Global event listeners registered by plugins
    event_listeners: HashMap<String, Vec<usize>>, // event_name -> plugin indices
}

impl PluginHost {
    pub fn new(plugin_dir: &Path) -> Self {
        Self {
            plugins: Vec::new(),
            plugin_dir: plugin_dir.to_path_buf(),
            event_listeners: HashMap::new(),
        }
    }

    /// Scan the plugin directory and load all valid plugins
    pub fn discover_plugins(&mut self) -> Result<usize> {
        let mut count = 0;

        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)
                .context("Failed to create plugin directory")?;
            return Ok(0);
        }

        let entries = std::fs::read_dir(&self.plugin_dir)
            .context("Failed to read plugin directory")?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    match self.load_plugin_from_dir(&path) {
                        Ok(_) => count += 1,
                        Err(e) => log::warn!("Failed to load plugin at {:?}: {}", path, e),
                    }
                }
            }
        }

        log::info!("Discovered {} plugins", count);
        Ok(count)
    }

    /// Load a plugin from its directory
    pub fn load_plugin_from_dir(&mut self, path: &Path) -> Result<()> {
        let manifest_path = path.join("manifest.json");
        let manifest_data = std::fs::read_to_string(&manifest_path)
            .context("Failed to read manifest.json")?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_data)
            .context("Failed to parse manifest.json")?;

        let wasm_path = path.join(&manifest.module);
        if !wasm_path.exists() {
            anyhow::bail!("WASM module not found: {:?}", wasm_path);
        }

        log::info!("Loading plugin: {} v{}", manifest.name, manifest.version);

        // In production: instantiate Wasmtime engine, compile module,
        // create sandbox with only the requested capabilities
        let plugin = LoadedPlugin {
            manifest,
            state: PluginState::Active,
            path: path.to_path_buf(),
            granted_capabilities: Vec::new(), // User must approve
            settings: HashMap::new(),
        };

        self.plugins.push(plugin);
        Ok(())
    }

    /// Load a single WASM file directly (for development)
    pub fn load_wasm(&mut self, name: &str, wasm_bytes: &[u8]) -> Result<()> {
        log::info!("Loading WASM plugin: {} ({} bytes)", name, wasm_bytes.len());

        // Validate WASM magic number
        if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != b"\0asm" {
            anyhow::bail!("Invalid WASM module: bad magic number");
        }

        let plugin = LoadedPlugin {
            manifest: PluginManifest {
                name: name.to_string(),
                version: "dev".to_string(),
                author: "Developer".to_string(),
                description: "Development plugin".to_string(),
                capabilities: Vec::new(),
                module: format!("{}.wasm", name),
                icon: None,
                min_app_version: None,
            },
            state: PluginState::Active,
            path: self.plugin_dir.join(name),
            granted_capabilities: Vec::new(),
            settings: HashMap::new(),
        };

        self.plugins.push(plugin);
        Ok(())
    }

    /// Grant a capability to a plugin
    pub fn grant_capability(&mut self, plugin_name: &str, cap: PluginCapability) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.name == plugin_name) {
            if !plugin.granted_capabilities.contains(&cap) {
                plugin.granted_capabilities.push(cap);
            }
            true
        } else {
            false
        }
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> &[LoadedPlugin] {
        &self.plugins
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&LoadedPlugin> {
        self.plugins.iter().find(|p| p.manifest.name == name)
    }

    /// Pause a plugin
    pub fn pause_plugin(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.name == name) {
            plugin.state = PluginState::Paused;
            true
        } else {
            false
        }
    }

    /// Resume a paused plugin
    pub fn resume_plugin(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.name == name) {
            if plugin.state == PluginState::Paused {
                plugin.state = PluginState::Active;
                return true;
            }
        }
        false
    }

    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, name: &str) -> bool {
        let before = self.plugins.len();
        self.plugins.retain(|p| p.manifest.name != name);
        self.plugins.len() < before
    }

    /// Set a sandboxed setting for a plugin
    pub fn set_plugin_setting(&mut self, plugin_name: &str, key: &str, value: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.name == plugin_name) {
            plugin.settings.insert(key.to_string(), value.to_string());
            true
        } else {
            false
        }
    }

    /// Get a sandboxed setting for a plugin
    pub fn get_plugin_setting(&self, plugin_name: &str, key: &str) -> Option<&str> {
        self.plugins.iter()
            .find(|p| p.manifest.name == plugin_name)
            .and_then(|p| p.settings.get(key).map(|s| s.as_str()))
    }

    /// Broadcast an event to all listening plugins
    pub fn broadcast_event(&self, event: &str, _data: &str) {
        if let Some(listeners) = self.event_listeners.get(event) {
            for &idx in listeners {
                if idx < self.plugins.len() && self.plugins[idx].state == PluginState::Active {
                    // In production: call the plugin's event handler via WASM
                    log::debug!("Event '{}' -> plugin '{}'", event, self.plugins[idx].manifest.name);
                }
            }
        }
    }

    /// Register a plugin as a listener for an event
    pub fn register_listener(&mut self, plugin_name: &str, event: &str) -> bool {
        if let Some(idx) = self.plugins.iter().position(|p| p.manifest.name == plugin_name) {
            self.event_listeners
                .entry(event.to_string())
                .or_default()
                .push(idx);
            true
        } else {
            false
        }
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new(Path::new("plugins"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_plugin_host_creation() {
        let host = PluginHost::new(Path::new("/tmp/snotes_test_plugins"));
        assert!(host.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_lifecycle() {
        let dir = PathBuf::from("/tmp/snotes_test_plugins_lc");
        let _ = fs::create_dir_all(&dir);

        let mut host = PluginHost::new(&dir);

        // Create a test plugin directory
        let plugin_dir = dir.join("test-plugin");
        let _ = fs::create_dir_all(&plugin_dir);
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: "A test plugin".to_string(),
            capabilities: vec![PluginCapability::ReadStrokes],
            module: "plugin.wasm".to_string(),
            icon: None,
            min_app_version: None,
        };
        fs::write(plugin_dir.join("manifest.json"), serde_json::to_string(&manifest).unwrap()).unwrap();
        // Create a minimal valid WASM file (magic + version)
        fs::write(plugin_dir.join("plugin.wasm"), b"\0asm\x01\x00\x00\x00").unwrap();

        host.load_plugin_from_dir(&plugin_dir).unwrap();
        assert_eq!(host.list_plugins().len(), 1);
        assert_eq!(host.list_plugins()[0].manifest.name, "test-plugin");

        // Grant capability
        assert!(host.grant_capability("test-plugin", PluginCapability::ReadStrokes));

        // Pause/resume
        assert!(host.pause_plugin("test-plugin"));
        assert_eq!(host.list_plugins()[0].state, PluginState::Paused);
        assert!(host.resume_plugin("test-plugin"));
        assert_eq!(host.list_plugins()[0].state, PluginState::Active);

        // Settings
        assert!(host.set_plugin_setting("test-plugin", "color", "#ff0000"));
        assert_eq!(host.get_plugin_setting("test-plugin", "color"), Some("#ff0000"));

        // Unload
        assert!(host.unload_plugin("test-plugin"));
        assert!(host.list_plugins().is_empty());

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_wasm_validation() {
        let mut host = PluginHost::default();
        // Invalid WASM
        assert!(host.load_wasm("bad", b"not wasm").is_err());
        // Valid WASM magic
        assert!(host.load_wasm("good", b"\0asm\x01\x00\x00\x00").is_ok());
    }
}
