//! Keyboard shortcuts system — configurable keybindings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A keyboard shortcut (e.g. Ctrl+Z)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Shortcut {
    pub fn new(key: &str) -> Self {
        Self { key: key.to_string(), ctrl: false, shift: false, alt: false, super_key: false }
    }

    pub fn ctrl(key: &str) -> Self {
        Self { key: key.to_string(), ctrl: true, shift: false, alt: false, super_key: false }
    }

    pub fn ctrl_shift(key: &str) -> Self {
        Self { key: key.to_string(), ctrl: true, shift: true, alt: false, super_key: false }
    }

    /// Format as GTK accelerator string (e.g. "<Control><Shift>z")
    pub fn to_gtk_accel(&self) -> String {
        let mut accel = String::new();
        if self.ctrl { accel.push_str("<Control>"); }
        if self.shift { accel.push_str("<Shift>"); }
        if self.alt { accel.push_str("<Alt>"); }
        if self.super_key { accel.push_str("<Super>"); }
        accel.push_str(&self.key);
        accel
    }

    /// Format as human-readable string (e.g. "Ctrl+Shift+Z")
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl { parts.push("Ctrl"); }
        if self.shift { parts.push("Shift"); }
        if self.alt { parts.push("Alt"); }
        if self.super_key { parts.push("Super"); }
        parts.push(&self.key);
        parts.join("+")
    }
}

/// Actions that can be bound to shortcuts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // File operations
    NewNotebook,
    NewPage,
    Save,
    SaveAs,
    ImportPdf,
    Export,
    // Edit operations
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    DeselectAll,
    // Tool selection
    ToolPen,
    ToolBrush,
    ToolPencil,
    ToolHighlighter,
    ToolEraser,
    ToolSelect,
    ToolShape,
    ToolText,
    ToolRuler,
    // View
    ZoomIn,
    ZoomOut,
    ZoomFit,
    ZoomReset,
    ToggleGrid,
    ToggleRulers,
    ToggleSidebar,
    Fullscreen,
    // Navigation
    NextPage,
    PrevPage,
    FirstPage,
    LastPage,
    // Misc
    Preferences,
    ShowShortcuts,
    Quit,
}

/// Shortcut manager — maps shortcuts to actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutManager {
    bindings: HashMap<Action, Shortcut>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self { bindings: Self::defaults() }
    }

    /// Get the default keybinding map
    fn defaults() -> HashMap<Action, Shortcut> {
        let mut m = HashMap::new();
        // File
        m.insert(Action::NewNotebook, Shortcut::ctrl("n"));
        m.insert(Action::NewPage, Shortcut::ctrl_shift("n"));
        m.insert(Action::Save, Shortcut::ctrl("s"));
        m.insert(Action::SaveAs, Shortcut::ctrl_shift("s"));
        m.insert(Action::ImportPdf, Shortcut::ctrl("i"));
        m.insert(Action::Export, Shortcut::ctrl_shift("e"));
        // Edit
        m.insert(Action::Undo, Shortcut::ctrl("z"));
        m.insert(Action::Redo, Shortcut::ctrl_shift("z"));
        m.insert(Action::Cut, Shortcut::ctrl("x"));
        m.insert(Action::Copy, Shortcut::ctrl("c"));
        m.insert(Action::Paste, Shortcut::ctrl("v"));
        m.insert(Action::Delete, Shortcut::new("Delete"));
        m.insert(Action::SelectAll, Shortcut::ctrl("a"));
        m.insert(Action::DeselectAll, Shortcut::new("Escape"));
        // Tools
        m.insert(Action::ToolPen, Shortcut::new("p"));
        m.insert(Action::ToolBrush, Shortcut::new("b"));
        m.insert(Action::ToolPencil, Shortcut::new("n"));
        m.insert(Action::ToolHighlighter, Shortcut::new("h"));
        m.insert(Action::ToolEraser, Shortcut::new("e"));
        m.insert(Action::ToolSelect, Shortcut::new("s"));
        m.insert(Action::ToolShape, Shortcut::new("r"));
        m.insert(Action::ToolText, Shortcut::new("t"));
        m.insert(Action::ToolRuler, Shortcut::new("l"));
        // View
        m.insert(Action::ZoomIn, Shortcut::ctrl("equal"));
        m.insert(Action::ZoomOut, Shortcut::ctrl("minus"));
        m.insert(Action::ZoomFit, Shortcut::ctrl("0"));
        m.insert(Action::ZoomReset, Shortcut::ctrl("1"));
        m.insert(Action::ToggleGrid, Shortcut::ctrl("g"));
        m.insert(Action::ToggleSidebar, Shortcut::new("F9"));
        m.insert(Action::Fullscreen, Shortcut::new("F11"));
        // Navigation
        m.insert(Action::NextPage, Shortcut::ctrl("Page_Down"));
        m.insert(Action::PrevPage, Shortcut::ctrl("Page_Up"));
        // Misc
        m.insert(Action::Preferences, Shortcut::ctrl("comma"));
        m.insert(Action::Quit, Shortcut::ctrl("q"));
        m
    }

    /// Get the shortcut for an action
    pub fn get(&self, action: &Action) -> Option<&Shortcut> {
        self.bindings.get(action)
    }

    /// Set a custom shortcut for an action
    pub fn set(&mut self, action: Action, shortcut: Shortcut) {
        self.bindings.insert(action, shortcut);
    }

    /// Reset a specific binding to default
    pub fn reset(&mut self, action: &Action) {
        let defaults = Self::defaults();
        if let Some(default) = defaults.get(action) {
            self.bindings.insert(action.clone(), default.clone());
        }
    }

    /// Reset all bindings to defaults
    pub fn reset_all(&mut self) {
        self.bindings = Self::defaults();
    }

    /// Find which action a shortcut is bound to
    pub fn action_for(&self, shortcut: &Shortcut) -> Option<&Action> {
        self.bindings.iter()
            .find(|(_, s)| *s == shortcut)
            .map(|(a, _)| a)
    }

    /// Get all bindings as a sorted list
    pub fn all_bindings(&self) -> Vec<(&Action, &Shortcut)> {
        let mut bindings: Vec<_> = self.bindings.iter().collect();
        bindings.sort_by_key(|(a, _)| format!("{:?}", a));
        bindings
    }
}

impl Default for ShortcutManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_display() {
        let s = Shortcut::ctrl_shift("z");
        assert_eq!(s.display(), "Ctrl+Shift+z");
        assert_eq!(s.to_gtk_accel(), "<Control><Shift>z");
    }

    #[test]
    fn test_shortcut_manager() {
        let mut sm = ShortcutManager::new();
        assert!(sm.get(&Action::Undo).is_some());
        assert_eq!(sm.get(&Action::Undo).unwrap().display(), "Ctrl+z");

        // Rebind
        sm.set(Action::Undo, Shortcut::ctrl_shift("u"));
        assert_eq!(sm.get(&Action::Undo).unwrap().display(), "Ctrl+Shift+u");

        // Reset
        sm.reset(&Action::Undo);
        assert_eq!(sm.get(&Action::Undo).unwrap().display(), "Ctrl+z");
    }

    #[test]
    fn test_reverse_lookup() {
        let sm = ShortcutManager::new();
        let ctrl_z = Shortcut::ctrl("z");
        assert_eq!(sm.action_for(&ctrl_z), Some(&Action::Undo));
    }
}
