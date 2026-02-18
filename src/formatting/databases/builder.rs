// src/formatting/databases/builder.rs
//! Table building logic for Notion databases.
//!
//! This module provides builders for constructing tables from database data,
//! keeping construction logic separate from rendering.

use super::types::*;
use crate::error::AppError;
use crate::formatting::properties::render_property_value;
use crate::model::{Database, DatabasePropertyType, Page};
use crate::output::create_clean_filename;
use std::collections::HashSet;

// --- Table Builder ---

/// Builder for constructing tables from databases.
pub struct TableBuilder<'a> {
    database: &'a Database,
    pages: &'a [Page],
    config: TableConfig<'a>,
}

impl<'a> TableBuilder<'a> {
    /// Creates a new table builder.
    pub fn new(database: &'a Database, pages: &'a [Page]) -> Self {
        Self {
            database,
            pages,
            config: TableConfig::default(),
        }
    }

    /// Sets the link configuration.
    #[allow(dead_code)]
    pub fn with_links(mut self, config: LinkConfig<'a>) -> Self {
        self.config.link_config = Some(config);
        self
    }

    /// Sets whether to include empty rows.
    #[allow(dead_code)]
    pub fn include_empty_rows(mut self, include: bool) -> Self {
        self.config.include_empty_pages = include;
        self
    }

    /// Sets the maximum number of rows to include.
    #[allow(dead_code)]
    pub fn max_rows(mut self, max: usize) -> Self {
        self.config.max_pages = Some(max);
        self
    }

    /// Builds the table.
    pub fn build(self) -> Result<Table, AppError> {
        let columns = self.build_columns()?;
        let rows = self.build_rows(&columns)?;
        let metadata = self.calculate_metadata(&columns, &rows);

        Ok(Table {
            columns,
            pages: rows,
            metadata,
        })
    }

    /// Builds columns from database properties.
    fn build_columns(&self) -> Result<Vec<Column>, AppError> {
        log::debug!(
            "build_columns: Building columns for database '{}'",
            self.database.title().as_plain_text()
        );
        log::debug!(
            "  Database properties: {:?}",
            self.database.properties.keys().collect::<Vec<_>>()
        );

        let mut columns: Vec<Column> = self
            .database
            .properties
            .iter()
            .map(|(name, schema)| {
                let property_type = property_type_from_schema(&schema.property_type);
                log::debug!("  Creating column '{}' of type {:?}", name, property_type);
                Column {
                    name: name.clone(),
                    property_type,
                    alignment: property_type.default_alignment(),
                    width_hint: None,
                }
            })
            .collect();

        // Sort columns to put title first if present
        columns.sort_by(|a, b| match (a.property_type, b.property_type) {
            (PropertyType::Title, PropertyType::Title) => std::cmp::Ordering::Equal,
            (PropertyType::Title, _) => std::cmp::Ordering::Less,
            (_, PropertyType::Title) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(columns)
    }

    /// Builds rows from pages.
    fn build_rows(&self, columns: &[Column]) -> Result<Vec<TableRow>, AppError> {
        let pages_to_process = if let Some(max) = self.config.max_pages {
            &self.pages[..self.pages.len().min(max)]
        } else {
            self.pages
        };

        pages_to_process
            .iter()
            .filter(|page| self.config.include_empty_pages || self.is_meaningful_row(page))
            .map(|page| self.build_row(page, columns))
            .collect()
    }

    /// Builds a single row from a page.
    fn build_row(&self, page: &Page, columns: &[Column]) -> Result<TableRow, AppError> {
        let mut row = TableRow::new(page.id.as_str().to_string());

        for column in columns {
            let cell = self.build_cell(page, column)?;
            row = row.with_cell(cell);
        }

        Ok(row)
    }

    /// Builds a single cell for a column.
    fn build_cell(&self, page: &Page, column: &Column) -> Result<TableCell, AppError> {
        let property_value = page.properties.get(&column.name);

        // Debug logging to track property lookup
        log::debug!(
            "build_cell: Looking for property '{}' in page '{}'",
            column.name,
            page.title().as_str()
        );
        log::debug!(
            "  Available properties: {:?}",
            page.properties.keys().collect::<Vec<_>>()
        );
        log::debug!("  Property found: {}", property_value.is_some());

        let formatted = render_property_value(property_value)?;

        log::debug!(
            "  Formatted value: '{}' (empty: {})",
            formatted,
            formatted.is_empty()
        );

        let value = match column.property_type {
            PropertyType::Title => self.build_title_cell_value(page, &formatted),
            _ if formatted.is_empty() => CellValue::Empty,
            _ => CellValue::Text(formatted),
        };

        let mut cell = TableCell::new(value);
        if column.property_type == PropertyType::Title {
            cell.metadata.is_title = true;
        }

        Ok(cell)
    }

    /// Builds the cell value for a title column.
    fn build_title_cell_value(&self, page: &Page, formatted: &str) -> CellValue {
        match &self.config.link_config {
            Some(link_config) if link_config.meaningful_ids.contains(page.id.as_str()) => {
                let title = if formatted.is_empty() {
                    page.title().as_str().to_string()
                } else {
                    formatted.to_string()
                };

                let row_filename = create_clean_filename(&title, page.id.as_str(), true);
                let url = link_config.url_resolver.resolve(&row_filename);

                CellValue::Link { text: title, url }
            }
            _ => self.format_title_without_link(page, formatted),
        }
    }

    /// Formats a title cell without a link.
    fn format_title_without_link(&self, page: &Page, formatted: &str) -> CellValue {
        if formatted.is_empty() {
            CellValue::Text(format!("*Untitled Row ({})*", page.id.as_str()))
        } else {
            CellValue::Text(formatted.to_string())
        }
    }

    /// Checks if a row is meaningful (has content).
    fn is_meaningful_row(&self, page: &Page) -> bool {
        !page.blocks.is_empty()
    }

    /// Calculates table metadata.
    fn calculate_metadata(&self, columns: &[Column], pages: &[TableRow]) -> TableMetadata {
        TableMetadata {
            has_title_column: columns
                .iter()
                .any(|c| c.property_type == PropertyType::Title),
            has_links: pages.iter().any(|row| {
                row.cells
                    .iter()
                    .any(|cell| matches!(cell.value, CellValue::Link { .. }))
            }),
            total_cells: columns.len() * pages.len(),
        }
    }
}

// --- Configuration Types ---

/// Configuration for table building.
#[derive(Default)]
struct TableConfig<'a> {
    link_config: Option<LinkConfig<'a>>,
    include_empty_pages: bool,
    max_pages: Option<usize>,
}

/// Configuration for generating links in table cells.
pub struct LinkConfig<'a> {
    pub meaningful_ids: &'a HashSet<String>,
    pub url_resolver: Box<dyn UrlResolver + 'a>,
}

