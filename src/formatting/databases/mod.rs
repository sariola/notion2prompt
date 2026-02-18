// src/formatting/databases/mod.rs
//! Type-safe table construction for Notion databases.
//!
//! This module provides a data-oriented approach to formatting databases,
//! separating table structure from rendering concerns.

pub mod builder;
mod render;
mod types;

// Re-export the public interface
pub use builder::{LinkConfig, RelativeUrlResolver, TableBuilder};

use crate::error::AppError;
use crate::model::{Database, Page};
use std::collections::HashSet;
use std::path::Path;

// --- Public API for Backward Compatibility ---

/// Formats rows of a database into a Markdown table.
/// This is the main entry point maintaining backward compatibility.
#[allow(dead_code)]
pub fn tabulate_rows(
    db: &Database,
    pages: &[Page],
    meaningful_rows_ids: &HashSet<String>,
    base_path: &Path,
    db_file_path: &Path,
) -> Result<String, AppError> {
    let url_resolver = RelativeUrlResolver::new(base_path, db_file_path);
    let link_config = LinkConfig {
        meaningful_ids: meaningful_rows_ids,
        url_resolver: Box::new(url_resolver),
    };

    let table = TableBuilder::new(db, pages)
        .with_links(link_config)
        .build()?;

    Ok(table.render_markdown())
}

/// Formats a database inline with proper indentation.
/// Used for nested database display within blocks.
pub fn format_database_inline(
    database: &Database,
    pages: &[Page],
    parent_indent: &str,
) -> Result<String, AppError> {
    log::debug!(
        "format_database_inline: Formatting database '{}' with {} pages",
        database.title().as_plain_text(),
        pages.len()
    );

    if pages.is_empty() {
        log::debug!("  Database has no pages to format");
        return Ok(format!(
            "{}🗄️ **{}**\n{}\n*No data available.*\n\n",
            parent_indent,
            database.title().as_plain_text(),
            parent_indent
        ));
    }

    let table = TableBuilder::new(database, pages)
        .include_empty_rows(true) // Include pages without blocks for child databases
        .build()?;

    log::debug!(
        "  Built table with {} columns and {} rows",
        table.columns.len(),
        table.pages.len()
    );

    let indent = format!("{}  ", parent_indent); // Add 2 spaces for nesting
    let formatted = table.render_indented(&indent);

    // Add database title as header
    let title = database.title().as_plain_text();
    let final_output = if title.is_empty() {
        formatted
    } else {
        format!("{}🗄️ **{}**\n\n{}", parent_indent, title, formatted)
    };

    log::debug!(
        "  format_database_inline output ({} characters):\n{}",
        final_output.len(),
        final_output.lines().take(10).collect::<Vec<_>>().join("\n")
    );

    Ok(final_output)
}

// --- Helper Functions ---

/// Identifies rows that should have their own files.
#[allow(dead_code)]
pub fn identify_meaningful_rows(pages: &[Page]) -> HashSet<String> {
    pages
        .iter()
        .filter(|row| !row.blocks.is_empty())
        .map(|row| row.id.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{ColumnAlignment, PropertyType};

    #[test]
    fn test_column_alignment() {
        assert_eq!(
            PropertyType::Number.default_alignment(),
            ColumnAlignment::Right
        );
        assert_eq!(
            PropertyType::Date.default_alignment(),
            ColumnAlignment::Center
        );
        assert_eq!(
            PropertyType::Title.default_alignment(),
            ColumnAlignment::Left
        );
    }

    #[test]
    fn test_property_type_names() {
        assert_eq!(PropertyType::MultiSelect.display_name(), "Multi-select");
        assert_eq!(PropertyType::CreatedTime.display_name(), "Created time");
        assert_eq!(PropertyType::UniqueId.display_name(), "ID");
    }
}
