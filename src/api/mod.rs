// src/api/mod.rs
//! Notion API interaction — the ability to retrieve content from a workspace.
//!
//! This module provides a data-oriented interface to the Notion API,
//! with clear separation between I/O operations, parsing, and business logic.

pub mod cache;
pub mod client;
mod concurrent_queue;
mod connection_pool;
mod fetch_queue;
pub mod notion_client_adapter;
pub mod object_graph;
mod parallel_fetcher;
pub mod parser;
mod responses;
mod simple_pagination;
mod types;

use crate::error::AppError;
use crate::model::{Block, Database, Page};
use crate::types::NotionId;

/// The ability to retrieve content from a Notion workspace.
///
/// This is the fundamental algebra for API interaction.
/// Business logic depends on this trait, never on HTTP details.
#[async_trait::async_trait]
pub trait NotionRepository: Send + Sync {
    async fn retrieve_page(&self, id: &NotionId) -> Result<Page, AppError>;
    async fn retrieve_database(&self, id: &NotionId) -> Result<Database, AppError>;
    async fn retrieve_block(&self, id: &NotionId) -> Result<Block, AppError>;
    async fn retrieve_children(&self, parent: &NotionId) -> Result<Vec<Block>, AppError>;
    async fn query_rows(&self, database: &NotionId) -> Result<Vec<Page>, AppError>;

    /// Resolves an object by trying page, then database, then block.
    async fn resolve_object(&self, id: &NotionId) -> Result<crate::model::NotionObject, AppError> {
        use crate::model::NotionObject;

        // Try page first (most common)
        if let Ok(page) = self.retrieve_page(id).await {
            return Ok(NotionObject::Page(page));
        }

        // Try database
        if let Ok(db) = self.retrieve_database(id).await {
            return Ok(NotionObject::Database(db));
        }

        // Try block (last resort)
        match self.retrieve_block(id).await {
            Ok(block) => Ok(NotionObject::Block(block)),
            Err(_) => Err(AppError::InvalidId(format!(
                "Could not determine type for ID: {} (object not found or access denied)",
                id.as_str()
            ))),
        }
    }
}

// Re-export the public interface
#[allow(unused_imports)]
pub use cache::CachedNotionClient;
pub use client::NotionHttpClient;
pub use parallel_fetcher::NotionFetcher;
