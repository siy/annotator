use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub fn poll_event(timeout: Duration) -> anyhow::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn is_ctrl(key: &KeyEvent, c: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(c)
}

pub fn is_shift_arrow(key: &KeyEvent) -> Option<KeyCode> {
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => Some(key.code),
            _ => None,
        }
    } else {
        None
    }
}
