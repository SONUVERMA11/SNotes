//! Undo/redo per-stroke O(1) history stack

use crate::ink::Stroke;
use uuid::Uuid;

/// An action that can be undone/redone
#[derive(Debug, Clone)]
pub enum HistoryAction {
    AddStroke { stroke: Stroke },
    RemoveStroke { stroke: Stroke },
    MoveStrokes { stroke_ids: Vec<Uuid>, dx: f64, dy: f64 },
    ScaleStrokes { stroke_ids: Vec<Uuid>, sx: f64, sy: f64, cx: f64, cy: f64 },
    ChangeLayer { stroke_id: Uuid, from_layer: Uuid, to_layer: Uuid },
    ChangeColor { stroke_id: Uuid, from_color: crate::ink::Color, to_color: crate::ink::Color },
    Composite { actions: Vec<HistoryAction> },
}

/// O(1) undo/redo history stack
pub struct HistoryStack {
    undo_stack: Vec<HistoryAction>,
    redo_stack: Vec<HistoryAction>,
    max_size: usize,
}

impl HistoryStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_size),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Push a new action (clears redo stack)
    pub fn push(&mut self, action: HistoryAction) {
        if self.undo_stack.len() >= self.max_size {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    /// Undo the last action
    pub fn undo(&mut self) -> Option<HistoryAction> {
        let action = self.undo_stack.pop()?;
        self.redo_stack.push(action.clone());
        Some(action)
    }

    /// Redo the last undone action
    pub fn redo(&mut self) -> Option<HistoryAction> {
        let action = self.redo_stack.pop()?;
        self.undo_stack.push(action.clone());
        Some(action)
    }

    pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }
    pub fn undo_count(&self) -> usize { self.undo_stack.len() }
    pub fn redo_count(&self) -> usize { self.redo_stack.len() }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::{Stroke, ToolType, Color};

    #[test]
    fn test_undo_redo() {
        let mut history = HistoryStack::new(100);
        let stroke = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
        history.push(HistoryAction::AddStroke { stroke });
        assert!(history.can_undo());
        assert!(!history.can_redo());
        history.undo();
        assert!(!history.can_undo());
        assert!(history.can_redo());
        history.redo();
        assert!(history.can_undo());
    }

    #[test]
    fn test_redo_cleared_on_new_action() {
        let mut history = HistoryStack::new(100);
        let s1 = Stroke::new(ToolType::Pen, Color::BLACK, 2.0, Uuid::new_v4());
        let s2 = Stroke::new(ToolType::Pen, Color::RED, 2.0, Uuid::new_v4());
        history.push(HistoryAction::AddStroke { stroke: s1 });
        history.undo();
        assert!(history.can_redo());
        history.push(HistoryAction::AddStroke { stroke: s2 });
        assert!(!history.can_redo()); // redo cleared
    }
}
