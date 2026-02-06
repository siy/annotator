use crate::core::annotation::Annotation;
use std::collections::BTreeMap;

pub fn export_markdown(annotations: &[Annotation]) -> String {
    if annotations.is_empty() {
        return "# Annotations\n\nNo annotations found.\n".to_string();
    }

    let mut by_file: BTreeMap<&str, Vec<&Annotation>> = BTreeMap::new();
    for a in annotations {
        by_file.entry(&a.file_path).or_default().push(a);
    }

    let mut out = String::from("# Annotations\n\n");

    for (file, mut anns) in by_file {
        anns.sort_by_key(|a| a.start_line);
        out.push_str(&format!("## `{file}`\n\n"));
        for a in anns {
            if a.start_line == a.end_line {
                out.push_str(&format!("- **Line {}**: {}\n", a.start_line, a.text));
            } else {
                out.push_str(&format!(
                    "- **Lines {}-{}**: {}\n",
                    a.start_line, a.end_line, a.text
                ));
            }
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let md = export_markdown(&[]);
        assert!(md.contains("No annotations found"));
    }

    #[test]
    fn test_export() {
        let anns = vec![
            Annotation::new("src/b.rs".into(), 10, 20, "refactor this".into()),
            Annotation::new("src/a.rs".into(), 5, 5, "fix bug".into()),
            Annotation::new("src/a.rs".into(), 15, 18, "add tests".into()),
        ];
        let md = export_markdown(&anns);
        assert!(md.contains("## `src/a.rs`"));
        assert!(md.contains("## `src/b.rs`"));
        assert!(md.contains("**Line 5**: fix bug"));
        assert!(md.contains("**Lines 10-20**: refactor this"));
        assert!(md.contains("**Lines 15-18**: add tests"));
        // a.rs should come before b.rs (sorted)
        let a_pos = md.find("src/a.rs").unwrap();
        let b_pos = md.find("src/b.rs").unwrap();
        assert!(a_pos < b_pos);
    }
}
