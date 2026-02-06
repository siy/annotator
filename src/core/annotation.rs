use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Annotation {
    pub id: Uuid,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Annotation {
    pub fn new(file_path: String, start_line: u32, end_line: u32, text: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            file_path,
            start_line,
            end_line,
            text,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn line_range(&self) -> std::ops::RangeInclusive<u32> {
        self.start_line..=self.end_line
    }

    pub fn contains_line(&self, line: u32) -> bool {
        self.line_range().contains(&line)
    }

    pub fn overlaps(&self, start: u32, end: u32) -> bool {
        self.start_line <= end && start <= self.end_line
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Unreviewed,
    Annotated,
    Clean,
}

impl Default for FileStatus {
    fn default() -> Self {
        Self::Unreviewed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileReviewState {
    pub file_path: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdjustResult {
    Shifted {
        old_start: u32,
        old_end: u32,
        new_start: u32,
        new_end: u32,
    },
    Conflict {
        deleted_lines: Vec<u32>,
    },
    Deleted,
    Unchanged,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_new() {
        let a = Annotation::new("src/main.rs".into(), 10, 20, "Fix this".into());
        assert_eq!(a.file_path, "src/main.rs");
        assert_eq!(a.start_line, 10);
        assert_eq!(a.end_line, 20);
        assert_eq!(a.text, "Fix this");
        assert!(a.created_at <= Utc::now());
    }

    #[test]
    fn test_contains_line() {
        let a = Annotation::new("f.rs".into(), 5, 10, "t".into());
        assert!(!a.contains_line(4));
        assert!(a.contains_line(5));
        assert!(a.contains_line(7));
        assert!(a.contains_line(10));
        assert!(!a.contains_line(11));
    }

    #[test]
    fn test_overlaps() {
        let a = Annotation::new("f.rs".into(), 5, 10, "t".into());
        assert!(!a.overlaps(1, 4));
        assert!(a.overlaps(1, 5));
        assert!(a.overlaps(5, 10));
        assert!(a.overlaps(8, 15));
        assert!(a.overlaps(10, 15));
        assert!(!a.overlaps(11, 15));
        assert!(a.overlaps(3, 20));
    }

    #[test]
    fn test_file_status_default() {
        assert_eq!(FileStatus::default(), FileStatus::Unreviewed);
    }

    #[test]
    fn test_annotation_serialization() {
        let a = Annotation::new("f.rs".into(), 1, 5, "note".into());
        let json = serde_json::to_string(&a).unwrap();
        let b: Annotation = serde_json::from_str(&json).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_file_review_state_serialization() {
        let s = FileReviewState {
            file_path: "src/lib.rs".into(),
            status: FileStatus::Clean,
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: FileReviewState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }
}