/// Trait for resolving URLs for table links.
pub trait UrlResolver {
    /// Resolves a filename to a URL.
    fn resolve(&self, filename: &str) -> String;
}

/// Relative path URL resolver.
pub struct RelativeUrlResolver<'a> {
    base_path: &'a std::path::Path,
    from_path: &'a std::path::Path,
}

impl<'a> RelativeUrlResolver<'a> {
    #[allow(dead_code)]
    pub fn new(base_path: &'a std::path::Path, from_path: &'a std::path::Path) -> Self {
        Self {
            base_path,
            from_path,
        }
    }
}

impl UrlResolver for RelativeUrlResolver<'_> {
    fn resolve(&self, filename: &str) -> String {
        let target_path = self.base_path.join(filename);
        crate::output::get_relative_path(self.from_path, &target_path)
            .unwrap_or_else(|_| filename.to_string())
    }
}

// --- Helper Functions ---

/// Converts a property schema to a property type.
fn property_type_from_schema(schema: &DatabasePropertyType) -> PropertyType {
    PropertyType::from(schema)
}

// --- Type Conversions ---

impl From<&DatabasePropertyType> for PropertyType {
    fn from(schema: &DatabasePropertyType) -> Self {
        use DatabasePropertyType::*;
        match schema {
            Title => PropertyType::Title,
            RichText => PropertyType::Text,
            Number { .. } => PropertyType::Number,
            Select { .. } => PropertyType::Select,
            MultiSelect { .. } => PropertyType::MultiSelect,
            Status { .. } => PropertyType::Status,
            Date => PropertyType::Date,
            People => PropertyType::Person,
            Files => PropertyType::Files,
            Checkbox => PropertyType::Checkbox,
            Url => PropertyType::Url,
            Email => PropertyType::Email,
            PhoneNumber => PropertyType::Phone,
            Formula { .. } => PropertyType::Formula,
            Relation { .. } => PropertyType::Relation,
            Rollup { .. } => PropertyType::Rollup,
            CreatedTime => PropertyType::CreatedTime,
            CreatedBy => PropertyType::CreatedBy,
            LastEditedTime => PropertyType::LastEditedTime,
            LastEditedBy => PropertyType::LastEditedBy,
        }
    }
}
