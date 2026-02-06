use crate::tui::annotation_popup::AnnotationPopup;
use crate::tui::app::{App, AppMode};
use crate::tui::file_list_popup::FileListPopup;
use crate::tui::highlight::Highlighter;
use crate::tui::status_bar::StatusBar;
use crate::tui::tree_view::TreeViewPopup;
use crate::tui::viewer::FileViewer;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

pub fn render(frame: &mut Frame, app: &App, highlighter: &Highlighter) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(size);

    let viewer_area = chunks[0];
    let status_area = chunks[1];

    // Highlight file content
    let content = app.file_content.join("\n");
    let file_path = app.current_file().unwrap_or("unknown");
    let highlighted = highlighter.highlight_lines(&content, file_path);

    let annotations = app.current_file_annotations();
    let viewer = FileViewer {
        highlighted_lines: &highlighted,
        scroll_offset: app.scroll_offset,
        cursor_line: app.cursor_line,
        cursor_col: app.cursor_col,
        annotations: &annotations,
        selection: &app.selection,
    };
    frame.render_widget(viewer, viewer_area);

    // Status bar
    let (reviewed, total) = app.review_progress();
    let status = StatusBar {
        filename: app.current_file().unwrap_or("(no file)"),
        cursor_line: app.cursor_line,
        cursor_col: app.cursor_col,
        annotation_count: annotations.len(),
        reviewed,
        total_files: total,
        message: app.status_message.as_deref(),
    };
    frame.render_widget(status, status_area);

    // Render popups based on mode
    match app.mode {
        AppMode::AnnotationInput => {
            let popup = AnnotationPopup {
                text: &app.annotation_input,
                cursor_pos: app.annotation_input_cursor,
                selection_line: app.selection.as_ref().map_or(app.cursor_line, |s| s.start_line),
                scroll_offset: app.scroll_offset,
                viewport_height: viewer_area.height,
                is_edit: false,
            };
            frame.render_widget(popup, viewer_area);
        }
        AppMode::AnnotationEdit => {
            let popup = AnnotationPopup {
                text: &app.annotation_input,
                cursor_pos: app.annotation_input_cursor,
                selection_line: app.cursor_line,
                scroll_offset: app.scroll_offset,
                viewport_height: viewer_area.height,
                is_edit: true,
            };
            frame.render_widget(popup, viewer_area);
        }
        AppMode::FileList => {
            let popup = FileListPopup {
                files: &app.files,
                filter: &app.file_list_filter,
                selected: app.file_list_selected,
                store: &app.store,
            };
            frame.render_widget(popup, size);
        }
        AppMode::TreeView => {
            let popup = TreeViewPopup {
                files: &app.files,
                expanded: &app.tree_expanded,
                selected: app.tree_selected,
                store: &app.store,
            };
            frame.render_widget(popup, size);
        }
        AppMode::ConflictResolution => {
            // Conflict resolution is handled separately
        }
        AppMode::Viewing => {}
    }
}
