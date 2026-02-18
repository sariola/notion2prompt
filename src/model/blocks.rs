use super::common::BlockCommon;
use crate::types::{BlockId, Color, PageId, RichTextItem};
use serde::{Deserialize, Serialize};

/// Text content block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBlockContent {
    pub rich_text: Vec<RichTextItem>,
    pub color: Color,
}

impl Default for TextBlockContent {
    fn default() -> Self {
        Self {
            rich_text: Vec::new(),
            color: Color::Default,
        }
    }
}

/// Paragraph block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ParagraphBlock {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Heading 1 block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading1Block {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Heading 2 block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading2Block {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Heading 3 block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading3Block {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Bulleted list item block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulletedListItemBlock {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Numbered list item block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberedListItemBlock {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Toggle block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleBlock {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// To-do block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ToDoBlock {
    pub common: BlockCommon,
    pub content: TextBlockContent,
    pub checked: bool,
}

/// Quote block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteBlock {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Callout block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalloutBlock {
    pub common: BlockCommon,
    pub icon: Option<Icon>,
    pub content: TextBlockContent,
}

/// Icon types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Icon {
    #[serde(rename = "emoji")]
    Emoji { emoji: String },
    #[serde(rename = "external")]
    External { external: ExternalFile },
    #[serde(rename = "file")]
    File { file: NotionFile },
}

/// Code block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeBlock {
    pub common: BlockCommon,
    pub language: String,
    pub caption: Vec<RichTextItem>,
    pub content: TextBlockContent,
}

/// Equation block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquationBlock {
    pub common: BlockCommon,
    pub expression: String,
}

/// Divider block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DividerBlock {
    pub common: BlockCommon,
}

/// Breadcrumb block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreadcrumbBlock {
    pub common: BlockCommon,
}

/// Table of contents block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableOfContentsBlock {
    pub common: BlockCommon,
}

/// Image block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageBlock {
    pub common: BlockCommon,
    pub image: FileObject,
    pub caption: Vec<RichTextItem>,
}

/// Video block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoBlock {
    pub common: BlockCommon,
    pub video: FileObject,
    pub caption: Vec<RichTextItem>,
}

/// File block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileBlock {
    pub common: BlockCommon,
    pub file: FileObject,
    pub caption: Vec<RichTextItem>,
}

/// PDF block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfBlock {
    pub common: BlockCommon,
    pub pdf: FileObject,
    pub caption: Vec<RichTextItem>,
}

/// Bookmark block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookmarkBlock {
    pub common: BlockCommon,
    pub url: String,
    pub caption: Vec<RichTextItem>,
}

/// Embed block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedBlock {
    pub common: BlockCommon,
    pub url: String,
}

/// Child page block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChildPageBlock {
    pub common: BlockCommon,
    pub title: String,
}

/// The resolution state of a child database's content.
///
/// A child database block in Notion can reference either an inline database
/// (whose data is directly fetchable) or a linked database (a read-only view
/// that the Notion API cannot retrieve). This type makes that distinction
/// explicit so every consumer handles each case meaningfully.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ChildDatabaseContent {
    /// Database was fetched successfully with schema and rows.
    Fetched(Box<super::Database>),
    /// This is a linked database — a view of another database.
    /// The Notion API does not support retrieving linked databases directly.
    LinkedDatabase,
    /// The database could not be accessed (permissions, not found, etc.)
    Inaccessible { reason: String },
    /// Fetch has not been attempted (initial state from block parsing).
    NotFetched,
}

impl Default for ChildDatabaseContent {
    fn default() -> Self {
        Self::NotFetched
    }
}

impl ChildDatabaseContent {
    /// Returns a reference to the database if it was successfully fetched.
    #[allow(dead_code)]
    pub fn as_database(&self) -> Option<&super::Database> {
        match self {
            Self::Fetched(db) => Some(db),
            _ => None,
        }
    }
}

/// Child database block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChildDatabaseBlock {
    pub common: BlockCommon,
    pub title: String,
    /// The resolution state of this child database reference.
    pub content: ChildDatabaseContent,
}

/// Link to page block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkToPageBlock {
    pub common: BlockCommon,
    pub page_id: PageId,
}

/// Table block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableBlock {
    pub common: BlockCommon,
    pub table_width: usize,
    pub has_column_header: bool,
    pub has_row_header: bool,
}

/// Table row block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRowBlock {
    pub common: BlockCommon,
    pub cells: Vec<Vec<RichTextItem>>,
}

/// Column list block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnListBlock {
    pub common: BlockCommon,
}

/// Column block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnBlock {
    pub common: BlockCommon,
}

/// Synced block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncedBlock {
    pub common: BlockCommon,
    pub synced_from: Option<SyncedFrom>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncedFrom {
    pub block_id: BlockId,
}

/// Template block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateBlock {
    pub common: BlockCommon,
    pub content: TextBlockContent,
}

/// Link preview block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkPreviewBlock {
    pub common: BlockCommon,
    pub url: String,
}

/// Unsupported block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnsupportedBlock {
    pub common: BlockCommon,
    pub block_type: String,
}

/// File object types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FileObject {
    #[serde(rename = "external")]
    External { external: ExternalFile },
    #[serde(rename = "file")]
    File { file: NotionFile },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalFile {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionFile {
    pub url: String,
    pub expiry_time: Option<chrono::DateTime<chrono::Utc>>,
}
