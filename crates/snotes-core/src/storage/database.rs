//! SQLite database CRUD operations for document metadata

use super::{StorageError, SCHEMA_SQL};
use crate::document::{Library, Notebook, Section, Page, PageTemplate};
use uuid::Uuid;

/// Database manager wrapping SQLite
pub struct Database {
    #[cfg(feature = "storage")]
    conn: Option<rusqlite::Connection>,
    #[cfg(not(feature = "storage"))]
    _phantom: (),
    pub path: String,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open(path: &str) -> Result<Self, StorageError> {
        #[cfg(feature = "storage")]
        {
            let conn = rusqlite::Connection::open(path)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
                .map_err(|e| StorageError::Database(e.to_string()))?;
            conn.execute_batch(SCHEMA_SQL)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            log::info!("Database opened at {}", path);
            Ok(Self { conn: Some(conn), path: path.to_string() })
        }
        #[cfg(not(feature = "storage"))]
        {
            log::warn!("Storage feature disabled, database operations are no-ops");
            Ok(Self { _phantom: (), path: path.to_string() })
        }
    }

    /// Open an in-memory database (for testing)
    pub fn open_memory() -> Result<Self, StorageError> {
        #[cfg(feature = "storage")]
        {
            let conn = rusqlite::Connection::open_in_memory()
                .map_err(|e| StorageError::Database(e.to_string()))?;
            conn.execute_batch("PRAGMA foreign_keys=ON;")
                .map_err(|e| StorageError::Database(e.to_string()))?;
            conn.execute_batch(SCHEMA_SQL)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            Ok(Self { conn: Some(conn), path: ":memory:".to_string() })
        }
        #[cfg(not(feature = "storage"))]
        {
            Ok(Self { _phantom: (), path: ":memory:".to_string() })
        }
    }

    // ─── Library CRUD ─────────────────────────────────────────────

    #[cfg(feature = "storage")]
    pub fn create_library(&self, library: &Library) -> Result<(), StorageError> {
        let conn = self.conn.as_ref().unwrap();
        conn.execute(
            "INSERT INTO libraries (id, name, created_at, modified_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![library.id.to_string(), library.name, library.created_at, library.modified_at],
        )?;
        Ok(())
    }

