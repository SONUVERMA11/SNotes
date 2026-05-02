//! Search engine — full-text search across notebooks, pages, and OCR text

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub notebook_id: Uuid,
    pub notebook_name: String,
    pub section_name: String,
    pub page_index: usize,
    pub match_type: MatchType,
    pub snippet: String,
    pub score: f64,
}

/// What kind of content matched
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    /// Notebook title matched
    NotebookTitle,
    /// Section title matched
    SectionTitle,
    /// Page tag matched
    Tag,
    /// Text annotation matched
    TextAnnotation,
    /// OCR-recognized text matched
    OcrText,
}

/// Search query options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub case_sensitive: bool,
    pub search_titles: bool,
    pub search_tags: bool,
    pub search_text_annotations: bool,
    pub search_ocr: bool,
    pub max_results: usize,
    pub notebook_filter: Option<Uuid>,
}

impl SearchQuery {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            case_sensitive: false,
            search_titles: true,
            search_tags: true,
            search_text_annotations: true,
            search_ocr: true,
            max_results: 50,
            notebook_filter: None,
        }
    }
}

/// Searchable index for a notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub entries: Vec<IndexEntry>,
}

/// An indexed entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub notebook_id: Uuid,
    pub notebook_name: String,
    pub section_name: String,
    pub page_index: usize,
    pub content_type: MatchType,
    pub content: String,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add an entry to the index
    pub fn add_entry(&mut self, entry: IndexEntry) {
        self.entries.push(entry);
    }

    /// Index a notebook title
    pub fn index_notebook(&mut self, id: Uuid, name: &str) {
        self.entries.push(IndexEntry {
            notebook_id: id,
            notebook_name: name.to_string(),
            section_name: String::new(),
            page_index: 0,
            content_type: MatchType::NotebookTitle,
            content: name.to_string(),
        });
    }

    /// Index a section title
    pub fn index_section(&mut self, notebook_id: Uuid, notebook_name: &str, section_name: &str) {
        self.entries.push(IndexEntry {
            notebook_id,
            notebook_name: notebook_name.to_string(),
            section_name: section_name.to_string(),
            page_index: 0,
            content_type: MatchType::SectionTitle,
            content: section_name.to_string(),
        });
    }

    /// Index a text annotation
    pub fn index_text(
        &mut self,
        notebook_id: Uuid,
        notebook_name: &str,
        section_name: &str,
        page_index: usize,
        text: &str,
    ) {
        self.entries.push(IndexEntry {
            notebook_id,
            notebook_name: notebook_name.to_string(),
            section_name: section_name.to_string(),
            page_index,
            content_type: MatchType::TextAnnotation,
            content: text.to_string(),
        });
    }

    /// Index OCR text
    pub fn index_ocr(
        &mut self,
        notebook_id: Uuid,
        notebook_name: &str,
        section_name: &str,
        page_index: usize,
        ocr_text: &str,
    ) {
        self.entries.push(IndexEntry {
            notebook_id,
            notebook_name: notebook_name.to_string(),
            section_name: section_name.to_string(),
            page_index,
            content_type: MatchType::OcrText,
            content: ocr_text.to_string(),
        });
    }

    /// Search the index
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let query_text = if query.case_sensitive {
            query.text.clone()
        } else {
            query.text.to_lowercase()
        };

        let mut results: Vec<SearchResult> = self.entries.iter()
            .filter(|entry| {
                // Filter by notebook if specified
                if let Some(nb_id) = query.notebook_filter {
                    if entry.notebook_id != nb_id {
                        return false;
                    }
                }

                // Filter by content type
                match entry.content_type {
                    MatchType::NotebookTitle | MatchType::SectionTitle => query.search_titles,
                    MatchType::Tag => query.search_tags,
                    MatchType::TextAnnotation => query.search_text_annotations,
                    MatchType::OcrText => query.search_ocr,
                }
            })
            .filter_map(|entry| {
                let content = if query.case_sensitive {
                    entry.content.clone()
                } else {
                    entry.content.to_lowercase()
                };

                if content.contains(&query_text) {
                    let snippet = make_snippet(&entry.content, &query.text, 80);
                    let score = compute_score(&content, &query_text, &entry.content_type);

                    Some(SearchResult {
                        notebook_id: entry.notebook_id,
                        notebook_name: entry.notebook_name.clone(),
                        section_name: entry.section_name.clone(),
                        page_index: entry.page_index,
                        match_type: entry.content_type.clone(),
                        snippet,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(query.max_results);
        results
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of indexed entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for SearchIndex {
    fn default() -> Self { Self::new() }
}

/// Create a snippet around the match
fn make_snippet(content: &str, query: &str, max_len: usize) -> String {
    let lower = content.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(pos) = lower.find(&query_lower) {
        let start = pos.saturating_sub(max_len / 4);
        let end = (pos + query.len() + max_len * 3 / 4).min(content.len());

        let mut snippet = content[start..end].to_string();
        if start > 0 { snippet = format!("…{}", snippet); }
        if end < content.len() { snippet.push('…'); }
        snippet
    } else {
        content.chars().take(max_len).collect()
    }
}

/// Compute relevance score
fn compute_score(content: &str, query: &str, match_type: &MatchType) -> f64 {
    let mut score = 0.0;

    // Exact match bonus
    if content == query {
        score += 100.0;
    }

    // Starts-with bonus
    if content.starts_with(query) {
        score += 50.0;
    }

    // Length similarity bonus (shorter = more relevant)
    let len_ratio = query.len() as f64 / content.len().max(1) as f64;
    score += len_ratio * 30.0;

    // Type bonus
    score += match match_type {
        MatchType::NotebookTitle => 20.0,
        MatchType::SectionTitle => 15.0,
        MatchType::Tag => 10.0,
        MatchType::TextAnnotation => 5.0,
        MatchType::OcrText => 1.0,
    };

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_basic() {
        let mut index = SearchIndex::new();
        let nb_id = Uuid::new_v4();

        index.index_notebook(nb_id, "Physics 101");
        index.index_section(nb_id, "Physics 101", "Mechanics");
        index.index_text(nb_id, "Physics 101", "Mechanics", 0, "Newton's laws of motion");

        let query = SearchQuery::new("physics");
        let results = index.search(&query);
        assert!(!results.is_empty());
        assert_eq!(results[0].notebook_name, "Physics 101");
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut index = SearchIndex::new();
        index.index_text(Uuid::new_v4(), "NB", "S", 0, "Einstein's Theory of Relativity");

        let results = index.search(&SearchQuery::new("einstein"));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_match() {
        let mut index = SearchIndex::new();
        index.index_notebook(Uuid::new_v4(), "Math");

        let results = index.search(&SearchQuery::new("chemistry"));
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_scoring() {
        let mut index = SearchIndex::new();
        let nb_id = Uuid::new_v4();

        index.index_notebook(nb_id, "Physics");
        index.index_text(nb_id, "Physics", "Ch1", 0, "Introduction to physics");

        let results = index.search(&SearchQuery::new("physics"));
        assert!(results.len() >= 2);
        // Notebook title should score higher than body text
        assert!(results[0].score >= results[1].score);
    }

    #[test]
    fn test_notebook_filter() {
        let mut index = SearchIndex::new();
        let nb1 = Uuid::new_v4();
        let nb2 = Uuid::new_v4();

        index.index_notebook(nb1, "Math Notes");
        index.index_notebook(nb2, "Math Exercises");

        let mut query = SearchQuery::new("math");
        query.notebook_filter = Some(nb1);

        let results = index.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].notebook_id, nb1);
    }

    #[test]
    fn test_snippet() {
        let snippet = make_snippet(
            "This is a very long text about quantum physics and relativity",
            "quantum",
            40,
        );
        assert!(snippet.contains("quantum"));
    }
}
