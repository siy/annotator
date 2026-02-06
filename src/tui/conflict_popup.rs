use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    Keep,
    Delete,
    Edit,
}

pub struct ConflictPopup<'a> {
    pub file_path: &'a str,
    pub start_line: u32,
    pub end_line: u32,
    pub annotation_text: &'a str,
    pub deleted_lines: &'a [u32],
    pub selected_choice: ConflictChoice,
}

impl<'a> Widget for ConflictPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = Style::default().bg(Color::Rgb(40, 30, 30)).fg(Color::White);
        let border_style = Style::default().fg(Color::Red);

        let popup_width = area.width.min(70);
        let popup_height = area.height.min(15);
        let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
        let popup = Rect::new(x, y, popup_width, popup_height);

        // Clear
        for py in popup.y..popup.y + popup.height {
            for px in popup.x..popup.x + popup.width {
                buf.set_string(px, py, " ", bg);
            }
        }

        // Border
        let top = format!("┌{}┐", "─".repeat(popup.width.saturating_sub(2) as usize));
        let bottom = format!("└{}┘", "─".repeat(popup.width.saturating_sub(2) as usize));
        buf.set_string(popup.x, popup.y, &top, border_style);
        buf.set_string(popup.x, popup.y + popup.height - 1, &bottom, border_style);
        for py in popup.y + 1..popup.y + popup.height - 1 {
            buf.set_string(popup.x, py, "│", border_style);
            buf.set_string(popup.x + popup.width - 1, py, "│", border_style);
        }

        buf.set_string(
            popup.x + 2,
            popup.y,
            " Annotation Conflict ",
            border_style.add_modifier(Modifier::BOLD),
        );

        // Info
        let info = format!(
            "File: {} (lines {}-{})",
            self.file_path, self.start_line, self.end_line
        );
        buf.set_string(popup.x + 2, popup.y + 1, &info, bg);

        let deleted = format!(
            "Deleted lines: {:?}",
            self.deleted_lines
        );
        buf.set_string(popup.x + 2, popup.y + 2, &deleted, bg.fg(Color::Red));

        // Annotation text preview
        let text_preview: String = self.annotation_text.chars().take(popup.width as usize - 6).collect();
        buf.set_string(
            popup.x + 2,
            popup.y + 4,
            format!("Note: {}", text_preview),
            bg,
        );

        // Choices
        let choices = [
            (ConflictChoice::Keep, "Keep annotation (adjust lines)"),
            (ConflictChoice::Delete, "Delete annotation"),
            (ConflictChoice::Edit, "Edit annotation"),
        ];
        for (i, (choice, label)) in choices.iter().enumerate() {
            let style = if *choice == self.selected_choice {
                bg.add_modifier(Modifier::REVERSED)
            } else {
                bg
            };
            let prefix = if *choice == self.selected_choice { "▸ " } else { "  " };
            buf.set_string(
                popup.x + 2,
                popup.y + 6 + i as u16,
                format!("{}{}", prefix, label),
                style,
            );
        }
    }
}