    #[cfg(feature = "storage")]
    pub fn get_library(&self, id: &Uuid) -> Result<Option<Library>, StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, modified_at FROM libraries WHERE id = ?1"
        )?;
        let mut rows = stmt.query(rusqlite::params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            Ok(Some(Library {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                name: row.get(1)?,
                notebooks: Vec::new(),
                created_at: row.get(2)?,
                modified_at: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "storage")]
    pub fn list_libraries(&self) -> Result<Vec<Library>, StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, modified_at FROM libraries ORDER BY name"
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            Ok(Library {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                name: row.get(1)?,
                notebooks: Vec::new(),
                created_at: row.get(2)?,
                modified_at: row.get(3)?,
            })
        })?;
        let mut libs = Vec::new();
        for row in rows {
            libs.push(row?);
        }
        Ok(libs)
    }

    // ─── Notebook CRUD ────────────────────────────────────────────

    #[cfg(feature = "storage")]
    pub fn create_notebook(&self, library_id: &Uuid, notebook: &Notebook) -> Result<(), StorageError> {
        let conn = self.conn.as_ref().unwrap();
        conn.execute(
            "INSERT INTO notebooks (id, library_id, name, cover_color, sort_order, created_at, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                notebook.id.to_string(), library_id.to_string(), notebook.name,
                notebook.cover_color, 0, notebook.created_at, notebook.modified_at
            ],
        )?;
        Ok(())
    }

    #[cfg(feature = "storage")]
    pub fn list_notebooks(&self, library_id: &Uuid) -> Result<Vec<Notebook>, StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, cover_color, created_at, modified_at FROM notebooks WHERE library_id = ?1 ORDER BY sort_order"
        )?;
        let rows = stmt.query_map(rusqlite::params![library_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            Ok(Notebook {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                name: row.get(1)?,
                cover_color: row.get(2)?,
                sections: Vec::new(),
                created_at: row.get(3)?,
                modified_at: row.get(4)?,
            })
        })?;
        let mut nbs = Vec::new();
        for row in rows { nbs.push(row?); }
        Ok(nbs)
    }

    // ─── Section CRUD ─────────────────────────────────────────────

    #[cfg(feature = "storage")]
    pub fn create_section(&self, notebook_id: &Uuid, section: &Section) -> Result<(), StorageError> {
        let conn = self.conn.as_ref().unwrap();
        conn.execute(
            "INSERT INTO sections (id, notebook_id, name, sort_order, created_at, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                section.id.to_string(), notebook_id.to_string(), section.name,
                0, section.created_at, section.modified_at
            ],
        )?;
        Ok(())
    }

    // ─── Page CRUD ────────────────────────────────────────────────

    #[cfg(feature = "storage")]
    pub fn create_page(&self, section_id: &Uuid, page: &Page) -> Result<(), StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let template_str = format!("{:?}", page.template).to_lowercase();
        conn.execute(
            "INSERT INTO pages (id, section_id, template, width, height, sort_order, background_pdf_page, background_image, created_at, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                page.id.to_string(), section_id.to_string(), template_str,
                page.width, page.height, 0,
                page.background_pdf_page, page.background_image,
                page.created_at, page.modified_at
            ],
        )?;
        Ok(())
    }

    #[cfg(feature = "storage")]
    pub fn list_pages(&self, section_id: &Uuid) -> Result<Vec<Page>, StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, template, width, height, background_pdf_page, background_image, created_at, modified_at FROM pages WHERE section_id = ?1 ORDER BY sort_order"
        )?;
        let rows = stmt.query_map(rusqlite::params![section_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let template_str: String = row.get(1)?;
            let template = match template_str.as_str() {
                "lined" => PageTemplate::Lined,
                "grid" => PageTemplate::Grid,
                "dotted" => PageTemplate::Dotted,
                "isometric" => PageTemplate::Isometric,
                "musicstaff" => PageTemplate::MusicStaff,
                "cornell" => PageTemplate::Cornell,
                _ => PageTemplate::Blank,
            };
            Ok(Page {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                template,
                width: row.get(2)?,
                height: row.get(3)?,
                layer_ids: Vec::new(),
                background_pdf_page: row.get(4)?,
                background_image: row.get(5)?,
                created_at: row.get(6)?,
                modified_at: row.get(7)?,
            })
        })?;
        let mut pages = Vec::new();
        for row in rows { pages.push(row?); }
        Ok(pages)
    }

    // ─── Stroke Data ──────────────────────────────────────────────

    #[cfg(feature = "storage")]
    pub fn save_stroke_data(&self, page_id: &Uuid, data: &[u8]) -> Result<(), StorageError> {
        let compressed = super::compress_strokes(data);
        let conn = self.conn.as_ref().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO stroke_data (page_id, data, compressed) VALUES (?1, ?2, 1)",
            rusqlite::params![page_id.to_string(), compressed],
        )?;
        Ok(())
    }

    #[cfg(feature = "storage")]
    pub fn load_stroke_data(&self, page_id: &Uuid) -> Result<Option<Vec<u8>>, StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT data, compressed FROM stroke_data WHERE page_id = ?1"
        )?;
        let mut rows = stmt.query(rusqlite::params![page_id.to_string()])?;
        if let Some(row) = rows.next()? {
            let data: Vec<u8> = row.get(0)?;
            let compressed: bool = row.get(1)?;
            if compressed {
                let decompressed = super::decompress_strokes(&data)?;
                Ok(Some(decompressed))
            } else {
                Ok(Some(data))
            }
        } else {
            Ok(None)
        }
    }

    // ─── Settings ─────────────────────────────────────────────────

    #[cfg(feature = "storage")]
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let conn = self.conn.as_ref().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    #[cfg(feature = "storage")]
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(rusqlite::params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    // ─── Delete operations ────────────────────────────────────────

    #[cfg(feature = "storage")]
    pub fn delete_notebook(&self, id: &Uuid) -> Result<(), StorageError> {
        let conn = self.conn.as_ref().unwrap();
        // Cascade delete sections, pages, layers, stroke_data
        let sections = self.list_sections(id)?;
        for section in &sections {
            self.delete_section(&section.id)?;
        }
        conn.execute("DELETE FROM notebooks WHERE id = ?1", rusqlite::params![id.to_string()])?;
        Ok(())
    }

    #[cfg(feature = "storage")]
    pub fn delete_section(&self, id: &Uuid) -> Result<(), StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let pages = self.list_pages(id)?;
        for page in &pages {
            conn.execute("DELETE FROM stroke_data WHERE page_id = ?1", rusqlite::params![page.id.to_string()])?;
            conn.execute("DELETE FROM layers WHERE page_id = ?1", rusqlite::params![page.id.to_string()])?;
        }
        conn.execute("DELETE FROM pages WHERE section_id = ?1", rusqlite::params![id.to_string()])?;
        conn.execute("DELETE FROM sections WHERE id = ?1", rusqlite::params![id.to_string()])?;
        Ok(())
    }

    #[cfg(feature = "storage")]
    fn list_sections(&self, notebook_id: &Uuid) -> Result<Vec<Section>, StorageError> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, modified_at FROM sections WHERE notebook_id = ?1 ORDER BY sort_order"
        )?;
        let rows = stmt.query_map(rusqlite::params![notebook_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            Ok(Section {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                name: row.get(1)?,
                pages: Vec::new(),
                created_at: row.get(2)?,
                modified_at: row.get(3)?,
            })
        })?;
        let mut secs = Vec::new();
        for row in rows { secs.push(row?); }
        Ok(secs)
    }
}

