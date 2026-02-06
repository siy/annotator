use crate::core::annotation::Annotation;
use crate::tui::selection::Selection;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Widget;

const GUTTER_WIDTH: u16 = 7;

pub struct FileViewer<'a> {
    pub highlighted_lines: &'a [Line<'a>],
    pub scroll_offset: u32,
    pub cursor_line: u32,
    pub cursor_col: u32,
    pub annotations: &'a [&'a Annotation],
    pub selection: &'a Option<Selection>,
}

impl<'a> Widget for FileViewer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let code_area = Rect {
            x: area.x + GUTTER_WIDTH + 1,
            y: area.y,
            width: area.width.saturating_sub(GUTTER_WIDTH + 1),
            height: area.height,
        };

        for row in 0..area.height {
            let line_num = self.scroll_offset + row as u32 + 1;
            let is_annotated = self
                .annotations
                .iter()
                .any(|a| a.contains_line(line_num));
            let is_cursor_line = line_num == self.cursor_line;
            let is_selected = self
                .selection
                .as_ref()
                .is_some_and(|s| s.contains_line(line_num));

            // Gutter: line number + annotation marker
            let marker = if is_annotated { ">" } else { " " };
            let gutter_style = if is_cursor_line {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line_num_str = if (line_num as usize) <= self.highlighted_lines.len() {
                format!("{:>4} {} ", line_num, marker)
            } else {
                format!("   ~ {} ", marker)
            };

            buf.set_string(area.x, area.y + row, &line_num_str, gutter_style);

            // Separator
            buf.set_string(
                area.x + GUTTER_WIDTH,
                area.y + row,
                "│",
                Style::default().fg(Color::DarkGray),
            );

            // Code content
            if (line_num as usize) <= self.highlighted_lines.len() {
                let line = &self.highlighted_lines[line_num as usize - 1];
                let mut col = 0u16;
                for span in &line.spans {
                    let text = &span.content;
                    for ch in text.chars() {
                        if col >= code_area.width {
                            break;
                        }
                        let mut style = span.style;
                        if is_selected {
                            style = style.bg(Color::Rgb(68, 68, 120));
                        } else if is_annotated {
                            style = style.bg(Color::Rgb(50, 50, 30));
                        }
                        if is_cursor_line && col == self.cursor_col as u16 {
                            style = style.add_modifier(Modifier::REVERSED);
                        }
                        buf.set_string(
                            code_area.x + col,
                            area.y + row,
                            ch.to_string(),
                            style,
                        );
                        col += 1;
                    }
                }
                // Fill remaining with cursor/selection styles
                while col < code_area.width {
                    let mut style = Style::default();
                    if is_selected {
                        style = style.bg(Color::Rgb(68, 68, 120));
                    } else if is_annotated {
                        style = style.bg(Color::Rgb(50, 50, 30));
                    }
                    if is_cursor_line && col == self.cursor_col as u16 {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    buf.set_string(code_area.x + col, area.y + row, " ", style);
                    col += 1;
                }
            }
        }
    }
}
