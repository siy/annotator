#[derive(Debug, Clone)]
pub struct Selection {
    pub anchor_line: u32,
    pub anchor_col: u32,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Selection {
    pub fn new(line: u32, col: u32) -> Self {
        Self {
            anchor_line: line,
            anchor_col: col,
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
        }
    }

    pub fn extend_to(&mut self, line: u32, col: u32) {
        if (line, col) < (self.anchor_line, self.anchor_col) {
            self.start_line = line;
            self.start_col = col;
            self.end_line = self.anchor_line;
            self.end_col = self.anchor_col;
        } else {
            self.start_line = self.anchor_line;
            self.start_col = self.anchor_col;
            self.end_line = line;
            self.end_col = col;
        }
    }

    pub fn contains_line(&self, line: u32) -> bool {
        line >= self.start_line && line <= self.end_line
    }
}
