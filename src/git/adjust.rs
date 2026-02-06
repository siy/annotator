use crate::core::annotation::{AdjustResult, Annotation};
use crate::git::diff::{FileDiff, FileDiffStatus};
use anyhow::Result;
use git2::Repository;

pub fn compute_diffs(repo: &Repository, from_commit: &str, to_commit: &str) -> Result<Vec<FileDiff>> {
    let from_oid = repo.revparse_single(from_commit)?.peel_to_commit()?.id();
    let to_oid = repo.revparse_single(to_commit)?.peel_to_commit()?.id();

    let from_tree = repo.find_commit(from_oid)?.tree()?;
    let to_tree = repo.find_commit(to_oid)?.tree()?;

    let mut diff_opts = git2::DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), Some(&mut diff_opts))?;

    let mut find_opts = git2::DiffFindOptions::new();
    find_opts.renames(true);
    find_opts.copies(false);
    let diff = {
        let mut d = diff;
        d.find_similar(Some(&mut find_opts))?;
        d
    };

    let mut file_diffs = Vec::new();

    for delta_idx in 0..diff.deltas().len() {
        let delta = diff.get_delta(delta_idx).unwrap();
        let status = match delta.status() {
            git2::Delta::Added => FileDiffStatus::Added,
            git2::Delta::Deleted => FileDiffStatus::Deleted,
            git2::Delta::Modified => FileDiffStatus::Modified,
            git2::Delta::Renamed => FileDiffStatus::Renamed,
            _ => continue,
        };

        let old_path = delta.old_file().path().map(|p| p.to_string_lossy().to_string());
        let new_path = delta.new_file().path().map(|p| p.to_string_lossy().to_string());

        let mut hunks = Vec::new();

        if let Ok(patch) = git2::Patch::from_diff(&diff, delta_idx)
            && let Some(patch) = patch {
                for hunk_idx in 0..patch.num_hunks() {
                    let (hunk_header, _) = patch.hunk(hunk_idx)?;
                    let mut lines = Vec::new();

                    for line_idx in 0..patch.num_lines_in_hunk(hunk_idx)? {
                        let line = patch.line_in_hunk(hunk_idx, line_idx)?;
                        let origin = match line.origin() {
                            '+' => crate::git::diff::DiffLineType::Addition,
                            '-' => crate::git::diff::DiffLineType::Deletion,
                            _ => crate::git::diff::DiffLineType::Context,
                        };
                        lines.push(crate::git::diff::DiffLine {
                            origin,
                            old_lineno: line.old_lineno(),
                            new_lineno: line.new_lineno(),
                            content: String::from_utf8_lossy(line.content()).to_string(),
                        });
                    }

                    hunks.push(crate::git::diff::Hunk {
                        old_start: hunk_header.old_start(),
                        old_lines: hunk_header.old_lines(),
                        new_start: hunk_header.new_start(),
                        new_lines: hunk_header.new_lines(),
                        lines,
                    });
                }
            }

        file_diffs.push(FileDiff {
            old_path,
            new_path,
            hunks,
            status,
        });
    }

    Ok(file_diffs)
}

