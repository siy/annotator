use anyhow::{Context, Result};
use std::path::Path;

/// Lists all git-tracked files in the repository, skipping binary files.
pub fn list_tracked_files(repo_path: &Path) -> Result<Vec<String>> {
    let repo = git2::Repository::open(repo_path)
        .with_context(|| format!("opening git repo at {}", repo_path.display()))?;
    let index = repo.index()?;
    let mut files = Vec::new();

    for entry in index.iter() {
        let path = String::from_utf8_lossy(&entry.path).to_string();
        let full_path = repo_path.join(&path);

        if is_binary_path(&full_path) {
            continue;
        }

        files.push(path);
    }

    files.sort();
    Ok(files)
}

fn is_binary_path(path: &Path) -> bool {
    // Check by extension first
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let binary_exts = [
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg",
            "pdf", "zip", "tar", "gz", "bz2", "xz", "7z",
            "exe", "dll", "so", "dylib", "o", "a",
            "wasm", "class", "pyc", "pyo",
            "ttf", "otf", "woff", "woff2", "eot",
            "mp3", "mp4", "wav", "avi", "mkv", "mov",
            "db", "sqlite", "sqlite3",
        ];
        if binary_exts.contains(&ext.to_lowercase().as_str()) {
            return true;
        }
    }

    // Check file content for null bytes (binary indicator)
    if let Ok(data) = std::fs::read(path) {
        let check_len = data.len().min(8192);
        return data[..check_len].contains(&0);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn add_and_commit(dir: &Path, files: &[&str]) {
        for f in files {
            Command::new("git")
                .args(["add", f])
                .current_dir(dir)
                .output()
                .unwrap();
        }
        Command::new("git")
            .args(["commit", "-m", "test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn test_list_tracked_files() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("lib.rs"), "pub fn foo() {}").unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/util.rs"), "// util").unwrap();

        add_and_commit(dir.path(), &["main.rs", "lib.rs", "src/util.rs"]);

        let files = list_tracked_files(dir.path()).unwrap();
        assert_eq!(files, vec!["lib.rs", "main.rs", "src/util.rs"]);
    }

    #[test]
    fn test_binary_skipped() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        std::fs::write(dir.path().join("code.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("image.png"), &[0u8; 100]).unwrap();

        add_and_commit(dir.path(), &["code.rs", "image.png"]);

        let files = list_tracked_files(dir.path()).unwrap();
        assert_eq!(files, vec!["code.rs"]);
    }

    #[test]
    fn test_binary_by_content() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        std::fs::write(dir.path().join("text.dat"), "hello world").unwrap();
        let mut binary_content = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x57];
        binary_content.extend_from_slice(&[0u8; 50]);
        std::fs::write(dir.path().join("binary.dat"), &binary_content).unwrap();

        add_and_commit(dir.path(), &["text.dat", "binary.dat"]);

        let files = list_tracked_files(dir.path()).unwrap();
        assert_eq!(files, vec!["text.dat"]);
    }
}
