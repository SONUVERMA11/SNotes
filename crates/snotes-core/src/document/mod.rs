//! Document model: Library → Notebooks → Sections → Pages

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The root library containing all notebooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: Uuid,
    pub name: String,
    pub notebooks: Vec<Notebook>,
    pub created_at: i64,
    pub modified_at: i64,
}

impl Library {
    pub fn new(name: &str) -> Self {
        let now = timestamp();
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            notebooks: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    pub fn add_notebook(&mut self, notebook: Notebook) {
        self.notebooks.push(notebook);
        self.modified_at = timestamp();
    }
}

/// A notebook containing sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: Uuid,
    pub name: String,
    pub cover_color: String,
    pub sections: Vec<Section>,
    pub created_at: i64,
    pub modified_at: i64,
}

impl Notebook {
    pub fn new(name: &str, cover_color: &str) -> Self {
        let now = timestamp();
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            cover_color: cover_color.to_string(),
            sections: vec![Section::new("Untitled Section")],
            created_at: now,
            modified_at: now,
        }
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
        self.modified_at = timestamp();
    }
}

/// A section containing pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: Uuid,
    pub name: String,
    pub pages: Vec<Page>,
    pub created_at: i64,
    pub modified_at: i64,
}

impl Section {
    pub fn new(name: &str) -> Self {
        let now = timestamp();
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            pages: vec![Page::new(PageTemplate::Blank)],
            created_at: now,
            modified_at: now,
        }
    }

    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
        self.modified_at = timestamp();
    }

    /// Reorder a page from one index to another
    pub fn reorder_page(&mut self, from: usize, to: usize) {
        if from < self.pages.len() && to < self.pages.len() {
            let page = self.pages.remove(from);
            self.pages.insert(to, page);
            self.modified_at = timestamp();
        }
    }
}

/// A single page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub template: PageTemplate,
    pub width: f64,
    pub height: f64,
    pub layer_ids: Vec<Uuid>,
    pub background_pdf_page: Option<u32>,
    pub background_image: Option<String>,
    pub created_at: i64,
    pub modified_at: i64,
}

impl Page {
    pub fn new(template: PageTemplate) -> Self {
        let now = timestamp();
        // A4 dimensions at 144 DPI
        Self {
            id: Uuid::new_v4(),
            template,
            width: 1191.0,
            height: 1684.0,
            layer_ids: Vec::new(),
            background_pdf_page: None,
            background_image: None,
            created_at: now,
            modified_at: now,
        }
    }

    pub fn new_with_size(template: PageTemplate, width: f64, height: f64) -> Self {
        let now = timestamp();
        Self {
            id: Uuid::new_v4(),
            template,
            width,
            height,
            layer_ids: Vec::new(),
            background_pdf_page: None,
            background_image: None,
            created_at: now,
            modified_at: now,
        }
    }
}

/// Page background template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageTemplate {
    Blank,
    Lined,
    Grid,
    Dotted,
    Isometric,
    MusicStaff,
    Cornell,
}

impl PageTemplate {
    pub fn display_name(&self) -> &'static str {
        match self {
            PageTemplate::Blank => "Blank",
            PageTemplate::Lined => "Lined",
            PageTemplate::Grid => "Grid",
            PageTemplate::Dotted => "Dotted",
            PageTemplate::Isometric => "Isometric",
            PageTemplate::MusicStaff => "Music Staff",
            PageTemplate::Cornell => "Cornell Notes",
        }
    }

    pub fn all() -> &'static [PageTemplate] {
        &[
            PageTemplate::Blank,
            PageTemplate::Lined,
            PageTemplate::Grid,
            PageTemplate::Dotted,
            PageTemplate::Isometric,
            PageTemplate::MusicStaff,
            PageTemplate::Cornell,
        ]
    }
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_hierarchy() {
        let mut lib = Library::new("My Library");
        let mut nb = Notebook::new("Physics", "#3498db");
        let mut section = Section::new("Chapter 1");
        section.add_page(Page::new(PageTemplate::Lined));
        nb.add_section(section);
        lib.add_notebook(nb);

        assert_eq!(lib.notebooks.len(), 1);
        assert_eq!(lib.notebooks[0].sections.len(), 2); // default + added
        assert_eq!(lib.notebooks[0].sections[1].pages.len(), 2); // default + added
    }
}