pub fn adjust_annotation(annotation: &Annotation, file_diff: &FileDiff) -> AdjustResult {
    match file_diff.status {
        FileDiffStatus::Deleted => return AdjustResult::Deleted,
        FileDiffStatus::Added => return AdjustResult::Unchanged,
        _ => {}
    }

    let mut offset: i64 = 0;
    let mut deleted_in_range = Vec::new();
    let start = annotation.start_line;
    let end = annotation.end_line;

    for hunk in &file_diff.hunks {
        let hunk_old_end = hunk.old_end();

        if hunk_old_end < start {
            // Hunk entirely before annotation
            offset += hunk.net_offset();
        } else if hunk.old_start > end {
            // Hunk entirely after annotation — stop accumulating
            break;
        } else {
            // Hunk overlaps annotation
            // Check for deleted lines within annotation range
            for deleted_line in hunk.deleted_old_lines() {
                if deleted_line >= start && deleted_line <= end {
                    deleted_in_range.push(deleted_line);
                }
            }

            // Calculate offset contribution from lines before annotation start
            let mut pre_offset: i64 = 0;
            for line in &hunk.lines {
                match line.origin {
                    crate::git::diff::DiffLineType::Deletion => {
                        if let Some(old) = line.old_lineno
                            && old < start {
                                pre_offset -= 1;
                            }
                    }
                    crate::git::diff::DiffLineType::Addition => {
                        if let Some(new) = line.new_lineno {
                            let effective_old = (new as i64 - offset - pre_offset) as u32;
                            if effective_old < start {
                                pre_offset += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
            offset += pre_offset;

            // Calculate size change within annotation range
            let mut range_offset: i64 = 0;
            for line in &hunk.lines {
                match line.origin {
                    crate::git::diff::DiffLineType::Deletion => {
                        if let Some(old) = line.old_lineno
                            && old >= start && old <= end {
                                range_offset -= 1;
                            }
                    }
                    crate::git::diff::DiffLineType::Addition => {
                        if let Some(new) = line.new_lineno {
                            let effective_old = (new as i64 - offset) as u32;
                            if effective_old >= start && effective_old <= end {
                                range_offset += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // After-annotation offset from this hunk
            let total_hunk_offset = hunk.net_offset();
            let _post_offset = total_hunk_offset - pre_offset - range_offset;
        }
    }

    let total_lines = end - start + 1;

    if deleted_in_range.len() as u32 == total_lines {
        return AdjustResult::Deleted;
    }

    if !deleted_in_range.is_empty() {
        return AdjustResult::Conflict { deleted_lines: deleted_in_range };
    }

    let new_start = (start as i64 + offset) as u32;
    let new_end = (end as i64 + offset) as u32;

    if new_start == start && new_end == end {
        AdjustResult::Unchanged
    } else {
        AdjustResult::Shifted {
            old_start: start,
            old_end: end,
            new_start,
            new_end,
        }
    }
}

pub fn adjust_annotations(
    annotations: &[Annotation],
    diffs: &[FileDiff],
) -> Vec<(Annotation, AdjustResult)> {
    let mut results = Vec::new();

    for annotation in annotations {
        // Find diff for this file
        let file_diff = diffs.iter().find(|d| {
            d.old_path.as_deref() == Some(&annotation.file_path)
                || d.new_path.as_deref() == Some(&annotation.file_path)
        });

        let result = match file_diff {
            Some(diff) => adjust_annotation(annotation, diff),
            None => AdjustResult::Unchanged,
        };

        results.push((annotation.clone(), result));
    }

    results
}

pub fn apply_adjustments(annotations: &mut Vec<Annotation>, results: &[(Annotation, AdjustResult)]) {
    for (original, result) in results {
        match result {
            AdjustResult::Shifted { new_start, new_end, .. } => {
                if let Some(a) = annotations.iter_mut().find(|a| a.id == original.id) {
                    a.start_line = *new_start;
                    a.end_line = *new_end;
                    a.updated_at = chrono::Utc::now();
                }
            }
            AdjustResult::Deleted => {
                annotations.retain(|a| a.id != original.id);
            }
            AdjustResult::Conflict { .. } => {
                // Conflicts are handled separately by the UI
            }
            AdjustResult::Unchanged => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::diff::*;

    fn make_annotation(start: u32, end: u32) -> Annotation {
        Annotation::new("test.rs".into(), start, end, "note".into())
    }

    fn make_hunk(old_start: u32, old_lines: u32, new_start: u32, new_lines: u32, lines: Vec<DiffLine>) -> Hunk {
        Hunk { old_start, old_lines, new_start, new_lines, lines }
    }

    fn deletion_line(old_lineno: u32) -> DiffLine {
        DiffLine {
            origin: DiffLineType::Deletion,
            old_lineno: Some(old_lineno),
            new_lineno: None,
            content: "deleted".into(),
        }
    }

    fn addition_line(new_lineno: u32) -> DiffLine {
        DiffLine {
            origin: DiffLineType::Addition,
            old_lineno: None,
            new_lineno: Some(new_lineno),
            content: "added".into(),
        }
    }

    fn context_line(old: u32, new: u32) -> DiffLine {
        DiffLine {
            origin: DiffLineType::Context,
            old_lineno: Some(old),
            new_lineno: Some(new),
            content: "ctx".into(),
        }
    }

    #[test]
    fn test_deleted_file() {
        let a = make_annotation(5, 10);
        let diff = FileDiff {
            old_path: Some("test.rs".into()),
            new_path: None,
            hunks: vec![],
            status: FileDiffStatus::Deleted,
        };
        assert_eq!(adjust_annotation(&a, &diff), AdjustResult::Deleted);
    }

    #[test]
    fn test_no_overlap_before() {
        // 3 lines inserted before annotation at lines 10-15
        let a = make_annotation(10, 15);
        let hunk = make_hunk(1, 0, 1, 3, vec![
            addition_line(1),
            addition_line(2),
            addition_line(3),
        ]);
        let diff = FileDiff {
            old_path: Some("test.rs".into()),
            new_path: Some("test.rs".into()),
            hunks: vec![hunk],
            status: FileDiffStatus::Modified,
        };
        assert_eq!(
            adjust_annotation(&a, &diff),
            AdjustResult::Shifted {
                old_start: 10,
                old_end: 15,
                new_start: 13,
                new_end: 18,
            }
        );
    }

    #[test]
    fn test_no_overlap_after() {
        let a = make_annotation(5, 10);
        let hunk = make_hunk(20, 2, 20, 5, vec![
            context_line(20, 20),
            deletion_line(21),
            addition_line(21),
            addition_line(22),
            addition_line(23),
            addition_line(24),
        ]);
        let diff = FileDiff {
            old_path: Some("test.rs".into()),
            new_path: Some("test.rs".into()),
            hunks: vec![hunk],
            status: FileDiffStatus::Modified,
        };
        assert_eq!(adjust_annotation(&a, &diff), AdjustResult::Unchanged);
    }

    #[test]
    fn test_all_lines_deleted() {
        let a = make_annotation(5, 7);
        let hunk = make_hunk(5, 3, 5, 0, vec![
            deletion_line(5),
            deletion_line(6),
            deletion_line(7),
        ]);
        let diff = FileDiff {
            old_path: Some("test.rs".into()),
            new_path: Some("test.rs".into()),
            hunks: vec![hunk],
            status: FileDiffStatus::Modified,
        };
        assert_eq!(adjust_annotation(&a, &diff), AdjustResult::Deleted);
    }

    #[test]
    fn test_partial_deletion_conflict() {
        let a = make_annotation(5, 10);
        let hunk = make_hunk(5, 3, 5, 1, vec![
            context_line(5, 5),
            deletion_line(6),
            deletion_line(7),
        ]);
        let diff = FileDiff {
            old_path: Some("test.rs".into()),
            new_path: Some("test.rs".into()),
            hunks: vec![hunk],
            status: FileDiffStatus::Modified,
        };
        let result = adjust_annotation(&a, &diff);
        match result {
            AdjustResult::Conflict { deleted_lines } => {
                assert_eq!(deleted_lines, vec![6, 7]);
            }
            other => panic!("expected Conflict, got {:?}", other),
        }
    }

    #[test]
    fn test_added_file_unchanged() {
        let a = make_annotation(1, 5);
        let diff = FileDiff {
            old_path: None,
            new_path: Some("test.rs".into()),
            hunks: vec![],
            status: FileDiffStatus::Added,
        };
        assert_eq!(adjust_annotation(&a, &diff), AdjustResult::Unchanged);
    }
}
