use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

pub struct AnnotationPopup<'a> {
    pub text: &'a str,
    pub cursor_pos: usize,
    pub selection_line: u32,
    pub scroll_offset: u32,
    pub viewport_height: u16,
    pub is_edit: bool,
}

impl<'a> AnnotationPopup<'a> {
    pub fn popup_rect(&self, area: Rect) -> Rect {
        let width = area.width.min(60);
        let height = 8u16.min(area.height / 2);
        let x = (area.width.saturating_sub(width)) / 2 + area.x;

        // Position above or below selection
        let selection_screen_row =
            self.selection_line.saturating_sub(self.scroll_offset + 1) as u16;
        let y = if selection_screen_row > height + 2 {
            selection_screen_row - height - 1 + area.y
        } else {
            (selection_screen_row + 2).min(area.height.saturating_sub(height)) + area.y
        };

        Rect::new(x, y, width, height)
    }
}

impl<'a> Widget for AnnotationPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup = self.popup_rect(area);
        let border_style = Style::default().fg(Color::Cyan);
        let bg = Style::default().bg(Color::Rgb(30, 34, 42)).fg(Color::White);

        // Clear popup area
        for y in popup.y..popup.y + popup.height {
            for x in popup.x..popup.x + popup.width {
                buf.set_string(x, y, " ", bg);
            }
        }

        // Border
        let top = format!(
            "┌{}┐",
            "─".repeat((popup.width.saturating_sub(2)) as usize)
        );
        let bottom = format!(
            "└{}┘",
            "─".repeat((popup.width.saturating_sub(2)) as usize)
        );
        buf.set_string(popup.x, popup.y, &top, border_style);
        buf.set_string(popup.x, popup.y + popup.height - 1, &bottom, border_style);
        for y in popup.y + 1..popup.y + popup.height - 1 {
            buf.set_string(popup.x, y, "│", border_style);
            buf.set_string(popup.x + popup.width - 1, y, "│", border_style);
        }

        // Title
        let title = if self.is_edit {
            " Edit Annotation "
        } else {
            " New Annotation "
        };
        buf.set_string(
            popup.x + 2,
            popup.y,
            title,
            border_style.add_modifier(Modifier::BOLD),
        );

        // Text content
        let inner_width = (popup.width.saturating_sub(4)) as usize;
        let lines: Vec<&str> = self.text.split('\n').collect();
        let max_lines = (popup.height.saturating_sub(3)) as usize;
        for (i, line) in lines.iter().take(max_lines).enumerate() {
            let display: String = line.chars().take(inner_width).collect();
            buf.set_string(popup.x + 2, popup.y + 1 + i as u16, &display, bg);
        }

        // Cursor
        let cursor_line = self.text[..self.cursor_pos].matches('\n').count();
        let cursor_col = self.text[..self.cursor_pos]
            .rfind('\n')
            .map(|p| self.cursor_pos - p - 1)
            .unwrap_or(self.cursor_pos);
        if cursor_line < max_lines && cursor_col < inner_width {
            let cx = popup.x + 2 + cursor_col as u16;
            let cy = popup.y + 1 + cursor_line as u16;
            if cx < popup.x + popup.width - 1 && cy < popup.y + popup.height - 1 {
                buf.set_style(
                    Rect::new(cx, cy, 1, 1),
                    bg.add_modifier(Modifier::REVERSED),
                );
            }
        }

        // Help text
        let help = "Enter: confirm │ Esc: cancel";
        if popup.height >= 4 {
            buf.set_string(
                popup.x + 2,
                popup.y + popup.height - 2,
                help,
                Style::default().fg(Color::DarkGray).bg(Color::Rgb(30, 34, 42)),
            );
        }
    }
}
