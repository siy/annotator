use crate::core::annotation::Annotation;

#[derive(Debug, Clone)]
pub enum UndoAction {
    Create(Annotation),
    Delete(Annotation),
    Update {
        old: Annotation,
        new: Annotation,
    },
}

impl UndoAction {
    pub fn invert(&self) -> UndoAction {
        match self {
            UndoAction::Create(a) => UndoAction::Delete(a.clone()),
            UndoAction::Delete(a) => UndoAction::Create(a.clone()),
            UndoAction::Update { old, new } => UndoAction::Update {
                old: new.clone(),
                new: old.clone(),
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct UndoStack {
    undo: Vec<UndoAction>,
    redo: Vec<UndoAction>,
}

impl UndoStack {
    pub fn push(&mut self, action: UndoAction) {
        self.undo.push(action);
        self.redo.clear();
    }

    pub fn undo(&mut self) -> Option<UndoAction> {
        let action = self.undo.pop()?;
        let inverted = action.invert();
        self.redo.push(action);
        Some(inverted)
    }

    pub fn redo(&mut self) -> Option<UndoAction> {
        let action = self.redo.pop()?;
        let inverted = action.invert();
        self.undo.push(action);
        Some(inverted)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_annotation(text: &str) -> Annotation {
        Annotation::new("f.rs".into(), 1, 1, text.into())
    }

    #[test]
    fn test_undo_redo_create() {
        let mut stack = UndoStack::default();
        let a = make_annotation("test");

        stack.push(UndoAction::Create(a.clone()));
        assert!(stack.can_undo());
        assert!(!stack.can_redo());

        let undo = stack.undo().unwrap();
        assert!(matches!(undo, UndoAction::Delete(_)));
        assert!(!stack.can_undo());
        assert!(stack.can_redo());

        let redo = stack.redo().unwrap();
        assert!(matches!(redo, UndoAction::Delete(_)));
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_push_clears_redo() {
        let mut stack = UndoStack::default();
        stack.push(UndoAction::Create(make_annotation("a")));
        stack.push(UndoAction::Create(make_annotation("b")));
        stack.undo();
        assert!(stack.can_redo());

        stack.push(UndoAction::Create(make_annotation("c")));
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_empty_undo_redo() {
        let mut stack = UndoStack::default();
        assert!(stack.undo().is_none());
        assert!(stack.redo().is_none());
    }

    #[test]
    fn test_update_invert() {
        let old = make_annotation("old");
        let new = make_annotation("new");
        let action = UndoAction::Update {
            old: old.clone(),
            new: new.clone(),
        };
        let inverted = action.invert();
        match inverted {
            UndoAction::Update { old: inv_old, new: inv_new } => {
                assert_eq!(inv_old.text, "new");
                assert_eq!(inv_new.text, "old");
            }
            _ => panic!("expected UpdateAnnotation"),
        }
    }
}
