use crate::core::annotation::{Annotation, FileStatus};
use crate::core::session::Session;
use crate::core::store::Store;
use crate::core::undo::{UndoAction, UndoStack};
use crate::tui::selection::Selection;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Viewing,
    AnnotationInput,
    AnnotationEdit,
    FileList,
    TreeView,
    ConflictResolution,
}

pub struct App {
    pub repo_root: PathBuf,
    pub store: Store,
    pub session: Session,
    pub mode: AppMode,
    pub files: Vec<String>,
    pub current_file_index: usize,
    pub file_content: Vec<String>,
    pub cursor_line: u32,
    pub cursor_col: u32,
    pub scroll_offset: u32,
    pub viewport_height: u16,
    pub viewport_width: u16,
    pub selection: Option<Selection>,
    pub annotations: Vec<Annotation>,
    pub undo_stack: UndoStack,
    pub should_quit: bool,
    pub annotation_input: String,
    pub annotation_input_cursor: usize,
    pub editing_annotation_id: Option<uuid::Uuid>,
    pub file_list_filter: String,
    pub file_list_selected: usize,
    pub tree_expanded: std::collections::HashSet<String>,
    pub tree_selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(repo_root: PathBuf) -> anyhow::Result<Self> {
        let annotator_dir = repo_root.join(".annotator");
        let store = Store::new(&annotator_dir);
        store.ensure_dir()?;

        let session = Session::load(&annotator_dir.join("session.json"))?;
        let files = crate::core::file_list::list_tracked_files(&repo_root)?;
        let annotations = store.load_annotations()?;

        let current_file_index = session
            .current_file
            .as_ref()
            .and_then(|f| files.iter().position(|x| x == f))
            .unwrap_or(0);

        let file_content = if !files.is_empty() {
            load_file_content(&repo_root, &files[current_file_index])
        } else {
            Vec::new()
        };

        Ok(Self {
            repo_root,
            store,
            mode: AppMode::Viewing,
            files,
            current_file_index,
            file_content,
            cursor_line: session.current_line.max(1),
            cursor_col: session.current_col,
            scroll_offset: session.scroll_offset,
            viewport_height: 24,
            viewport_width: 80,
            selection: None,
            annotations,
            undo_stack: UndoStack::default(),
            should_quit: false,
            annotation_input: String::new(),
            annotation_input_cursor: 0,
            editing_annotation_id: None,
            file_list_filter: String::new(),
            file_list_selected: 0,
            tree_expanded: std::collections::HashSet::new(),
            tree_selected: 0,
            status_message: None,
            session,
        })
    }

    pub fn current_file(&self) -> Option<&str> {
        self.files.get(self.current_file_index).map(|s| s.as_str())
    }

    pub fn current_file_annotations(&self) -> Vec<&Annotation> {
        let file = match self.current_file() {
            Some(f) => f,
            None => return Vec::new(),
        };
        self.annotations
            .iter()
            .filter(|a| a.file_path == file)
            .collect()
    }

    pub fn load_current_file(&mut self) {
        if let Some(file) = self.current_file() {
            self.file_content = load_file_content(&self.repo_root, file);
        } else {
            self.file_content = Vec::new();
        }
    }

    pub fn switch_to_file(&mut self, index: usize) {
        if index < self.files.len() {
            self.current_file_index = index;
            self.cursor_line = 1;
            self.cursor_col = 0;
            self.scroll_offset = 0;
            self.selection = None;
            self.load_current_file();
        }
    }

    pub fn next_unreviewed_file(&mut self) {
        let start = self.current_file_index + 1;
        for i in 0..self.files.len() {
            let idx = (start + i) % self.files.len();
            let status = self
                .store
                .get_file_status(&self.files[idx])
                .unwrap_or(FileStatus::Unreviewed);
            if status == FileStatus::Unreviewed {
                self.switch_to_file(idx);
                return;
            }
        }
        self.status_message = Some("All files reviewed!".into());
    }

    pub fn create_annotation(&mut self) {
        let file = match self.current_file() {
            Some(f) => f.to_string(),
            None => return,
        };

        let (start, end) = if let Some(ref sel) = self.selection {
            (sel.start_line, sel.end_line)
        } else {
            (self.cursor_line, self.cursor_line)
        };

        let annotation = Annotation::new(file.clone(), start, end, self.annotation_input.clone());
        self.undo_stack
            .push(UndoAction::Create(annotation.clone()));
        self.annotations.push(annotation.clone());
        let _ = self.store.append_annotation(&annotation);
        let _ = self
            .store
            .set_file_status(&file, FileStatus::Annotated);
        self.annotation_input.clear();
        self.annotation_input_cursor = 0;
        self.selection = None;
        self.mode = AppMode::Viewing;
    }

