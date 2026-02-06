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

        // Build a set of annotation end_lines to show inline text after
        let annotation_display: Vec<(u32, &str)> = self
            .annotations
            .iter()
            .map(|a| (a.end_line, a.text.as_str()))
            .collect();

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
            } else if is_annotated {
                Style::default().fg(Color::Rgb(200, 180, 100))
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
            let mut code_end_col = 0u16;
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
                code_end_col = col;
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

            // Show annotation text inline at the end_line of each annotation
            for (end_line, text) in &annotation_display {
                if line_num == *end_line {
                    let note_style = Style::default()
                        .fg(Color::Rgb(180, 160, 80))
                        .bg(Color::Rgb(50, 50, 30));
                    let gap = 2u16;
                    let start_col = code_end_col + gap;
                    if start_col < code_area.width {
                        let prefix = " // ";
                        let max_chars = (code_area.width - start_col) as usize;
                        let note_text: String = text
                            .replace('\n', " ")
                            .chars()
                            .take(max_chars.saturating_sub(prefix.len()))
                            .collect();
                        let display = format!("{}{}", prefix, note_text);
                        buf.set_string(
                            code_area.x + start_col,
                            area.y + row,
                            &display,
                            note_style,
                        );
                    }
                }
            }
        }
    }
}
