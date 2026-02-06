use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub current_file: Option<String>,
    pub current_line: u32,
    pub current_col: u32,
    pub scroll_offset: u32,
    pub last_adjust_commit: Option<String>,
}

impl Session {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let data = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_default() {
        let s = Session::default();
        assert!(s.current_file.is_none());
        assert_eq!(s.current_line, 0);
        assert_eq!(s.current_col, 0);
        assert_eq!(s.scroll_offset, 0);
        assert!(s.last_adjust_commit.is_none());
    }

    #[test]
    fn test_session_save_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("session.json");

        let s = Session {
            current_file: Some("src/main.rs".into()),
            current_line: 42,
            current_col: 8,
            scroll_offset: 30,
            last_adjust_commit: Some("abc123".into()),
        };
        s.save(&path).unwrap();

        let loaded = Session::load(&path).unwrap();
        assert_eq!(loaded.current_file.as_deref(), Some("src/main.rs"));
        assert_eq!(loaded.current_line, 42);
        assert_eq!(loaded.current_col, 8);
        assert_eq!(loaded.scroll_offset, 30);
        assert_eq!(loaded.last_adjust_commit.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_session_load_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");
        let s = Session::load(&path).unwrap();
        assert!(s.current_file.is_none());
    }
}