    pub fn update_annotation(&mut self) {
        if let Some(id) = self.editing_annotation_id
            && let Some(annotation) = self.annotations.iter_mut().find(|a| a.id == id) {
                let old = annotation.clone();
                annotation.text = self.annotation_input.clone();
                annotation.updated_at = chrono::Utc::now();
                let new = annotation.clone();
                self.undo_stack
                    .push(UndoAction::Update { old, new: new.clone() });
                let _ = self.store.update_annotation(&new);
            }
        self.editing_annotation_id = None;
        self.annotation_input.clear();
        self.annotation_input_cursor = 0;
        self.mode = AppMode::Viewing;
    }

    pub fn delete_annotation_at_cursor(&mut self) {
        let file = match self.current_file() {
            Some(f) => f.to_string(),
            None => return,
        };
        let line = self.cursor_line;
        if let Some(idx) = self
            .annotations
            .iter()
            .position(|a| a.file_path == file && a.contains_line(line))
        {
            let removed = self.annotations.remove(idx);
            self.undo_stack
                .push(UndoAction::Delete(removed.clone()));
            let _ = self.store.delete_annotation(removed.id);

            let has_annotations = self.annotations.iter().any(|a| a.file_path == file);
            if !has_annotations {
                let _ = self
                    .store
                    .set_file_status(&file, FileStatus::Unreviewed);
            }
        }
    }

    pub fn mark_file_clean(&mut self) {
        if let Some(file) = self.current_file() {
            let _ = self
                .store
                .set_file_status(file, FileStatus::Clean);
            self.status_message = Some(format!("Marked {} as clean", file));
            self.next_unreviewed_file();
        }
    }

    pub fn apply_undo(&mut self) {
        if let Some(action) = self.undo_stack.undo() {
            self.apply_undo_action(&action);
        }
    }

    pub fn apply_redo(&mut self) {
        if let Some(action) = self.undo_stack.redo() {
            self.apply_undo_action(&action);
        }
    }

    fn apply_undo_action(&mut self, action: &UndoAction) {
        match action {
            UndoAction::Create(a) => {
                self.annotations.push(a.clone());
                let _ = self.store.append_annotation(a);
            }
            UndoAction::Delete(a) => {
                self.annotations.retain(|x| x.id != a.id);
                let _ = self.store.delete_annotation(a.id);
            }
            UndoAction::Update { new, .. } => {
                if let Some(existing) = self.annotations.iter_mut().find(|a| a.id == new.id) {
                    *existing = new.clone();
                    let _ = self.store.update_annotation(new);
                }
            }
        }
    }

    pub fn save_session(&self) {
        let session = Session {
            current_file: self.current_file().map(|s| s.to_string()),
            current_line: self.cursor_line,
            current_col: self.cursor_col,
            scroll_offset: self.scroll_offset,
            last_adjust_commit: self.session.last_adjust_commit.clone(),
        };
        let path = self.repo_root.join(".annotator/session.json");
        let _ = session.save(&path);
    }

    pub fn ensure_cursor_visible(&mut self) {
        let view_h = self.viewport_height as u32;
        if view_h == 0 {
            return;
        }
        if self.cursor_line < self.scroll_offset + 1 {
            self.scroll_offset = self.cursor_line.saturating_sub(1);
        } else if self.cursor_line > self.scroll_offset + view_h {
            self.scroll_offset = self.cursor_line - view_h;
        }
    }

    pub fn total_lines(&self) -> u32 {
        self.file_content.len() as u32
    }

    pub fn review_progress(&self) -> (usize, usize) {
        let total = self.files.len();
        let reviewed = self.files.iter().filter(|f| {
            self.store.get_file_status(f).unwrap_or(FileStatus::Unreviewed) != FileStatus::Unreviewed
        }).count();
        (reviewed, total)
    }
}

fn load_file_content(repo_root: &Path, relative_path: &str) -> Vec<String> {
    let full = repo_root.join(relative_path);
    match std::fs::read_to_string(&full) {
        Ok(content) => content.lines().map(|l| l.to_string()).collect(),
        Err(_) => vec!["[Error reading file]".to_string()],
    }
}
