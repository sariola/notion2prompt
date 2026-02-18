// src/lib.rs
//! notion2prompt library — converts Notion pages and databases into structured prompts.
//!
//! # Public API
//!
//! The library exposes types organized by concern:
//! - **Error handling** — `AppError`, `ValidationError`
//! - **Configuration** — `PipelineConfig`
//! - **Domain model** — `NotionObject`, `Page`, `Database`, `Block`, etc.
//! - **Domain types** — `NotionId`, `ApiKey`, `BlockId`, `PageId`, etc.
//! - **API client** — `NotionFetcher`, `NotionHttpClient`, parsers
//! - **Formatting** — `render_blocks`, `RenderContext`, `TableBuilder`

// Internal modules — must match what's in main.rs
mod analytics;
#[cfg(feature = "bench")]
pub mod api;
#[cfg(not(feature = "bench"))]
mod api;

mod algebras;
mod config;
mod constants;
mod error;
mod error_recovery;

#[cfg(feature = "bench")]
pub mod formatting;
#[cfg(not(feature = "bench"))]
mod formatting;

#[cfg(feature = "bench")]
pub mod model;
#[cfg(not(feature = "bench"))]
mod model;

mod output;
mod pipeline;

#[cfg(feature = "bench")]
pub mod types;
#[cfg(not(feature = "bench"))]
mod types;

// --- Error Handling ---
pub use crate::error::{AppError, DatabaseFetchFailure};
pub use crate::types::ValidationError;

// --- Configuration ---
pub use crate::config::PipelineConfig;

// --- Domain Model ---
pub use crate::model::{
    Block, BlockCommon, BlockVisitor, Database, DatabaseProperty, DatabasePropertyType,
    DatabaseTitle, NotionObject, NumberFormat, Page, PageTitle, Parent, PropertyTypeValue,
    PropertyValue,
};

// --- Block Types ---
pub use crate::model::blocks::{
    BookmarkBlock, BreadcrumbBlock, BulletedListItemBlock, CalloutBlock, ChildDatabaseBlock,
    ChildDatabaseContent, ChildPageBlock, CodeBlock, ColumnBlock, ColumnListBlock, DividerBlock,
    EmbedBlock, EquationBlock, ExternalFile, FileBlock, FileObject, Heading1Block, Heading2Block,
    Heading3Block, Icon, ImageBlock, LinkPreviewBlock, LinkToPageBlock, NotionFile,
    NumberedListItemBlock, ParagraphBlock, PdfBlock, QuoteBlock, SyncedBlock, SyncedFrom,
    TableBlock, TableOfContentsBlock, TableRowBlock, TemplateBlock, TextBlockContent, ToDoBlock,
    ToggleBlock, UnsupportedBlock, VideoBlock,
};

// --- Domain Types ---
pub use crate::types::{
    Annotations, ApiKey, BlockId, Color, DatabaseId, DateValue, EquationData, FormulaResult, Link,
    MentionData, MentionType, NotionId, PageId, PartialUser, PropertyName, RenderedPrompt,
    RichTextItem, RichTextType, RollupResult, SelectOption, TemplateName, TextContent, User,
    UserId, ValidatedUrl, Warning, WarningLevel,
};

// --- API Client ---
pub use crate::api::{
    cache::CachedNotionClient,
    client::ApiResponse,
    object_graph::ObjectGraph,
    parser::{
        parse_block_response, parse_blocks_pagination, parse_database_response,
        parse_page_response, parse_pages_pagination,
    },
    NotionFetcher, NotionHttpClient, NotionRepository,
};

// --- Formatting ---
pub use crate::formatting::block_renderer::{
    compose_block_markdown, compose_database_summary, compose_notion_markdown,
    compose_page_markdown, render_blocks, RenderContext,
};
pub use crate::formatting::databases::builder::TableBuilder;
pub use crate::formatting::direct_template::render_prompt;

// --- Pipeline Traits ---
pub use crate::pipeline::{ContentSource, PromptComposer, PromptDelivery};

// --- Algebras (Capability Traits) ---
pub use crate::algebras::{
    DepthLimiter, FetchError, NotionContent, NotionContentExt, TrackError, VisitTracker,
};
