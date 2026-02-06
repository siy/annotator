use crate::core::annotation::FileStatus;
use crate::core::store::Store;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

pub struct FileListPopup<'a> {
    pub files: &'a [String],
    pub filter: &'a str,
    pub selected: usize,
    pub store: &'a Store,
}

impl<'a> FileListPopup<'a> {
    pub fn filtered_files(&self) -> Vec<(usize, &'a String)> {
        self.files
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                if self.filter.is_empty() {
                    return true;
                }
                let pattern = glob::Pattern::new(self.filter);
                match pattern {
                    Ok(p) => p.matches(f),
                    Err(_) => f.contains(self.filter),
                }
            })
            .collect()
    }
}

impl<'a> Widget for FileListPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = Style::default().bg(Color::Rgb(30, 34, 42)).fg(Color::White);
        let border_style = Style::default().fg(Color::Cyan);

        // Clear area
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.set_string(x, y, " ", bg);
            }
        }

        // Border
        let top = format!("┌{}┐", "─".repeat(area.width.saturating_sub(2) as usize));
        let bottom = format!("└{}┘", "─".repeat(area.width.saturating_sub(2) as usize));
        buf.set_string(area.x, area.y, &top, border_style);
        buf.set_string(area.x, area.y + area.height - 1, &bottom, border_style);
        for y in area.y + 1..area.y + area.height - 1 {
            buf.set_string(area.x, y, "│", border_style);
            buf.set_string(area.x + area.width - 1, y, "│", border_style);
        }

        // Title
        buf.set_string(
            area.x + 2,
            area.y,
            " Files ",
            border_style.add_modifier(Modifier::BOLD),
        );

        // Filter input
        let filter_str = format!("Filter: {}", self.filter);
        buf.set_string(area.x + 2, area.y + 1, &filter_str, bg);

        // File list
        let filtered = self.filtered_files();
        let list_start = area.y + 3;
        let max_items = (area.height.saturating_sub(5)) as usize;

        let scroll = if self.selected >= max_items {
            self.selected - max_items + 1
        } else {
            0
        };

        for (i, (_, file)) in filtered.iter().skip(scroll).take(max_items).enumerate() {
            let display_idx = scroll + i;
            let status = self
                .store
                .get_file_status(file)
                .unwrap_or(FileStatus::Unreviewed);
            let icon = match status {
                FileStatus::Unreviewed => "[ ]",
                FileStatus::Annotated => "[A]",
                FileStatus::Clean => "[OK]",
            };

            let is_selected = display_idx == self.selected;
            let style = if is_selected {
                bg.add_modifier(Modifier::REVERSED)
            } else {
                bg
            };

            let inner_width = area.width.saturating_sub(4) as usize;
            let entry = format!("{} {}", icon, file);
            let display: String = entry.chars().take(inner_width).collect();
            buf.set_string(area.x + 2, list_start + i as u16, &display, style);
        }

        // Help
        if area.height >= 5 {
            let help = "Enter: open │ Esc: close │ Type to filter";
            buf.set_string(
                area.x + 2,
                area.y + area.height - 2,
                help,
                Style::default().fg(Color::DarkGray).bg(Color::Rgb(30, 34, 42)),
            );
        }
    }
}
