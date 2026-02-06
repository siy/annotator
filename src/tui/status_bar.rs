use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

pub struct StatusBar<'a> {
    pub filename: &'a str,
    pub cursor_line: u32,
    pub cursor_col: u32,
    pub annotation_count: usize,
    pub reviewed: usize,
    pub total_files: usize,
    pub message: Option<&'a str>,
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = Style::default().bg(Color::Rgb(40, 44, 52)).fg(Color::White);

        // Fill background
        for x in area.x..area.x + area.width {
            buf.set_string(x, area.y, " ", bg);
            if area.height > 1 {
                buf.set_string(x, area.y + 1, " ", bg);
            }
        }

        // Separator line
        let sep = "─".repeat(area.width as usize);
        buf.set_string(area.x, area.y, &sep, Style::default().fg(Color::DarkGray));

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
    }
}
