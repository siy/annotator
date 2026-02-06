use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Navigation
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    PageUp,
    PageDown,
    Home,
    End,

    // Selection
    SelectUp,
    SelectDown,
    SelectLeft,
    SelectRight,

    // Annotations
    CreateAnnotation,
    EditAnnotation,
    DeleteAnnotation,

    // File management
    MarkClean,
    NextUnreviewed,
    OpenFileList,
    OpenTreeView,

    // Undo/Redo
    Undo,
    Redo,

    // App
    Quit,

    // Popup actions
    Confirm,
    Cancel,
    InputChar(char),
    InputBackspace,
    InputDelete,
    InputNewline,
}

pub fn map_key_viewing(key: KeyEvent) -> Option<Action> {
    // Check Ctrl combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('e') => Some(Action::EditAnnotation),
            KeyCode::Char('d') => Some(Action::DeleteAnnotation),
            KeyCode::Char('z') => Some(Action::Undo),
            KeyCode::Char('y') => Some(Action::Redo),
            KeyCode::Char('m') => Some(Action::MarkClean),
            KeyCode::Char('n') => Some(Action::NextUnreviewed),
            KeyCode::Char('f') => Some(Action::OpenFileList),
            KeyCode::Char('t') => Some(Action::OpenTreeView),
            _ => None,
        };
    }

    // Check Shift+arrow for selection
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        return match key.code {
            KeyCode::Up => Some(Action::SelectUp),
            KeyCode::Down => Some(Action::SelectDown),
            KeyCode::Left => Some(Action::SelectLeft),
            KeyCode::Right => Some(Action::SelectRight),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Up => Some(Action::CursorUp),
        KeyCode::Down => Some(Action::CursorDown),
        KeyCode::Left => Some(Action::CursorLeft),
        KeyCode::Right => Some(Action::CursorRight),
        KeyCode::PageUp => Some(Action::PageUp),
        KeyCode::PageDown => Some(Action::PageDown),
        KeyCode::Home => Some(Action::Home),
        KeyCode::End => Some(Action::End),
        KeyCode::Enter => Some(Action::CreateAnnotation),
        _ => None,
    }
}

pub fn map_key_input(key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('q') => Some(Action::Cancel),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Char(c) => Some(Action::InputChar(c)),
        KeyCode::Backspace => Some(Action::InputBackspace),
        KeyCode::Delete => Some(Action::InputDelete),
        _ => None,
    }
}

pub fn map_key_file_list(key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('q') | KeyCode::Char('f') => Some(Action::Cancel),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Up => Some(Action::CursorUp),
        KeyCode::Down => Some(Action::CursorDown),
        KeyCode::Char(c) => Some(Action::InputChar(c)),
        KeyCode::Backspace => Some(Action::InputBackspace),
        _ => None,
    }
}

pub fn map_key_tree(key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('q') | KeyCode::Char('t') => Some(Action::Cancel),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Up => Some(Action::CursorUp),
        KeyCode::Down => Some(Action::CursorDown),
        _ => None,
    }
}

pub fn map_key_conflict(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Up => Some(Action::CursorUp),
        KeyCode::Down => Some(Action::CursorDown),
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Esc => Some(Action::Cancel),
        _ => None,
    }
}
