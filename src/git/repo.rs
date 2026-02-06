use anyhow::{Context, Result};
use git2::Repository;
use std::path::Path;

pub fn open_repo(path: &Path) -> Result<Repository> {
    Repository::open(path).with_context(|| format!("opening git repo at {}", path.display()))
}

pub fn head_commit_id(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    Ok(commit.id().to_string())
}

pub fn find_repo_root(start: &Path) -> Result<std::path::PathBuf> {
    let repo = Repository::discover(start)
        .with_context(|| format!("finding git repo from {}", start.display()))?;
    let workdir = repo
        .workdir()
        .context("bare repositories are not supported")?;
    Ok(workdir.to_path_buf())
}
