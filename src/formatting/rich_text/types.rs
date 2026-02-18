// src/formatting/rich_text/types.rs
//! Type definitions for rich text representation.
//!
//! This module defines types for representing formatted rich text,
//! separating structure from rendering logic.

use crate::types::NotionId;

// --- Core Types ---

/// Represents a formatted text segment with styling.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattedText {
    pub segments: Vec<TextSegment>,
}

impl FormattedText {
    /// Creates a new empty formatted text.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Adds a segment to the formatted text.
    pub fn with_segment(mut self, segment: TextSegment) -> Self {
        self.segments.push(segment);
        self
    }
}

/// Represents a single segment of text with consistent formatting.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSegment {
    pub content: TextContent,
    pub style: TextStyle,
}

impl TextSegment {
    /// Checks if the segment is empty.
    pub fn is_empty(&self) -> bool {
        match &self.content {
            TextContent::Plain(s) => s.is_empty(),
            TextContent::Equation(e) => e.expression.is_empty(),
            TextContent::Mention(m) => m.is_empty(),
        }
    }
}

/// Content types for text segments.
#[derive(Debug, Clone, PartialEq)]
pub enum TextContent {
    Plain(String),
    Equation(EquationContent),
    Mention(MentionContent),
}

/// Equation content.
#[derive(Debug, Clone, PartialEq)]
pub struct EquationContent {
    pub expression: String,
    pub inline: bool,
}

/// Mention content types.
#[derive(Debug, Clone, PartialEq)]
pub enum MentionContent {
    User { id: String, name: String },
    Page { id: NotionId, title: String },
    Database { id: NotionId, title: String },
    Date { start: String, end: Option<String> },
    Link { url: ValidatedUrl, text: String },
}

impl MentionContent {
    /// Checks if the mention is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            MentionContent::User { name, .. } => name.is_empty(),
            MentionContent::Page { title, .. } => title.is_empty(),
            MentionContent::Database { title, .. } => title.is_empty(),
            MentionContent::Date { start, .. } => start.is_empty(),
            MentionContent::Link { text, .. } => text.is_empty(),
        }
    }
}

/// Text styling options.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
    pub color: TextColor,
    pub link: Option<ValidatedUrl>,
}

impl TextStyle {
    /// Checks if any styling is applied.
    pub fn has_styling(&self) -> bool {
        self.bold
            || self.italic
            || self.strikethrough
            || self.underline
            || self.code
            || self.color != TextColor::Default
            || self.link.is_some()
    }
}

/// Text color options.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextColor {
    #[default]
    Default,
    Gray,
    Brown,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Red,
    GrayBackground,
    BrownBackground,
    OrangeBackground,
    YellowBackground,
    GreenBackground,
    BlueBackground,
    PurpleBackground,
    PinkBackground,
    RedBackground,
}

/// Validated URL type.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedUrl {
    url: String,
    url_type: UrlType,
}

impl ValidatedUrl {
    /// Creates a new validated URL.
    pub fn parse(url: &str) -> Result<Self, UrlValidationError> {
        let url_type = determine_url_type(url)?;
        Ok(Self {
            url: url.to_string(),
            url_type,
        })
    }

    /// Gets the URL string.
    pub fn as_str(&self) -> &str {
        &self.url
    }

    /// Checks if this is a Notion URL.
    pub fn is_notion_url(&self) -> bool {
        matches!(self.url_type, UrlType::Notion)
    }
}

/// URL types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UrlType {
    Http,
    Https,
    Mailto,
    Internal,
    Notion,
}

/// URL validation error.
#[derive(Debug, Clone, PartialEq)]
pub struct UrlValidationError {
    pub url: String,
    pub reason: String,
}

impl std::fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid URL '{}': {}", self.url, self.reason)
    }
}

impl std::error::Error for UrlValidationError {}

// --- Helper Functions ---

/// Determines the type of a URL.
fn determine_url_type(url: &str) -> Result<UrlType, UrlValidationError> {
    if url.is_empty() {
        return Err(UrlValidationError {
            url: url.to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }

    if url.starts_with("https://") {
        if url.contains("notion.so/") || url.contains("notion.site/") {
            Ok(UrlType::Notion)
        } else {
            Ok(UrlType::Https)
        }
    } else if url.starts_with("http://") {
        Ok(UrlType::Http)
    } else if url.starts_with("mailto:") {
        Ok(UrlType::Mailto)
    } else if url.starts_with('/') || url.starts_with('#') {
        Ok(UrlType::Internal)
    } else if url.starts_with("notion://") {
        Ok(UrlType::Notion)
    } else {
        Err(UrlValidationError {
            url: url.to_string(),
            reason: "URL must start with a valid protocol".to_string(),
        })
    }
}
