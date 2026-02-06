use crate::core::annotation::FileStatus;
use crate::core::store::Store;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug)]
pub enum TreeNode {
    Dir {
        name: String,
        children: BTreeMap<String, TreeNode>,
    },
    File {
        name: String,
        full_path: String,
    },
}

impl TreeNode {
    pub fn build(files: &[String]) -> TreeNode {
        let mut root = BTreeMap::new();
        for file in files {
            let parts: Vec<&str> = file.split('/').collect();
            insert_path(&mut root, &parts, file);
        }
        TreeNode::Dir {
            name: ".".into(),
            children: root,
        }
    }

    pub fn flatten(
        &self,
        expanded: &HashSet<String>,
        prefix: &str,
    ) -> Vec<(String, String, bool)> {
        // Returns (display_text, path_or_key, is_dir)
        let mut result = Vec::new();
        if let TreeNode::Dir { children, .. } = self {
            for node in children.values() {
                match node {
                    TreeNode::Dir { name, children: _ } => {
                        let path = if prefix.is_empty() {
                            name.clone()
                        } else {
                            format!("{}/{}", prefix, name)
                        };
                        let is_expanded = expanded.contains(&path);
                        let icon = if is_expanded { "▾ " } else { "▸ " };
                        let indent = prefix.matches('/').count();
                        let display = format!(
                            "{}{}{}/",
                            "  ".repeat(indent),
                            icon,
                            name
                        );
                        result.push((display, path.clone(), true));
                        if is_expanded {
                            result.extend(node.flatten(expanded, &path));
                        }
                    }
                    TreeNode::File { name, full_path } => {
                        let indent = prefix.matches('/').count();
                        let display = format!("{}  {}", "  ".repeat(indent), name);
                        result.push((display, full_path.clone(), false));
                    }
                }
            }
        }
        result
    }
}

fn insert_path(tree: &mut BTreeMap<String, TreeNode>, parts: &[&str], full_path: &str) {
    if parts.len() == 1 {
        tree.insert(
            parts[0].to_string(),
            TreeNode::File {
                name: parts[0].to_string(),
                full_path: full_path.to_string(),
            },
        );
    } else {
        let dir_name = parts[0].to_string();
        let node = tree
            .entry(dir_name.clone())
            .or_insert_with(|| TreeNode::Dir {
                name: dir_name,
                children: BTreeMap::new(),
            });
        if let TreeNode::Dir { children, .. } = node {
            insert_path(children, &parts[1..], full_path);
        }
    }
}

pub struct TreeViewPopup<'a> {
    pub files: &'a [String],
    pub expanded: &'a HashSet<String>,
    pub selected: usize,
    pub store: &'a Store,
}

impl<'a> Widget for TreeViewPopup<'a> {
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

        buf.set_string(
            area.x + 2,
            area.y,
            " Tree ",
            border_style.add_modifier(Modifier::BOLD),
        );

        let tree = TreeNode::build(self.files);
        let items = tree.flatten(self.expanded, "");
        let list_start = area.y + 1;
        let max_items = (area.height.saturating_sub(3)) as usize;

        let scroll = if self.selected >= max_items {
            self.selected - max_items + 1
        } else {
            0
        };

        for (i, (display, path, is_dir)) in items.iter().skip(scroll).take(max_items).enumerate() {
            let display_idx = scroll + i;
            let is_selected = display_idx == self.selected;

            let status_icon = if !is_dir {
                match self.store.get_file_status(path).unwrap_or(FileStatus::Unreviewed) {
                    FileStatus::Unreviewed => " ",
                    FileStatus::Annotated => "A",
                    FileStatus::Clean => "✓",
                }
            } else {
                " "
            };

            let style = if is_selected {
                bg.add_modifier(Modifier::REVERSED)
            } else {
                bg
            };

            let inner_width = area.width.saturating_sub(6) as usize;
            let entry = format!("{} {}", status_icon, display);
            let truncated: String = entry.chars().take(inner_width).collect();
            buf.set_string(area.x + 2, list_start + i as u16, &truncated, style);
        }

        // Help
        if area.height >= 3 {
            let help = "Enter: open/toggle │ Esc: close";
            buf.set_string(
                area.x + 2,
                area.y + area.height - 2,
                help,
                Style::default().fg(Color::DarkGray).bg(Color::Rgb(30, 34, 42)),
            );
        }
    }
}
