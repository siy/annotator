use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, FontStyle};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_lines(&self, content: &str, file_path: &str) -> Vec<Line<'static>> {
        let syntax = self
            .syntax_set
            .find_syntax_for_file(file_path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();

        for line in LinesWithEndings::from(content) {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();

            let spans: Vec<Span<'static>> = ranges
                .into_iter()
                .map(|(style, text)| {
                    let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                    let mut ratatui_style = Style::default().fg(fg);
                    if style.font_style.contains(FontStyle::BOLD) {
                        ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::BOLD);
                    }
                    if style.font_style.contains(FontStyle::ITALIC) {
                        ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::ITALIC);
                    }
                    Span::styled(text.trim_end_matches('\n').to_string(), ratatui_style)
                })
                .collect();

            result.push(Line::from(spans));
        }

        result
    }
}
