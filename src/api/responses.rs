// src/api/responses.rs
//! API response types using notion-client library for production-ready parsing.
//!
//! This module replaces manual JSON parsing with notion-client's battle-tested
//! serde implementations, ensuring compatibility with all Notion API features.

use serde::{Deserialize, Serialize};

// Re-export notion-client types for production use
pub use notion_client::objects::{
    block::Block as NotionBlock, database::Database as NotionDatabase, error::Error as NotionError,
    page::Page as NotionPage,
};

/// Generic paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub object: String,
    pub results: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Database query response using notion-client types
pub type QueryDatabaseResponse = PaginatedResponse<NotionPage>;

/// Block children response using notion-client types
pub type RetrieveBlockChildrenResponse = PaginatedResponse<NotionBlock>;

/// Trait for converting notion-client types to domain types
pub trait ToDomain<T> {
    fn to_domain(self) -> Result<T, crate::error::AppError>;
}

/// Conversion implementations for seamless domain integration
impl ToDomain<crate::model::Page> for NotionPage {
    fn to_domain(self) -> Result<crate::model::Page, crate::error::AppError> {
        crate::api::notion_client_adapter::convert_page(self)
    }
}

impl ToDomain<crate::model::Database> for NotionDatabase {
    fn to_domain(self) -> Result<crate::model::Database, crate::error::AppError> {
        crate::api::notion_client_adapter::convert_database(self)
    }
}

impl ToDomain<crate::model::Block> for NotionBlock {
    fn to_domain(self) -> Result<crate::model::Block, crate::error::AppError> {
        crate::api::notion_client_adapter::convert_block(self)
    }
}

/// Batch conversion for pages
impl QueryDatabaseResponse {
    /// Convert all pages to domain model
    pub fn into_domain_pages(self) -> Result<Vec<crate::model::Page>, crate::error::AppError> {
        self.results.into_iter().map(ToDomain::to_domain).collect()
    }
}

/// Batch conversion for blocks
impl RetrieveBlockChildrenResponse {
    /// Convert all blocks to domain model
    pub fn into_domain_blocks(self) -> Result<Vec<crate::model::Block>, crate::error::AppError> {
        self.results.into_iter().map(ToDomain::to_domain).collect()
    }
}

/// Response envelope for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(T),
    Error(NotionError),
}

impl<T> ApiResponse<T> {
    /// Extract success value or convert error
    #[allow(dead_code)]
    pub fn into_result(self) -> Result<T, crate::error::AppError> {
        match self {
            ApiResponse::Success(value) => Ok(value),
            ApiResponse::Error(error) => Err(crate::error::NotionClientError::NotionApi {
                status: error.status,
                code: error.code,
                message: error.message,
                request_id: error.request_id,
            }
            .into()),
        }
    }
}

// Convenience type aliases for common response patterns
#[allow(dead_code)]
pub type PageApiResponse = ApiResponse<NotionPage>;
#[allow(dead_code)]
pub type DatabaseApiResponse = ApiResponse<NotionDatabase>;
#[allow(dead_code)]
pub type BlockApiResponse = ApiResponse<NotionBlock>;
#[allow(dead_code)]
pub type QueryApiResponse = ApiResponse<QueryDatabaseResponse>;
#[allow(dead_code)]
pub type BlockChildrenApiResponse = ApiResponse<RetrieveBlockChildrenResponse>;
