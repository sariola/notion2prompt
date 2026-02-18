// src/formatting/properties/mod.rs
//! Type-safe property formatting for Notion database values.
//!
//! This module provides a structured approach to formatting Notion properties,
//! separating the concerns of data extraction, transformation, and presentation.

mod formatters;
mod render;
mod types;

// Re-export the public interface
pub use formatters::format_property_value;
pub use render::escape_for_table_cell;
pub use types::{FormattedProperty, Renderable};

use crate::error::AppError;
use crate::model::PropertyValue;

// --- Public API ---

/// Renders a property value to its markdown string representation.
pub fn render_property_value(value: Option<&PropertyValue>) -> Result<String, AppError> {
    match value {
        None => Ok(String::new()),
        Some(pv) => {
            let formatted = format_property_value(pv)?;
            Ok(formatted.render_markdown())
        }
    }
}

/// Formats a property for display in a table cell.
#[allow(dead_code)]
pub fn format_property_for_table(value: Option<&PropertyValue>) -> Result<String, AppError> {
    let rendered = render_property_value(value)?;
    Ok(escape_for_table_cell(&rendered))
}

// --- Property Formatter Trait ---

/// Trait for formatting property values in a type-safe manner.
pub trait PropertyFormatter {
    /// Formats a property value into a structured representation.
    #[allow(dead_code)]
    fn format(&self, value: Option<&PropertyValue>) -> Result<FormattedProperty, AppError>;
}

/// Default property formatter implementation.
pub struct DefaultPropertyFormatter;

impl PropertyFormatter for DefaultPropertyFormatter {
    fn format(&self, value: Option<&PropertyValue>) -> Result<FormattedProperty, AppError> {
        match value {
            None => Ok(FormattedProperty::Empty),
            Some(pv) => format_property_value(pv),
        }
    }
}

// --- Legacy Support ---

impl FormattedProperty {
    /// Checks if the property is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        matches!(self, FormattedProperty::Empty)
    }

    /// Legacy render method for backward compatibility.
    #[allow(dead_code)]
    pub fn render(&self) -> String {
        self.render_markdown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PropertyTypeValue;
    use crate::types::RichTextItem;

    #[test]
    fn test_format_empty_property() {
        let result = render_property_value(None).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_text_property() {
        let prop = PropertyValue {
            id: crate::types::PropertyName::new("test"),
            type_specific_value: PropertyTypeValue::Title {
                title: vec![RichTextItem::plain_text("Hello World")],
            },
        };

        let result = render_property_value(Some(&prop)).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_table_cell_escaping() {
        let result = escape_for_table_cell("a|b\nc|d");
        assert_eq!(result, "a\\|b<br>c\\|d");
    }
}