#[cfg(all(test, feature = "storage"))]
mod tests {
    use super::*;
    use crate::document::*;

    #[test]
    fn test_library_crud() {
        let db = Database::open_memory().unwrap();
        let lib = Library::new("Test Library");
        db.create_library(&lib).unwrap();

        let fetched = db.get_library(&lib.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Test Library");

        let all = db.list_libraries().unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_notebook_crud() {
        let db = Database::open_memory().unwrap();
        let lib = Library::new("Lib");
        db.create_library(&lib).unwrap();

        let nb = Notebook::new("Physics", "#3498db");
        db.create_notebook(&lib.id, &nb).unwrap();

        let nbs = db.list_notebooks(&lib.id).unwrap();
        assert_eq!(nbs.len(), 1);
        assert_eq!(nbs[0].name, "Physics");
    }

    #[test]
    fn test_stroke_data_roundtrip() {
        let db = Database::open_memory().unwrap();
        let lib = Library::new("Lib");
        db.create_library(&lib).unwrap();
        let nb = Notebook::new("NB", "#000");
        db.create_notebook(&lib.id, &nb).unwrap();
        let sec = Section::new("S1");
        db.create_section(&nb.id, &sec).unwrap();
        let page = Page::new(PageTemplate::Blank);
        db.create_page(&sec.id, &page).unwrap();

        let stroke_data = b"fake stroke binary data for testing 12345678";
        db.save_stroke_data(&page.id, stroke_data).unwrap();

        let loaded = db.load_stroke_data(&page.id).unwrap().unwrap();
        assert_eq!(loaded, stroke_data);
    }

    #[test]
    fn test_settings() {
        let db = Database::open_memory().unwrap();
        db.set_setting("theme", "dark").unwrap();
        let val = db.get_setting("theme").unwrap().unwrap();
        assert_eq!(val, "dark");
    }
}
