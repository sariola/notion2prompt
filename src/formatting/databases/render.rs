// src/formatting/databases/render.rs
//! Markdown rendering for tables.
//!
//! This module handles the rendering of table structures to Markdown format,
//! keeping rendering logic separate from data structures.

use super::types::*;
use std::fmt::Write;

/// Trait for rendering tables to different formats.
pub trait TableRenderer {
    /// Renders the table to the target format.
    fn render(&self, table: &Table) -> String;
}

/// Markdown table renderer.
pub struct MarkdownRenderer {
    config: RenderConfig,
}

impl MarkdownRenderer {
    /// Creates a new Markdown renderer with default configuration.
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
        }
    }

    /// Creates a new Markdown renderer with custom configuration.
    #[allow(dead_code)]
    pub fn with_config(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Renders a table with a specific indentation prefix.
    pub fn render_indented(&self, table: &Table, indent: &str) -> String {
        let base_render = self.render(table);
        base_render
            .lines()
            .map(|line| format!("{}{}", indent, line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl TableRenderer for MarkdownRenderer {
    fn render(&self, table: &Table) -> String {
        let mut output = String::new();

        if table.is_empty() {
            output.push_str("*No data available.*\n");
            return output;
        }

        if table.columns.is_empty() {
            output.push_str("*Database has no properties defined.*\n");
            return output;
        }

        // Render header
        self.render_header(&mut output, &table.columns);

        // Render separator
        self.render_separator(&mut output, &table.columns);

        // Render rows
        for row in &table.pages {
            self.render_row(&mut output, row);
        }

        let _ = writeln!(output);
        output
    }
}

impl MarkdownRenderer {
    /// Renders the table header.
    fn render_header(&self, output: &mut String, columns: &[Column]) {
        let _ = write!(output, "| ");
        for (i, col) in columns.iter().enumerate() {
            if i > 0 && self.config.add_spacing {
                let _ = write!(output, " ");
            }
            let _ = write!(output, "{} |", escape_for_table(col.name.as_str()));
        }
        let _ = writeln!(output);
    }

    /// Renders the header separator.
    fn render_separator(&self, output: &mut String, columns: &[Column]) {
        let _ = write!(output, "|");
        for col in columns {
            let _ = write!(output, " {} |", col.alignment.to_markdown());
        }
        let _ = writeln!(output);
    }

    /// Renders a single row.
    fn render_row(&self, output: &mut String, row: &TableRow) {
        let _ = write!(output, "| ");
        for (i, cell) in row.cells.iter().enumerate() {
            if i > 0 && self.config.add_spacing {
                let _ = write!(output, " ");
            }
            let _ = write!(output, "{} |", self.render_cell(cell));
        }
        let _ = writeln!(output);
    }

    /// Renders a single cell.
    fn render_cell(&self, cell: &TableCell) -> String {
        match &cell.value {
            CellValue::Text(text) => escape_for_table(text),
            CellValue::Link { text, url } => {
                if self.config.render_links {
                    let link = format!("[{}]({})", text, url);
                    escape_for_table(&link)
                } else {
                    escape_for_table(text)
                }
            }
            CellValue::Empty => {
                if self.config.show_empty_cells {
                    self.config.empty_cell_text.clone()
                } else {
                    String::new()
                }
            }
        }
    }
}

/// Configuration for table rendering.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Whether to render links as Markdown links.
    pub render_links: bool,
    /// Whether to show empty cells with placeholder text.
    pub show_empty_cells: bool,
    /// Text to use for empty cells.
    pub empty_cell_text: String,
    /// Whether to add extra spacing between columns.
    pub add_spacing: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            render_links: true,
            show_empty_cells: false,
            empty_cell_text: "-".to_string(),
            add_spacing: false,
        }
    }
}

/// Escapes text for use in a Markdown table cell.
pub fn escape_for_table(text: &str) -> String {
    text.replace('|', "\\|")
        .replace('\n', "<br>")
        .replace('\r', "")
}

// --- Convenience Functions ---

impl Table {
    /// Renders the table to Markdown using default settings.
    #[allow(dead_code)]
    pub fn render_markdown(&self) -> String {
        let renderer = MarkdownRenderer::new();
        renderer.render(self)
    }

    /// Renders the table with custom indentation.
    pub fn render_indented(&self, indent: &str) -> String {
        let renderer = MarkdownRenderer::new();
        renderer.render_indented(self, indent)
    }
}

impl CellValue {
    /// Renders the cell value with proper escaping for table cells.
    #[allow(dead_code)]
    pub fn render_escaped(&self) -> String {
        match self {
            CellValue::Text(s) => escape_for_table(s),
            CellValue::Link { text, url } => {
                let link = format!("[{}]({})", text, url);
                escape_for_table(&link)
            }
            CellValue::Empty => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_for_table() {
        assert_eq!(escape_for_table("normal text"), "normal text");
        assert_eq!(escape_for_table("a|b"), "a\\|b");
        assert_eq!(escape_for_table("line1\nline2"), "line1<br>line2");
        assert_eq!(escape_for_table("a|b\nc|d"), "a\\|b<br>c\\|d");
    }

    #[test]
    fn test_empty_table_render() {
        let table = Table::new();
        let rendered = table.render_markdown();
        assert_eq!(rendered, "*No data available.*\n");
    }

    #[test]
    fn test_cell_value_rendering() {
        let text = CellValue::Text("Hello | World".to_string());
        assert_eq!(text.render_escaped(), "Hello \\| World");

        let link = CellValue::Link {
            text: "Click Here".to_string(),
            url: "https://example.com".to_string(),
        };
        assert_eq!(link.render_escaped(), "[Click Here](https://example.com)");

        let empty = CellValue::Empty;
        assert_eq!(empty.render_escaped(), "");
    }
}
