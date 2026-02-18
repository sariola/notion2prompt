// src/api/parser.rs
//! Production-ready parsing using notion-client library.
//!
//! This module replaces manual JSON parsing with notion-client's validated
//! serde implementations, ensuring robust handling of all Notion API responses.

use super::client::ApiResponse;
use super::responses::{
    NotionBlock, NotionDatabase, NotionError, NotionPage, QueryDatabaseResponse,
    RetrieveBlockChildrenResponse, ToDomain,
};
use crate::error::{AppError, NotionClientError};
use crate::model::{Block, Database, NotionObject, Page};
use reqwest::StatusCode;
use serde_json::Value;

/// Parse any Notion API response using notion-client types
pub fn parse_api_response<T>(result: ApiResponse<String>) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    if result.status.is_success() {
        parse_with_notion_client(&result.data, &result.url)
    } else {
        parse_error_with_notion_client(&result.data, result.status, &result.url)
    }
}

/// Parse successful response using notion-client's robust parsing
fn parse_with_notion_client<T>(body: &str, url: &str) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(body).map_err(|e| {
        log::error!("Failed to parse response from {}: {}", url, e);

        let preview = if body.len() > 500 {
            format!("{}...", &body[..500])
        } else {
            body.to_string()
        };

        NotionClientError::Deserialization {
            source: e,
            body: preview,
        }
        .into()
    })
}

/// Parse error response using notion-client error types
fn parse_error_with_notion_client<T>(
    body: &str,
    status: StatusCode,
    url: &str,
) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    // Try to parse as NotionError first
    if let Ok(notion_error) = serde_json::from_str::<NotionError>(body) {
        return Err(NotionClientError::NotionApi {
            status: notion_error.status,
            code: notion_error.code,
            message: notion_error.message,
            request_id: notion_error.request_id,
        }
        .into());
    }

    // Fallback to generic error with HTTP status code
    Err(AppError::NotionService {
        code: crate::error::NotionErrorCode::from_http_status(status.as_u16()),
        message: format!("HTTP {} from {}", status, url),
        status,
    })
}

/// Parse page response using notion-client
pub fn parse_page_response(result: ApiResponse<String>) -> Result<Page, AppError> {
    let notion_page: NotionPage = parse_api_response(result)?;
    ToDomain::to_domain(notion_page)
}

/// Parse database response using notion-client
pub fn parse_database_response(result: ApiResponse<String>) -> Result<Database, AppError> {
    let notion_database: NotionDatabase = parse_api_response(result)?;
    ToDomain::to_domain(notion_database)
}

/// Parse block response using notion-client
pub fn parse_block_response(result: ApiResponse<String>) -> Result<Block, AppError> {
    let notion_block: NotionBlock = parse_api_response(result)?;
    ToDomain::to_domain(notion_block)
}

/// Parse any object type (page/database/block) dynamically
#[allow(dead_code)]
pub fn parse_notion_object(result: ApiResponse<String>) -> Result<NotionObject, AppError> {
    // First parse as generic JSON to check object type
    let json: Value =
        serde_json::from_str(&result.data).map_err(|e| NotionClientError::Deserialization {
            source: e,
            body: result.data.clone(),
        })?;

    let object_type = json.get("object").and_then(|v| v.as_str()).ok_or_else(|| {
        AppError::MalformedResponse("Missing 'object' field in response".to_string())
    })?;

    match object_type {
        "page" => {
            let page = parse_page_response(result)?;
            Ok(NotionObject::Page(page))
        }
        "database" => {
            let database = parse_database_response(result)?;
            Ok(NotionObject::Database(database))
        }
        "block" => {
            let block = parse_block_response(result)?;
            Ok(NotionObject::Block(block))
        }
        _ => Err(AppError::MalformedResponse(format!(
            "Unknown object type: {}",
            object_type
        ))),
    }
}

/// Parse with automatic error handling using notion-client types
#[allow(dead_code)]
pub fn parse_with_error_handling<T>(body: &str) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    // Try to parse as success first
    if let Ok(result) = serde_json::from_str::<T>(body) {
        return Ok(result);
    }

    // Try to parse as error
    if let Ok(error) = serde_json::from_str::<NotionError>(body) {
        return Err(NotionClientError::NotionApi {
            status: error.status,
            code: error.code,
            message: error.message,
            request_id: error.request_id,
        }
        .into());
    }

    // Fallback to generic parsing error
    Err(AppError::MalformedResponse(
        "Unable to parse response as success or error".to_string(),
    ))
}

/// Pagination function for pages (using notion-client) - returns PaginatedResponse
pub fn parse_pages_pagination(
    result: ApiResponse<String>,
) -> Result<super::types::PaginatedResponse<Page>, AppError> {
    let response: QueryDatabaseResponse = parse_api_response(result)?;
    let pages = response.clone().into_domain_pages()?;

    Ok(super::types::PaginatedResponse {
        object: response.object,
        results: pages,
        next_cursor: response.next_cursor,
        has_more: response.has_more,
    })
}

/// Pagination function for blocks (using notion-client) - returns PaginatedResponse
pub fn parse_blocks_pagination(
    result: ApiResponse<String>,
) -> Result<super::types::PaginatedResponse<Block>, AppError> {
    let response: RetrieveBlockChildrenResponse = parse_api_response(result)?;
    let blocks = response.clone().into_domain_blocks()?;

    Ok(super::types::PaginatedResponse {
        object: response.object,
        results: blocks,
        next_cursor: response.next_cursor,
        has_more: response.has_more,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_parsing_with_notion_client() {
        let error_json = r#"{
            "object": "error",
            "status": 404,
            "code": "object_not_found",
            "message": "Could not find page with ID: abc123",
            "request_id": "req_123"
        }"#;

        let result = parse_with_error_handling::<NotionPage>(error_json);
        assert!(result.is_err());

        if let Err(AppError::NotionClient(NotionClientError::NotionApi { code, .. })) = result {
            assert_eq!(code, "object_not_found");
        } else {
            panic!("Expected NotionClientError::NotionApi");
        }
    }
}
