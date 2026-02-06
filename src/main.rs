#![allow(dead_code)]

use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

mod cli;
mod core;
mod export;
mod git;
mod tui;

use cli::{Cli, Command, ExportFormat};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Review { path } => cmd_review(&path),
        Command::Adjust { path, auto_resolve } => cmd_adjust(&path, auto_resolve),
        Command::Export { path, format } => cmd_export(&path, format),
        Command::Status { path } => cmd_status(&path),
    }
}

fn cmd_review(path: &Path) -> Result<()> {
    let repo_root = git::repo::find_repo_root(path)?;
    let mut app = tui::app::App::new(repo_root)?;

    // Check for pending adjustments
    if let Some(ref last_commit) = app.session.last_adjust_commit.clone() {
        let repo = git::repo::open_repo(&app.repo_root)?;
        let head = git::repo::head_commit_id(&repo)?;
        if head != *last_commit {
            run_adjustment(&mut app, last_commit, &head)?;
        }
    } else {
        // Set initial commit
        let repo = git::repo::open_repo(&app.repo_root)?;
        if let Ok(head) = git::repo::head_commit_id(&repo) {
            app.session.last_adjust_commit = Some(head);
        }
    }

    run_tui(app)
}

fn run_tui(mut app: tui::app::App) -> Result<()> {
    use crossterm::{
        event::Event,
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use std::io;
    use std::time::Duration;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let highlighter = tui::highlight::Highlighter::new();

    loop {
        terminal.draw(|f| {
            let size = f.area();
            app.viewport_height = size.height.saturating_sub(3);
            app.viewport_width = size.width;
            tui::render::render(f, &app, &highlighter);
        })?;

        if app.should_quit {
            break;
        }

        if let Some(Event::Key(key)) = tui::event::poll_event(Duration::from_millis(100))? {
            handle_key(&mut app, key);
        }
    }

    app.save_session();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key(app: &mut tui::app::App, key: crossterm::event::KeyEvent) {
    use tui::app::AppMode;
    use tui::keymap::*;

    match app.mode {
        AppMode::Viewing => {
            if let Some(action) = map_key_viewing(key) {
                handle_viewing_action(app, action);
            }
        }
        AppMode::AnnotationInput | AppMode::AnnotationEdit => {
            if let Some(action) = map_key_input(key) {
                handle_input_action(app, action);
            }
        }
        AppMode::FileList => {
            if let Some(action) = map_key_file_list(key) {
                handle_file_list_action(app, action);
            }
        }
        AppMode::TreeView => {
            if let Some(action) = map_key_tree(key) {
                handle_tree_action(app, action);
            }
        }
        AppMode::ConflictResolution => {
            if let Some(action) = map_key_conflict(key) {
                handle_conflict_action(app, action);
            }
        }
    }
}

fn handle_viewing_action(app: &mut tui::app::App, action: tui::keymap::Action) {
    use tui::keymap::Action;
    use tui::selection::Selection;

    match action {
        Action::CursorUp => {
            app.cursor_line = app.cursor_line.saturating_sub(1).max(1);
            app.selection = None;
            app.ensure_cursor_visible();
        }
        Action::CursorDown => {
            app.cursor_line = (app.cursor_line + 1).min(app.total_lines().max(1));
            app.selection = None;
            app.ensure_cursor_visible();
        }
        Action::CursorLeft => {
            app.cursor_col = app.cursor_col.saturating_sub(1);
            app.selection = None;
        }
        Action::CursorRight => {
            app.cursor_col += 1;
            app.selection = None;
        }
        Action::PageUp => {
            let page = app.viewport_height as u32;
            app.cursor_line = app.cursor_line.saturating_sub(page).max(1);
            app.scroll_offset = app.scroll_offset.saturating_sub(page);
            app.selection = None;
        }
        Action::PageDown => {
            let page = app.viewport_height as u32;
            let max = app.total_lines().max(1);
            app.cursor_line = (app.cursor_line + page).min(max);
            app.scroll_offset = (app.scroll_offset + page).min(max.saturating_sub(1));
            app.selection = None;
        }
        Action::Home => {
            app.cursor_col = 0;
        }
        Action::End => {
            if let Some(line) = app.file_content.get(app.cursor_line as usize - 1) {
                app.cursor_col = line.len() as u32;
            }
        }
        Action::SelectUp => {
            let new_line = app.cursor_line.saturating_sub(1).max(1);
            if app.selection.is_none() {
                app.selection = Some(Selection::new(app.cursor_line, app.cursor_col));
            }
            app.cursor_line = new_line;
            app.selection.as_mut().unwrap().extend_to(app.cursor_line, app.cursor_col);
            app.ensure_cursor_visible();
        }
        Action::SelectDown => {
            let new_line = (app.cursor_line + 1).min(app.total_lines().max(1));
            if app.selection.is_none() {
                app.selection = Some(Selection::new(app.cursor_line, app.cursor_col));
            }
            app.cursor_line = new_line;
            app.selection.as_mut().unwrap().extend_to(app.cursor_line, app.cursor_col);
            app.ensure_cursor_visible();
        }
        Action::SelectLeft => {
            let new_col = app.cursor_col.saturating_sub(1);
            if app.selection.is_none() {
                app.selection = Some(Selection::new(app.cursor_line, app.cursor_col));
            }
            app.cursor_col = new_col;
            app.selection.as_mut().unwrap().extend_to(app.cursor_line, app.cursor_col);
        }
        Action::SelectRight => {
            if app.selection.is_none() {
                app.selection = Some(Selection::new(app.cursor_line, app.cursor_col));
            }
            app.cursor_col += 1;
            app.selection.as_mut().unwrap().extend_to(app.cursor_line, app.cursor_col);
        }
        Action::CreateAnnotation => {
            app.mode = tui::app::AppMode::AnnotationInput;
            app.annotation_input.clear();
            app.annotation_input_cursor = 0;
        }
        Action::EditAnnotation => {
            let file = app.current_file().map(|s| s.to_string());
            if let Some(file) = file {
                let line = app.cursor_line;
                if let Some(ann) = app.annotations.iter().find(|a| a.file_path == file && a.contains_line(line)) {
                    app.editing_annotation_id = Some(ann.id);
                    app.annotation_input = ann.text.clone();
                    app.annotation_input_cursor = ann.text.len();
                    app.mode = tui::app::AppMode::AnnotationEdit;
                }
            }
        }
        Action::DeleteAnnotation => app.delete_annotation_at_cursor(),
        Action::MarkClean => app.mark_file_clean(),
        Action::NextUnreviewed => app.next_unreviewed_file(),
        Action::OpenFileList => {
            app.mode = tui::app::AppMode::FileList;
            app.file_list_filter.clear();
            app.file_list_selected = 0;
        }
        Action::OpenTreeView => {
            app.mode = tui::app::AppMode::TreeView;
            app.tree_selected = 0;
        }
        Action::Undo => app.apply_undo(),
        Action::Redo => app.apply_redo(),
        Action::Quit => {
            app.should_quit = true;
        }
        _ => {}
    }
    app.status_message = None;
}

fn handle_input_action(app: &mut tui::app::App, action: tui::keymap::Action) {
    use tui::keymap::Action;

    match action {
        Action::Confirm => {
            if app.mode == tui::app::AppMode::AnnotationEdit {
                app.update_annotation();
            } else {
                app.create_annotation();
            }
        }
        Action::Cancel => {
            app.mode = tui::app::AppMode::Viewing;
            app.annotation_input.clear();
            app.annotation_input_cursor = 0;
            app.editing_annotation_id = None;
        }
        Action::InputChar(c) => {
            app.annotation_input.insert(app.annotation_input_cursor, c);
            app.annotation_input_cursor += c.len_utf8();
        }
        Action::InputBackspace => {
            if app.annotation_input_cursor > 0 {
                let prev = app.annotation_input[..app.annotation_input_cursor]
                    .chars()
                    .last()
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
                app.annotation_input_cursor -= prev;
                app.annotation_input.remove(app.annotation_input_cursor);
            }
        }
        Action::InputDelete => {
            if app.annotation_input_cursor < app.annotation_input.len() {
                app.annotation_input.remove(app.annotation_input_cursor);
            }
        }
        _ => {}
    }
}

fn handle_file_list_action(app: &mut tui::app::App, action: tui::keymap::Action) {
    use tui::keymap::Action;

    let popup = tui::file_list_popup::FileListPopup {
        files: &app.files,
        filter: &app.file_list_filter,
        selected: app.file_list_selected,
        store: &app.store,
    };

    match action {
        Action::Cancel => {
            app.mode = tui::app::AppMode::Viewing;
        }
        Action::Confirm => {
            let filtered = popup.filtered_files();
            if let Some((orig_idx, _)) = filtered.get(app.file_list_selected) {
                app.switch_to_file(*orig_idx);
            }
            app.mode = tui::app::AppMode::Viewing;
        }
        Action::CursorUp => {
            app.file_list_selected = app.file_list_selected.saturating_sub(1);
        }
        Action::CursorDown => {
            let filtered = popup.filtered_files();
            app.file_list_selected = (app.file_list_selected + 1).min(filtered.len().saturating_sub(1));
        }
        Action::InputChar(c) => {
            app.file_list_filter.push(c);
            app.file_list_selected = 0;
        }
        Action::InputBackspace => {
            app.file_list_filter.pop();
            app.file_list_selected = 0;
        }
        _ => {}
    }
}

fn handle_tree_action(app: &mut tui::app::App, action: tui::keymap::Action) {
    use tui::keymap::Action;
    use tui::tree_view::TreeNode;

    let tree = TreeNode::build(&app.files);
    let items = tree.flatten(&app.tree_expanded, "");

    match action {
        Action::Cancel => {
            app.mode = tui::app::AppMode::Viewing;
        }
        Action::Confirm => {
            if let Some((_, path, is_dir)) = items.get(app.tree_selected) {
                if *is_dir {
                    if app.tree_expanded.contains(path) {
                        app.tree_expanded.remove(path);
                    } else {
                        app.tree_expanded.insert(path.clone());
                    }
                } else {
                    if let Some(idx) = app.files.iter().position(|f| f == path) {
                        app.switch_to_file(idx);
                    }
                    app.mode = tui::app::AppMode::Viewing;
                }
            }
        }
        Action::CursorUp => {
            app.tree_selected = app.tree_selected.saturating_sub(1);
        }
        Action::CursorDown => {
            app.tree_selected = (app.tree_selected + 1).min(items.len().saturating_sub(1));
        }
        _ => {}
    }
}

fn handle_conflict_action(app: &mut tui::app::App, _action: tui::keymap::Action) {
    app.mode = tui::app::AppMode::Viewing;
}

fn run_adjustment(app: &mut tui::app::App, from: &str, to: &str) -> Result<()> {
    let repo = git::repo::open_repo(&app.repo_root)?;
    let diffs = git::adjust::compute_diffs(&repo, from, to)?;

    git::rename::apply_renames(&mut app.annotations, &diffs);

    let results = git::adjust::adjust_annotations(&app.annotations, &diffs);
    git::adjust::apply_adjustments(&mut app.annotations, &results);

    app.store.save_annotations(&app.annotations)?;

    app.session.last_adjust_commit = Some(to.to_string());
    app.save_session();

    Ok(())
}

fn cmd_adjust(path: &Path, _auto_resolve: bool) -> Result<()> {
    let repo_root = git::repo::find_repo_root(path)?;
    let annotator_dir = repo_root.join(".annotator");
    let store = core::store::Store::new(&annotator_dir);
    let session = core::session::Session::load(&annotator_dir.join("session.json"))?;

    let last_commit = session
        .last_adjust_commit
        .clone()
        .context("No previous adjust commit recorded. Run 'annotator review' first.")?;

    let repo = git::repo::open_repo(&repo_root)?;
    let head = git::repo::head_commit_id(&repo)?;

    if head == last_commit {
        println!("Already up to date.");
        return Ok(());
    }

    let diffs = git::adjust::compute_diffs(&repo, &last_commit, &head)?;
    let mut annotations = store.load_annotations()?;

    let renames = git::rename::apply_renames(&mut annotations, &diffs);
    for (old, new) in &renames {
        println!("Renamed: {} -> {}", old, new);
    }

    let results = git::adjust::adjust_annotations(&annotations, &diffs);
    let mut conflicts = Vec::new();
    let mut shifted = 0;
    let mut deleted = 0;

    for (ann, result) in &results {
        match result {
            core::annotation::AdjustResult::Shifted { old_start, old_end, new_start, new_end } => {
                println!(
                    "Shifted: {}:{}-{} -> {}-{}",
                    ann.file_path, old_start, old_end, new_start, new_end
                );
                shifted += 1;
            }
            core::annotation::AdjustResult::Deleted => {
                println!("Deleted: {}:{}-{}", ann.file_path, ann.start_line, ann.end_line);
                deleted += 1;
            }
            core::annotation::AdjustResult::Conflict { deleted_lines } => {
                println!(
                    "CONFLICT: {}:{}-{} (deleted lines: {:?})",
                    ann.file_path, ann.start_line, ann.end_line, deleted_lines
                );
                conflicts.push(ann.clone());
            }
            core::annotation::AdjustResult::Unchanged => {}
        }
    }

    git::adjust::apply_adjustments(&mut annotations, &results);
    store.save_annotations(&annotations)?;

    let mut new_session = session;
    new_session.last_adjust_commit = Some(head);
    new_session.save(&annotator_dir.join("session.json"))?;

    println!(
        "\nAdjusted: {} shifted, {} deleted, {} conflicts",
        shifted,
        deleted,
        conflicts.len()
    );

    Ok(())
}

fn cmd_export(path: &Path, format: ExportFormat) -> Result<()> {
    let repo_root = git::repo::find_repo_root(path)?;
    let store = core::store::Store::new(&repo_root.join(".annotator"));
    let annotations = store.load_annotations()?;

    let output = match format {
        ExportFormat::Markdown => export::markdown::export_markdown(&annotations),
        ExportFormat::Json => export::json::export_json(&annotations)?,
    };

    println!("{}", output);
    Ok(())
}

fn cmd_status(path: &Path) -> Result<()> {
    let repo_root = git::repo::find_repo_root(path)?;
    let store = core::store::Store::new(&repo_root.join(".annotator"));
    let annotations = store.load_annotations()?;
    let files = core::file_list::list_tracked_files(&repo_root)?;
    let statuses = store.load_file_statuses()?;

    let total = files.len();
    let clean = statuses
        .iter()
        .filter(|s| s.status == core::annotation::FileStatus::Clean)
        .count();
    let annotated = statuses
        .iter()
        .filter(|s| s.status == core::annotation::FileStatus::Annotated)
        .count();
    let unreviewed = total - clean - annotated;

    println!("Review Progress");
    println!("===============");
    println!("Total files:   {}", total);
    println!("Unreviewed:    {}", unreviewed);
    println!("Annotated:     {}", annotated);
    println!("Clean:         {}", clean);
    println!("Annotations:   {}", annotations.len());

    if total > 0 {
        let pct = ((clean + annotated) as f64 / total as f64 * 100.0) as u32;
        println!("Progress:      {}%", pct);
    }

    Ok(())
}
