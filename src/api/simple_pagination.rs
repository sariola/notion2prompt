// src/api/simple_pagination.rs
//! Simplified pagination without BoxFuture.

use super::types::{PaginatedResponse, PaginationResult};
use crate::constants::NOTION_API_PAGE_SIZE;
use crate::error::AppError;

/// Fetches all pages using async closures directly.
pub async fn fetch_all_pages_simple<T, F, Fut>(
    mut fetch_fn: F,
    max_pages: Option<u32>,
) -> Result<PaginationResult<T>, AppError>
where
    T: Send + 'static,
    F: FnMut(u32, Option<String>) -> Fut,
    Fut: std::future::Future<Output = Result<PaginatedResponse<T>, AppError>>,
{
    let mut all_items = Vec::new();
    let mut cursor = None;
    let mut pages_fetched = 0u32;

    loop {
        // Check if we've reached the page limit
        if let Some(max) = max_pages {
            if pages_fetched >= max {
                log::debug!("Reached maximum page limit: {}", max);
                break;
            }
        }

        // Fetch the next page
        let response = fetch_fn(NOTION_API_PAGE_SIZE as u32, cursor).await?;

        let has_more = response.has_more;
        cursor = response.next_cursor.clone();
        all_items.extend(response.results);
        pages_fetched += 1;

        if !has_more || cursor.is_none() {
            break;
        }
    }

    Ok(PaginationResult {
        total_fetched: all_items.len(),
        items: all_items,
    })
}
