//! Storage: SQLite metadata + LZ4-compressed FlatBuffers for strokes

mod database;
pub mod autosave;
pub mod search;

pub use database::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compression error: {0}")]
    Compression(String),
    #[cfg(feature = "storage")]
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Compress stroke data with LZ4
pub fn compress_strokes(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

/// Decompress LZ4 stroke data
pub fn decompress_strokes(data: &[u8]) -> Result<Vec<u8>, StorageError> {
    lz4_flex::decompress_size_prepended(data)
        .map_err(|e| StorageError::Compression(e.to_string()))
}

/// SQL schema for the metadata database
pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS libraries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS notebooks (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL,
    name TEXT NOT NULL,
    cover_color TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    FOREIGN KEY (library_id) REFERENCES libraries(id)
);

CREATE TABLE IF NOT EXISTS sections (
    id TEXT PRIMARY KEY,
    notebook_id TEXT NOT NULL,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id)
);

CREATE TABLE IF NOT EXISTS pages (
    id TEXT PRIMARY KEY,
    section_id TEXT NOT NULL,
    template TEXT NOT NULL DEFAULT 'blank',
    width REAL NOT NULL,
    height REAL NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    background_pdf_page INTEGER,
    background_image TEXT,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    FOREIGN KEY (section_id) REFERENCES sections(id)
);

CREATE TABLE IF NOT EXISTS layers (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL,
    name TEXT NOT NULL,
    visible INTEGER NOT NULL DEFAULT 1,
    locked INTEGER NOT NULL DEFAULT 0,
    opacity REAL NOT NULL DEFAULT 1.0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (page_id) REFERENCES pages(id)
);

CREATE TABLE IF NOT EXISTS stroke_data (
    page_id TEXT PRIMARY KEY,
    data BLOB NOT NULL,
    compressed INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (page_id) REFERENCES pages(id)
);

CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    hash TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL,
    data BLOB NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_notebooks_library ON notebooks(library_id);
CREATE INDEX IF NOT EXISTS idx_sections_notebook ON sections(notebook_id);
CREATE INDEX IF NOT EXISTS idx_pages_section ON pages(section_id);
CREATE INDEX IF NOT EXISTS idx_layers_page ON layers(page_id);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_roundtrip() {
        let original = b"Hello, S Notes! This is stroke data that gets compressed.";
        let compressed = compress_strokes(original);
        assert!(compressed.len() > 0);
        let decompressed = decompress_strokes(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_lz4_large_data() {
        let original: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let compressed = compress_strokes(&original);
        assert!(compressed.len() < original.len()); // Should compress
        let decompressed = decompress_strokes(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }
}
