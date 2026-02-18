// src/formatting/databases/types.rs
//! Type definitions for table structures.
//!
//! This module defines the core types for representing tables,
//! separating structure from rendering concerns.

use crate::types::PropertyName;
use std::fmt;

// --- Table Structure ---

/// Represents a table's structure and content in a type-safe manner.
#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<Column>,
    pub pages: Vec<TableRow>,
    #[allow(dead_code)]
    pub metadata: TableMetadata,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    /// Creates a new empty table.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            pages: Vec::new(),
            metadata: TableMetadata::default(),
        }
    }

    /// Returns the number of columns.
    #[allow(dead_code)]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns the number of rows.
    #[allow(dead_code)]
    pub fn row_count(&self) -> usize {
        self.pages.len()
    }

    /// Checks if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() || self.pages.is_empty()
    }
}

/// Metadata about a table.
#[derive(Debug, Clone, Default)]
pub struct TableMetadata {
    #[allow(dead_code)]
    pub has_title_column: bool,
    #[allow(dead_code)]
    pub has_links: bool,
    #[allow(dead_code)]
    pub total_cells: usize,
}

/// Represents a table column with metadata.
#[derive(Debug, Clone)]
pub struct Column {
    pub name: PropertyName,
    pub property_type: PropertyType,
    pub alignment: ColumnAlignment,
    #[allow(dead_code)]
    pub width_hint: Option<usize>,
}

/// Type-safe representation of property types for columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Title,
    #[allow(dead_code)]
    Text,
    Number,
    Select,
    MultiSelect,
    Date,
    Person,
    Files,
    Checkbox,
    Url,
    Email,
    Phone,
    Formula,
    Relation,
    Rollup,
    CreatedTime,
    CreatedBy,
    LastEditedTime,
    LastEditedBy,
    #[allow(dead_code)]
    Status,
    #[allow(dead_code)]
    UniqueId,
    #[allow(dead_code)]
    Verification,
}

impl PropertyType {
    /// Determines column alignment based on property type.
    pub fn default_alignment(&self) -> ColumnAlignment {
        match self {
            PropertyType::Number => ColumnAlignment::Right,
            PropertyType::Date | PropertyType::CreatedTime | PropertyType::LastEditedTime => {
                ColumnAlignment::Center
            }
            PropertyType::Checkbox => ColumnAlignment::Center,
            _ => ColumnAlignment::Left,
        }
    }

    /// Returns a display name for the property type.
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            PropertyType::Title => "Title",
            PropertyType::Text => "Text",
            PropertyType::Number => "Number",
            PropertyType::Select => "Select",
            PropertyType::MultiSelect => "Multi-select",
            PropertyType::Date => "Date",
            PropertyType::Person => "Person",
            PropertyType::Files => "Files",
            PropertyType::Checkbox => "Checkbox",
            PropertyType::Url => "URL",
            PropertyType::Email => "Email",
            PropertyType::Phone => "Phone",
            PropertyType::Formula => "Formula",
            PropertyType::Relation => "Relation",
            PropertyType::Rollup => "Rollup",
            PropertyType::CreatedTime => "Created time",
            PropertyType::CreatedBy => "Created by",
            PropertyType::LastEditedTime => "Last edited time",
            PropertyType::LastEditedBy => "Last edited by",
            PropertyType::Status => "Status",
            PropertyType::UniqueId => "ID",
            PropertyType::Verification => "Verification",
        }
    }
}

/// Column alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    Left,
    #[allow(dead_code)]
    Center,
    #[allow(dead_code)]
    Right,
}

impl ColumnAlignment {
    /// Converts to Markdown alignment syntax.
    pub fn to_markdown(self) -> &'static str {
        match self {
            ColumnAlignment::Left => "---",
            ColumnAlignment::Center => ":---:",
            ColumnAlignment::Right => "---:",
        }
    }
}

impl fmt::Display for ColumnAlignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_markdown())
    }
}

/// Represents a single row in the table.
#[derive(Debug, Clone)]
pub struct TableRow {
    #[allow(dead_code)]
    pub page_id: String,
    pub cells: Vec<TableCell>,
}

impl TableRow {
    /// Creates a new row with the given page ID.
    pub fn new(page_id: String) -> Self {
        Self {
            page_id,
            cells: Vec::new(),
        }
    }

    /// Adds a cell to the row.
    pub fn with_cell(mut self, cell: TableCell) -> Self {
        self.cells.push(cell);
        self
    }
}

/// Represents a single cell in the table.
#[derive(Debug, Clone)]
pub struct TableCell {
    pub value: CellValue,
    pub metadata: CellMetadata,
}

impl TableCell {
    /// Creates a new cell with the given value.
    pub fn new(value: CellValue) -> Self {
        Self {
            value,
            metadata: CellMetadata::default(),
        }
    }

    /// Creates a new cell with text content.
    #[allow(dead_code)]
    pub fn text(text: impl Into<String>) -> Self {
        Self::new(CellValue::Text(text.into()))
    }

    /// Creates a new cell with a link.
    #[allow(dead_code)]
    pub fn link(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self::new(CellValue::Link {
            text: text.into(),
            url: url.into(),
        })
    }

    /// Creates an empty cell.
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self::new(CellValue::Empty)
    }
}

/// Cell metadata.
#[derive(Debug, Clone, Default)]
pub struct CellMetadata {
    #[allow(dead_code)]
    pub is_title: bool,
    #[allow(dead_code)]
    pub is_primary: bool,
    #[allow(dead_code)]
    pub column_span: usize,
}

/// Type-safe representation of cell values.
#[derive(Debug, Clone)]
pub enum CellValue {
    Text(String),
    Link { text: String, url: String },
    Empty,
}

impl CellValue {
    /// Checks if the cell is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }
}
