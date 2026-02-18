// src/formatting/properties/types.rs
//! Domain types for formatted properties.
//!
//! This module defines the core types for representing formatted property values,
//! separating data representation from rendering concerns.

use chrono::{DateTime, Utc};
use std::fmt;

// --- Core Formatted Types ---

/// Represents a formatted property value with its semantic type preserved.
#[derive(Debug, Clone, PartialEq)]
pub enum FormattedProperty {
    Text(String),
    Number(NumberValue),
    Boolean(bool),
    Date(DateRange),
    Select(String),
    MultiSelect(Vec<String>),
    Status(String),
    People(Vec<String>),
    Files(Vec<FileLink>),
    Url(UrlLink),
    Email(String),
    Phone(String),
    Formula(FormulaValue),
    Relation(FormattedRelation),
    Rollup(RollupValue),
    CreatedTime(DateTime<Utc>),
    LastEditedTime(DateTime<Utc>),
    CreatedBy(String),
    LastEditedBy(String),
    UniqueId(String),
    Verification(VerificationValue),
    #[allow(dead_code)]
    List(Vec<FormattedProperty>),
    Empty,
}

/// Represents a numeric value with formatting metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue {
    pub value: f64,
    pub format: NumberFormat,
}

impl NumberValue {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            format: NumberFormat::Auto,
        }
    }

    #[allow(dead_code)]
    pub fn with_format(value: f64, format: NumberFormat) -> Self {
        Self { value, format }
    }
}

/// Number formatting options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberFormat {
    Auto,
    #[allow(dead_code)]
    Integer,
    #[allow(dead_code)]
    Decimal(u8), // Number of decimal places
    #[allow(dead_code)]
    Percentage,
    #[allow(dead_code)]
    Currency(CurrencyFormat),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrencyFormat {
    pub symbol: &'static str,
    pub position: CurrencyPosition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrencyPosition {
    #[allow(dead_code)]
    Prefix,
    #[allow(dead_code)]
    Suffix,
}

/// Represents a date or date range.
#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    pub start: String,
    pub end: Option<String>,
}

impl fmt::Display for DateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.end {
            Some(end) => write!(f, "{} → {}", self.start, end),
            None => write!(f, "{}", self.start),
        }
    }
}

/// Represents a file with name and URL.
#[derive(Debug, Clone, PartialEq)]
pub struct FileLink {
    pub name: String,
    pub url: String,
}

/// Represents a URL with optional display text.
#[derive(Debug, Clone, PartialEq)]
pub struct UrlLink {
    pub url: String,
    pub text: Option<String>,
}

/// Represents a formula result value.
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaValue {
    String(String),
    Number(NumberValue),
    Boolean(bool),
    Date(String),
}

/// Represents a formatted relation value.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattedRelation {
    pub ids: Vec<String>,
    pub has_more: bool,
}

/// Represents a rollup value.
#[derive(Debug, Clone, PartialEq)]
pub enum RollupValue {
    Number(NumberValue),
    Date(String),
    Array(Vec<String>),
    String(String),
    Boolean(bool),
    Unsupported,
    Incomplete,
}

/// Represents a verification value.
#[derive(Debug, Clone, PartialEq)]
pub struct VerificationValue {
    pub state: String,
    pub verified_by: Option<String>,
}

// --- Property Metadata ---

/// Metadata about a property for formatting decisions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PropertyMetadata {
    pub property_id: String,
    pub property_type: String,
    pub is_title: bool,
    pub is_primary: bool,
}

// --- Trait Definitions ---

/// Trait for types that can be rendered to different formats.
pub trait Renderable {
    /// Renders to plain text.
    fn render_text(&self) -> String;

    /// Renders to Markdown.
    fn render_markdown(&self) -> String {
        self.render_text() // Default implementation
    }

    /// Renders to HTML (escaped).
    #[allow(dead_code)]
    fn render_html(&self) -> String {
        html_escape(&self.render_text()) // Default implementation
    }
}

// --- Utility Functions ---

/// Basic HTML escaping.
#[allow(dead_code)]
fn html_escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
