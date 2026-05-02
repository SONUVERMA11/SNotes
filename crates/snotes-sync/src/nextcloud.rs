//! Nextcloud integration — sync notebooks via Nextcloud WebDAV + Notes API

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NextcloudError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("HTTP error: {0}")]
    Http(String),
}

/// Nextcloud server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextcloudConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
    /// Remote directory for S Notes data
    pub remote_path: String,
    /// Sync interval in seconds
    pub sync_interval_secs: u64,
    /// Auto-sync on changes
    pub auto_sync: bool,
}

impl Default for NextcloudConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            username: String::new(),
            password: String::new(),
            remote_path: "/S Notes/".to_string(),
            sync_interval_secs: 300,
            auto_sync: true,
        }
    }
}

/// Nextcloud server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub version: String,
    pub webdav_url: String,
    pub max_upload_size: u64,
    pub has_notes_app: bool,
}

/// Nextcloud sync client
pub struct NextcloudClient {
    config: NextcloudConfig,
    client: reqwest::Client,
    connected: bool,
}

impl NextcloudClient {
    pub fn new(config: NextcloudConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            connected: false,
        }
    }

    /// Test the connection and fetch server capabilities
    pub async fn test_connection(&mut self) -> Result<ServerCapabilities, NextcloudError> {
        let url = format!("{}/ocs/v1.php/cloud/capabilities?format=json", self.config.server_url);

        let response = self.client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("OCS-APIRequest", "true")
            .send()
            .await
            .map_err(|e| NextcloudError::ConnectionFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(NextcloudError::AuthFailed("Invalid credentials".to_string()));
        }

        if !response.status().is_success() {
            return Err(NextcloudError::ApiError(
                format!("Server returned status: {}", response.status()),
            ));
        }

        self.connected = true;
        let webdav_url = format!("{}/remote.php/dav/files/{}", self.config.server_url, self.config.username);

        Ok(ServerCapabilities {
            version: "unknown".to_string(),
            webdav_url,
            max_upload_size: 512 * 1024 * 1024, // 512 MB default
            has_notes_app: false,
        })
    }

    /// Get the WebDAV URL for a remote path
    pub fn webdav_url(&self, path: &str) -> String {
        format!(
            "{}/remote.php/dav/files/{}/{}",
            self.config.server_url,
            self.config.username,
            path.trim_start_matches('/')
        )
    }

    /// Ensure the remote sync directory exists
    pub async fn ensure_remote_dir(&self) -> Result<(), NextcloudError> {
        let url = self.webdav_url(&self.config.remote_path);

        let response = self.client
            .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| NextcloudError::ConnectionFailed(e.to_string()))?;

        // 201 = created, 405 = already exists — both OK
        if response.status().is_success() || response.status().as_u16() == 405 {
            Ok(())
        } else {
            Err(NextcloudError::ApiError(
                format!("Failed to create remote directory: {}", response.status()),
            ))
        }
    }

    /// Upload a file to Nextcloud via WebDAV
    pub async fn upload_file(&self, remote_path: &str, data: Vec<u8>) -> Result<(), NextcloudError> {
        let url = self.webdav_url(remote_path);

        let response = self.client
            .put(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .body(data)
            .send()
            .await
            .map_err(|e| NextcloudError::ConnectionFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(NextcloudError::ApiError(
                format!("Upload failed: {}", response.status()),
            ))
        }
    }

    /// Download a file from Nextcloud via WebDAV
    pub async fn download_file(&self, remote_path: &str) -> Result<Vec<u8>, NextcloudError> {
        let url = self.webdav_url(remote_path);

        let response = self.client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| NextcloudError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(NextcloudError::ApiError(
                format!("Download failed: {}", response.status()),
            ));
        }

        response.bytes().await
            .map(|b| b.to_vec())
            .map_err(|e| NextcloudError::ApiError(e.to_string()))
    }

    /// List files in a remote directory via WebDAV PROPFIND
    pub async fn list_remote(&self, remote_path: &str) -> Result<Vec<RemoteEntry>, NextcloudError> {
        let url = self.webdav_url(remote_path);

        let propfind_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:displayname/>
    <d:getlastmodified/>
    <d:getcontentlength/>
    <d:resourcetype/>
    <d:getetag/>
  </d:prop>
</d:propfind>"#;

        let response = self.client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(propfind_xml)
            .send()
            .await
            .map_err(|e| NextcloudError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() && response.status().as_u16() != 207 {
            return Err(NextcloudError::ApiError(
                format!("PROPFIND failed: {}", response.status()),
            ));
        }

        // Parse XML response (simplified — production would use quick-xml)
        let body = response.text().await
            .map_err(|e| NextcloudError::ApiError(e.to_string()))?;

        Ok(parse_propfind_entries(&body))
    }

    pub fn is_connected(&self) -> bool { self.connected }
}

/// A file/directory entry from WebDAV PROPFIND
#[derive(Debug, Clone)]
pub struct RemoteEntry {
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// Simplified PROPFIND XML parser
fn parse_propfind_entries(xml: &str) -> Vec<RemoteEntry> {
    let mut entries = Vec::new();

    // Very simplified parsing — production would use quick-xml
    for href_start in xml.match_indices("<d:href>").map(|(i, _)| i) {
        let href_end = xml[href_start..].find("</d:href>").unwrap_or(0);
        if href_end > 0 {
            let path = &xml[href_start + 8..href_start + href_end];
            let is_dir = xml[href_start..].contains("<d:collection/>");
            entries.push(RemoteEntry {
                path: path.to_string(),
                is_directory: is_dir,
                size: 0,
                etag: None,
                last_modified: None,
            });
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = NextcloudConfig::default();
        assert_eq!(config.remote_path, "/S Notes/");
        assert_eq!(config.sync_interval_secs, 300);
        assert!(config.auto_sync);
    }

    #[test]
    fn test_webdav_url() {
        let config = NextcloudConfig {
            server_url: "https://cloud.example.com".to_string(),
            username: "alice".to_string(),
            ..Default::default()
        };
        let client = NextcloudClient::new(config);
        let url = client.webdav_url("/S Notes/notebook1.snotes");
        assert_eq!(url, "https://cloud.example.com/remote.php/dav/files/alice/S Notes/notebook1.snotes");
    }
}
