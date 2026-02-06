use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

pub struct StatusBar<'a> {
    pub filename: &'a str,
    pub cursor_line: u32,
    pub cursor_col: u32,
    pub annotation_count: usize,
    pub reviewed: usize,
    pub total_files: usize,
    pub message: Option<&'a str>,
    pub annotation_preview: Option<&'a str>,
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = Style::default().bg(Color::Rgb(40, 44, 52)).fg(Color::White);
        let key_style = Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .fg(Color::Rgb(180, 200, 255))
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .fg(Color::DarkGray);

        // Fill background
        for row in 0..area.height {
            for x in area.x..area.x + area.width {
                buf.set_string(x, area.y + row, " ", bg);
            }
        }

        // Row 0: separator
        let sep = "─".repeat(area.width as usize);
        buf.set_string(area.x, area.y, &sep, Style::default().fg(Color::DarkGray));

        // Row 1: file info
        if area.height > 1 {
            let left = if let Some(msg) = self.message {
                format!(" {}  {}", self.filename, msg)
            } else {
                format!(" {}", self.filename)
            };
            let right = format!(
                "Ln {}, Col {} │ {} annotations │ {}/{} reviewed ",
                self.cursor_line,
                self.cursor_col,
                self.annotation_count,
                self.reviewed,
                self.total_files,
            );

            buf.set_string(area.x, area.y + 1, &left, bg);
            let right_x = (area.x + area.width).saturating_sub(right.len() as u16);
            buf.set_string(right_x, area.y + 1, &right, bg);
        }

        // Row 2: annotation preview or hotkey hints
        if area.height > 2 {
            if let Some(preview) = self.annotation_preview {
                let note_style = Style::default()
                    .bg(Color::Rgb(40, 44, 52))
                    .fg(Color::Yellow);
                let label_style = Style::default()
                    .bg(Color::Rgb(40, 44, 52))
                    .fg(Color::Rgb(180, 200, 255))
                    .add_modifier(Modifier::BOLD);
                buf.set_string(area.x + 1, area.y + 2, "Note: ", label_style);
                let max_len = area.width.saturating_sub(8) as usize;
                let text: String = preview
                    .replace('\n', " ")
                    .chars()
                    .take(max_len)
                    .collect();
                buf.set_string(area.x + 7, area.y + 2, &text, note_style);
            } else {
                let hints: &[(&str, &str)] = &[
                    ("^Q", "Quit"),
                    ("Enter", "Annotate/Edit"),
                    ("^D", "Delete"),
                    ("^M", "Clean"),
                    ("^N", "Next"),
                    ("^F", "Files"),
                    ("^T", "Tree"),
                    ("^Z", "Undo"),
                ];

                let mut x = area.x + 1;
                for (key, label) in hints {
                    if x + (key.len() + label.len() + 2) as u16 > area.x + area.width {
                        break;
                    }
                    buf.set_string(x, area.y + 2, key, key_style);
                    x += key.len() as u16;
                    buf.set_string(x, area.y + 2, " ", desc_style);
                    x += 1;
                    buf.set_string(x, area.y + 2, label, desc_style);
                    x += label.len() as u16 + 2;
                }
            }
        }
    }
}
