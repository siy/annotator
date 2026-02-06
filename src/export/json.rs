use crate::core::annotation::Annotation;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct ExportAnnotation<'a> {
    file_path: &'a str,
    start_line: u32,
    end_line: u32,
    text: &'a str,
}

#[derive(Serialize)]
struct ExportFile<'a> {
    file: &'a str,
    annotations: Vec<ExportAnnotation<'a>>,
}

#[derive(Serialize)]
struct ExportRoot<'a> {
    files: Vec<ExportFile<'a>>,
    total_annotations: usize,
}

pub fn export_json(annotations: &[Annotation]) -> anyhow::Result<String> {
    let mut by_file: BTreeMap<&str, Vec<&Annotation>> = BTreeMap::new();
    for a in annotations {
        by_file.entry(&a.file_path).or_default().push(a);
    }

    let files: Vec<ExportFile> = by_file
        .into_iter()
        .map(|(file, mut anns)| {
            anns.sort_by_key(|a| a.start_line);
            ExportFile {
                file,
                annotations: anns
                    .iter()
                    .map(|a| ExportAnnotation {
                        file_path: &a.file_path,
                        start_line: a.start_line,
                        end_line: a.end_line,
                        text: &a.text,
                    })
                    .collect(),
            }
        })
        .collect();

    let root = ExportRoot {
        total_annotations: annotations.len(),
        files,
    };

    Ok(serde_json::to_string_pretty(&root)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let json = export_json(&[]).unwrap();
        assert!(json.contains("\"total_annotations\": 0"));
    }

    #[test]
    fn test_export() {
        let anns = vec![
            Annotation::new("src/a.rs".into(), 5, 10, "fix".into()),
            Annotation::new("src/b.rs".into(), 1, 1, "note".into()),
        ];
        let json = export_json(&anns).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["total_annotations"], 2);
        assert_eq!(parsed["files"].as_array().unwrap().len(), 2);
    }
}
