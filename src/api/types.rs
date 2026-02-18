// src/api/types.rs
//! Type definitions for the Notion API module.
//!
//! This module defines immutable types for API operations,
//! following data-oriented design principles.

use crate::types::{NotionId, Warning};
use serde::Deserialize;

// --- Fetch Context Types ---

/// Immutable context for recursive fetching operations.
#[derive(Debug, Clone)]
pub struct FetchContext {
    /// Set of already visited IDs to prevent cycles (persistent for cheap cloning)
    visited: im::HashSet<NotionId>,
    /// Remaining recursion depth
    pub depth_remaining: u8,
    /// Remaining item limit
    pub items_remaining: u32,
    /// Always fetch child databases regardless of depth
    pub always_fetch_databases: bool,
}

impl FetchContext {
    /// Creates a new fetch context with the given limits.
    pub fn new(max_depth: u8, max_items: u32) -> Self {
        Self::with_options(max_depth, max_items, false)
    }

    /// Creates a new fetch context with options.
    pub fn with_options(max_depth: u8, max_items: u32, always_fetch_databases: bool) -> Self {
        // Clamp depth to prevent stack overflow
        let safe_depth = max_depth.min(crate::constants::NOTION_MAX_FETCH_DEPTH);
        if max_depth > safe_depth {
            log::warn!(
                "Requested recursion depth {} exceeds maximum safe depth {}. Clamping to safe value.",
                max_depth, safe_depth
            );
        }

        Self {
            visited: im::HashSet::new(),
            depth_remaining: safe_depth,
            items_remaining: max_items,
            always_fetch_databases,
        }
    }

    /// Returns a new context with the given ID marked as visited.
    pub fn with_visited(self, id: NotionId) -> Self {
        let mut visited = self.visited;
        visited.insert(id);
        Self { visited, ..self }
    }

    /// Returns a new context with decremented depth.
    pub fn with_decremented_depth(self) -> Self {
        Self {
            depth_remaining: self.depth_remaining.saturating_sub(1),
            ..self
        }
    }

    /// Returns a new context with decremented item count.
    pub fn with_items_used(self, count: u32) -> Self {
        Self {
            items_remaining: self.items_remaining.saturating_sub(count),
            ..self
        }
    }

    /// Checks if we should continue fetching.
    pub fn should_fetch(&self, id: &NotionId) -> bool {
        !self.visited.contains(id) && self.depth_remaining > 0 && self.items_remaining > 0
    }
}

/// Result of a fetch operation with metadata.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FetchResult<T> {
    /// The fetched data
    pub data: T,
    /// Updated context after the fetch
    pub context: FetchContext,
    /// Metadata about the fetch operation
    pub metadata: FetchMetadata,
}

impl<T> FetchResult<T> {
    /// Maps the data while preserving context and metadata.
    #[allow(dead_code)]
    pub fn map<U, F>(self, f: F) -> FetchResult<U>
    where
        F: FnOnce(T) -> U,
    {
        FetchResult {
            data: f(self.data),
            context: self.context,
            metadata: self.metadata,
        }
    }
}

/// Metadata about a fetch operation.
#[derive(Debug, Clone, Default)]
pub struct FetchMetadata {
    /// Number of items fetched in this operation
    pub items_fetched: u32,
    /// Maximum depth reached
    pub max_depth_reached: u8,
    /// Links discovered during fetching
    pub links_found: Vec<DiscoveredLink>,
    /// Warnings generated during fetch
    pub warnings: Vec<Warning>,
}

impl FetchMetadata {
    /// Combines two metadata instances.
    pub fn merge(self, other: Self) -> Self {
        Self {
            items_fetched: self.items_fetched + other.items_fetched,
            max_depth_reached: self.max_depth_reached.max(other.max_depth_reached),
            links_found: [self.links_found, other.links_found].concat(),
            warnings: [self.warnings, other.warnings].concat(),
        }
    }
}

/// A discovered link during content traversal.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiscoveredLink {
    pub id: NotionId,
    pub link_type: LinkType,
    pub origin: LinkOrigin,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum LinkType {
    Page,
    Database,
    Block,
    Unknown,
}

/// Where a link was discovered in the content tree.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum LinkOrigin {
    LinkToPageBlock,
    ChildDatabaseBlock,
    RichTextMention,
    EmbedBlock,
}

// --- API Request Types ---

/// Hint about what type of Notion object an ID refers to.
///
/// Detected from URL structure to avoid wasting API calls trying
/// the wrong endpoint first.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectTypeHint {
    /// URL contains `?v=` — likely a database view
    Database,
    /// No URL clues available
    Unknown,
}

impl ObjectTypeHint {
    /// Detects a type hint from a raw Notion URL or ID string.
    pub fn from_input(input: &str) -> Self {
        // Database views have a `?v=` query parameter
        if input.contains("?v=") || input.contains("&v=") {
            return ObjectTypeHint::Database;
        }
        ObjectTypeHint::Unknown
    }
}

/// What we intend to accomplish by fetching this object.
/// Determines priority, follow-up work, and depth handling.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchObjective {
    /// Fetch a page/database and recursively explore its children
    ExploreRecursively { type_hint: ObjectTypeHint },
    /// Fetch a child database's structure and query its rows
    ResolveChildDatabase { source_block_id: NotionId },
}

/// Configuration for a fetch request.
#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub id: NotionId,
    pub objective: FetchObjective,
}

// --- API Response Types ---

/// Generic paginated response from Notion API.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PaginatedResponse<T> {
    pub object: String,
    pub results: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Error response from Notion API.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct NotionApiErrorResponse {
    pub code: String,
    pub message: String,
}

/// Types of Notion objects for type determination.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum NotionObjectType {
    Page,
    Database,
    Block,
}

// --- Pagination Types ---

/// Result of a pagination operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub total_fetched: usize,
}

/// Request for paginated data.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PaginationRequest {
    pub page_size: u32,
    pub max_pages: Option<u32>,
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            page_size: 100,
            max_pages: None,
        }
    }
}
