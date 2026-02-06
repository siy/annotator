use crate::core::annotation::Annotation;
use crate::git::diff::{FileDiff, FileDiffStatus};

/// Update annotation file paths for renamed files.
/// Returns the list of annotations that had their paths updated.
pub fn apply_renames(annotations: &mut [Annotation], diffs: &[FileDiff]) -> Vec<(String, String)> {
    let mut renames = Vec::new();

    for diff in diffs {
        if diff.status == FileDiffStatus::Renamed
            && let (Some(old), Some(new)) = (&diff.old_path, &diff.new_path) {
                for annotation in annotations.iter_mut() {
                    if annotation.file_path == *old {
                        annotation.file_path = new.clone();
                        annotation.updated_at = chrono::Utc::now();
                    }
                }
                renames.push((old.clone(), new.clone()));
            }
    }

    renames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_renames() {
        let mut annotations = vec![
            Annotation::new("old/path.rs".into(), 1, 5, "note".into()),
            Annotation::new("other.rs".into(), 1, 1, "note2".into()),
        ];

        let diffs = vec![FileDiff {
            old_path: Some("old/path.rs".into()),
            new_path: Some("new/path.rs".into()),
            hunks: vec![],
            status: FileDiffStatus::Renamed,
        }];

        let renames = apply_renames(&mut annotations, &diffs);
        assert_eq!(renames, vec![("old/path.rs".into(), "new/path.rs".into())]);
        assert_eq!(annotations[0].file_path, "new/path.rs");
        assert_eq!(annotations[1].file_path, "other.rs");
    }

    #[test]
    fn test_no_renames() {
        let mut annotations = vec![
            Annotation::new("f.rs".into(), 1, 1, "note".into()),
        ];
        let diffs = vec![FileDiff {
            old_path: Some("f.rs".into()),
            new_path: Some("f.rs".into()),
            hunks: vec![],
            status: FileDiffStatus::Modified,
        }];

        let renames = apply_renames(&mut annotations, &diffs);
        assert!(renames.is_empty());
        assert_eq!(annotations[0].file_path, "f.rs");
    }
}
