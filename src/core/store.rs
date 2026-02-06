use crate::core::annotation::{Annotation, FileReviewState};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct Store {
    annotations_path: PathBuf,
    file_status_path: PathBuf,
}

impl Store {
    pub fn new(annotator_dir: &Path) -> Self {
        Self {
            annotations_path: annotator_dir.join("annotations.jsonl"),
            file_status_path: annotator_dir.join("file_status.jsonl"),
        }
    }

    pub fn ensure_dir(&self) -> Result<()> {
        if let Some(parent) = self.annotations_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    // --- Annotations ---

    pub fn load_annotations(&self) -> Result<Vec<Annotation>> {
        load_jsonl(&self.annotations_path)
    }

    pub fn append_annotation(&self, annotation: &Annotation) -> Result<()> {
        self.ensure_dir()?;
        append_jsonl(&self.annotations_path, annotation)
    }

    pub fn save_annotations(&self, annotations: &[Annotation]) -> Result<()> {
        self.ensure_dir()?;
        atomic_write_jsonl(&self.annotations_path, annotations)
    }

    pub fn update_annotation(&self, updated: &Annotation) -> Result<()> {
        let mut all = self.load_annotations()?;
        if let Some(existing) = all.iter_mut().find(|a| a.id == updated.id) {
            *existing = updated.clone();
        }
        self.save_annotations(&all)
    }

    pub fn delete_annotation(&self, id: Uuid) -> Result<()> {
        let all = self.load_annotations()?;
        let filtered: Vec<_> = all.into_iter().filter(|a| a.id != id).collect();
        self.save_annotations(&filtered)
    }

    pub fn annotations_for_file(&self, file_path: &str) -> Result<Vec<Annotation>> {
        Ok(self
            .load_annotations()?
            .into_iter()
            .filter(|a| a.file_path == file_path)
            .collect())
    }

    // --- File status ---

    pub fn load_file_statuses(&self) -> Result<Vec<FileReviewState>> {
        load_jsonl(&self.file_status_path)
    }

    pub fn save_file_statuses(&self, statuses: &[FileReviewState]) -> Result<()> {
        self.ensure_dir()?;
        atomic_write_jsonl(&self.file_status_path, statuses)
    }

    pub fn set_file_status(&self, file_path: &str, status: crate::core::annotation::FileStatus) -> Result<()> {
        let mut all = self.load_file_statuses()?;
        if let Some(existing) = all.iter_mut().find(|s| s.file_path == file_path) {
            existing.status = status;
        } else {
            all.push(FileReviewState {
                file_path: file_path.to_string(),
                status,
            });
        }
        self.save_file_statuses(&all)
    }

    pub fn get_file_status(&self, file_path: &str) -> Result<crate::core::annotation::FileStatus> {
        let all = self.load_file_statuses()?;
        Ok(all
            .iter()
            .find(|s| s.file_path == file_path)
            .map(|s| s.status)
            .unwrap_or_default())
    }
}

fn load_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let mut items = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let item: T = serde_json::from_str(line)
            .with_context(|| format!("parsing line {} of {}", i + 1, path.display()))?;
        items.push(item);
    }
    Ok(items)
}

fn append_jsonl<T: serde::Serialize>(path: &Path, item: &T) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let json = serde_json::to_string(item)?;
    writeln!(file, "{json}")?;
    Ok(())
}

fn atomic_write_jsonl<T: serde::Serialize>(path: &Path, items: &[T]) -> Result<()> {
    let tmp = path.with_extension("jsonl.tmp");
    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp)?;
        for item in items {
            let json = serde_json::to_string(item)?;
            writeln!(file, "{json}")?;
        }
        file.flush()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::annotation::{FileStatus, Annotation};
    use tempfile::TempDir;

    fn make_store() -> (TempDir, Store) {
        let dir = TempDir::new().unwrap();
        let annotator_dir = dir.path().join(".annotator");
        std::fs::create_dir_all(&annotator_dir).unwrap();
        let store = Store::new(&annotator_dir);
        (dir, store)
    }

    #[test]
    fn test_empty_load() {
        let (_dir, store) = make_store();
        assert!(store.load_annotations().unwrap().is_empty());
        assert!(store.load_file_statuses().unwrap().is_empty());
    }

    #[test]
    fn test_append_and_load_annotations() {
        let (_dir, store) = make_store();
        let a1 = Annotation::new("f1.rs".into(), 1, 5, "note1".into());
        let a2 = Annotation::new("f2.rs".into(), 10, 20, "note2".into());

        store.append_annotation(&a1).unwrap();
        store.append_annotation(&a2).unwrap();

        let loaded = store.load_annotations().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], a1);
        assert_eq!(loaded[1], a2);
    }

    #[test]
    fn test_atomic_rewrite() {
        let (_dir, store) = make_store();
        let a1 = Annotation::new("f.rs".into(), 1, 1, "a".into());
        let a2 = Annotation::new("f.rs".into(), 2, 2, "b".into());

        store.append_annotation(&a1).unwrap();
        store.append_annotation(&a2).unwrap();

        // Rewrite with just a1
        store.save_annotations(&[a1.clone()]).unwrap();
        let loaded = store.load_annotations().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], a1);
    }

    #[test]
    fn test_update_annotation() {
        let (_dir, store) = make_store();
        let mut a = Annotation::new("f.rs".into(), 1, 5, "old".into());
        store.append_annotation(&a).unwrap();

        a.text = "new".into();
        a.updated_at = chrono::Utc::now();
        store.update_annotation(&a).unwrap();

        let loaded = store.load_annotations().unwrap();
        assert_eq!(loaded[0].text, "new");
    }

    #[test]
    fn test_delete_annotation() {
        let (_dir, store) = make_store();
        let a1 = Annotation::new("f.rs".into(), 1, 1, "a".into());
        let a2 = Annotation::new("f.rs".into(), 2, 2, "b".into());
        store.append_annotation(&a1).unwrap();
        store.append_annotation(&a2).unwrap();

        store.delete_annotation(a1.id).unwrap();
        let loaded = store.load_annotations().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, a2.id);
    }

    #[test]
    fn test_annotations_for_file() {
        let (_dir, store) = make_store();
        let a1 = Annotation::new("f1.rs".into(), 1, 1, "a".into());
        let a2 = Annotation::new("f2.rs".into(), 2, 2, "b".into());
        let a3 = Annotation::new("f1.rs".into(), 3, 3, "c".into());
        store.append_annotation(&a1).unwrap();
        store.append_annotation(&a2).unwrap();
        store.append_annotation(&a3).unwrap();

        let f1 = store.annotations_for_file("f1.rs").unwrap();
        assert_eq!(f1.len(), 2);
        assert_eq!(f1[0].id, a1.id);
        assert_eq!(f1[1].id, a3.id);
    }

    #[test]
    fn test_file_status() {
        let (_dir, store) = make_store();
        assert_eq!(store.get_file_status("f.rs").unwrap(), FileStatus::Unreviewed);

        store.set_file_status("f.rs", FileStatus::Clean).unwrap();
        assert_eq!(store.get_file_status("f.rs").unwrap(), FileStatus::Clean);

        store.set_file_status("f.rs", FileStatus::Annotated).unwrap();
        assert_eq!(store.get_file_status("f.rs").unwrap(), FileStatus::Annotated);
    }
}
