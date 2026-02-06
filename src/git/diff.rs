use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub origin: DiffLineType,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub hunks: Vec<Hunk>,
    pub status: FileDiffStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDiffStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
}

impl Hunk {
    pub fn old_end(&self) -> u32 {
        if self.old_lines == 0 {
            self.old_start
        } else {
            self.old_start + self.old_lines - 1
        }
    }

    pub fn net_offset(&self) -> i64 {
        self.new_lines as i64 - self.old_lines as i64
    }

    pub fn deleted_old_lines(&self) -> Vec<u32> {
        self.lines
            .iter()
            .filter(|l| l.origin == DiffLineType::Deletion)
            .filter_map(|l| l.old_lineno)
            .collect()
    }
}
