// src/api/cache.rs
//! Disk-based response cache for Notion API calls.
//!
//! Caches raw JSON responses keyed by endpoint, with TTL-based expiry.
//! On cache hit, the cached JSON is re-parsed through the same parsers
//! used for live API responses — the domain model is never serialized.

use super::client::{extract_response_text, ApiResponse, NotionHttpClient};
use super::parser;
use crate::constants::NOTION_API_PAGE_SIZE;
use crate::error::AppError;
use crate::model::{Block, Database, Page};
use crate::types::NotionId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Disk cache
// ---------------------------------------------------------------------------

/// TTL-based file cache for raw API response strings.
///
/// Cache operations are best-effort: read/write failures are silently
/// ignored so a broken cache never prevents fresh API calls.
pub struct DiskCache {
    cache_dir: PathBuf,
    ttl_secs: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    data: String,
    cached_at: u64,
}

impl DiskCache {
    /// Creates a new cache in `$XDG_CACHE_HOME/notion2prompt` (or `~/.cache/notion2prompt`).
    ///
    /// On creation, expired entries are purged to prevent unbounded disk growth.
    pub async fn new(ttl_secs: u64) -> Result<Self, std::io::Error> {
        let cache_dir = Self::default_cache_dir();
        tokio::fs::create_dir_all(&cache_dir).await?;
        let cache = Self {
            cache_dir,
            ttl_secs,
        };
        cache.purge_expired().await;
        Ok(cache)
    }

    fn default_cache_dir() -> PathBuf {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".cache")
            })
            .join("notion2prompt")
    }

    /// Returns cached data if the entry exists and has not expired.
    pub async fn get(&self, key: &str) -> Option<String> {
        let path = self.key_to_path(key);
        let content = tokio::fs::read_to_string(&path).await.ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        if now.saturating_sub(entry.cached_at) > self.ttl_secs {
            let _ = tokio::fs::remove_file(&path).await;
            return None;
        }
        Some(entry.data)
    }

    /// Stores data in the cache. Errors are silently ignored (cache is best-effort).
    pub async fn set(&self, key: &str, data: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = CacheEntry {
            data: data.to_string(),
            cached_at: now,
        };
        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = tokio::fs::write(self.key_to_path(key), json).await;
        }
    }

    /// Removes all expired cache entries from disk.
    ///
    /// Called automatically on construction to prevent unbounded disk growth.
    /// Errors are silently ignored — a broken purge never blocks operation.
    async fn purge_expired(&self) {
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => return,
        };

        let mut dir = match tokio::fs::read_dir(&self.cache_dir).await {
            Ok(d) => d,
            Err(_) => return,
        };

        while let Ok(Some(entry)) = dir.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Ok(cached) = serde_json::from_str::<CacheEntry>(&content) {
                    if now.saturating_sub(cached.cached_at) > self.ttl_secs {
                        let _ = tokio::fs::remove_file(&path).await;
                    }
                }
            }
        }
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        self.cache_dir
            .join(format!("{:016x}.json", hasher.finish()))
    }
}

// ---------------------------------------------------------------------------
// Cached Notion client
// ---------------------------------------------------------------------------

/// A [`NotionRepository`](super::NotionRepository) implementation that caches
/// raw API JSON responses to disk. Cache hits re-parse through the standard
/// parsers so the domain model is never serialized/deserialized directly.
pub struct CachedNotionClient {
    inner: NotionHttpClient,
    cache: DiskCache,
}

impl CachedNotionClient {
    /// Wraps an existing HTTP client with a disk cache.
    ///
    /// `ttl_secs` controls how long cached responses are considered valid.
    /// Expired entries are purged on construction.
    pub async fn new(inner: NotionHttpClient, ttl_secs: u64) -> Result<Self, AppError> {
        let cache = DiskCache::new(ttl_secs)
            .await
            .map_err(|e| AppError::InternalError {
                message: format!("Failed to initialize disk cache: {}", e),
                source: None,
            })?;
        Ok(Self { inner, cache })
    }

    /// Performs a cached GET returning the raw response text.
    async fn cached_get(
        &self,
        cache_key: &str,
        endpoint: &str,
    ) -> Result<ApiResponse<String>, AppError> {
        if let Some(cached) = self.cache.get(cache_key).await {
            log::debug!("Cache hit: {}", cache_key);
            return Ok(ApiResponse {
                data: cached,
                status: reqwest::StatusCode::OK,
                url: format!("cache://{}", cache_key),
            });
        }

        log::debug!("Cache miss: {}", cache_key);
        let response = self.inner.get(endpoint).await?;
        let api_response = extract_response_text(response).await?;

        if api_response.status.is_success() {
            self.cache.set(cache_key, &api_response.data).await;
        }

        Ok(api_response)
    }

    /// Fetches all pages for a GET-paginated endpoint, caching the raw
    /// response text for each page as a JSON array.
    async fn cached_get_paginated_blocks(
        &self,
        cache_key: &str,
        base_endpoint: &str,
    ) -> Result<Vec<Block>, AppError> {
        // Check for cached full result
        if let Some(cached) = self.cache.get(cache_key).await {
            log::debug!("Cache hit: {}", cache_key);
            let raw_pages: Vec<String> =
                serde_json::from_str(&cached).map_err(|e| AppError::InternalError {
                    message: format!("Cache deserialization failed for {}: {}", cache_key, e),
                    source: None,
                })?;
            let mut all_blocks = Vec::new();
            for raw in raw_pages {
                let api_resp = ApiResponse {
                    data: raw,
                    status: reqwest::StatusCode::OK,
                    url: String::new(),
                };
                let parsed = parser::parse_blocks_pagination(api_resp)?;
                all_blocks.extend(parsed.results);
            }
            return Ok(all_blocks);
        }

        // Fetch all pages, collecting raw JSON for caching
        log::debug!("Cache miss: {}", cache_key);
        let mut all_blocks = Vec::new();
        let mut raw_responses = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let endpoint = match &cursor {
                Some(c) => format!(
                    "{}?page_size={}&start_cursor={}",
                    base_endpoint, NOTION_API_PAGE_SIZE, c
                ),
                None => format!("{}?page_size={}", base_endpoint, NOTION_API_PAGE_SIZE),
            };

            let response = self.inner.get(&endpoint).await?;
            let api_response = extract_response_text(response).await?;
            raw_responses.push(api_response.data.clone());

            let parsed = parser::parse_blocks_pagination(api_response)?;
            let has_more = parsed.has_more;
            cursor = parsed.next_cursor;
            all_blocks.extend(parsed.results);

            if !has_more || cursor.is_none() {
                break;
            }
        }

        // Cache the raw page responses
        if let Ok(cache_data) = serde_json::to_string(&raw_responses) {
            self.cache.set(cache_key, &cache_data).await;
        }

        Ok(all_blocks)
    }

    /// Fetches all pages for a POST-paginated endpoint (database queries),
    /// caching the raw response text for each page as a JSON array.
    async fn cached_post_paginated_pages(
        &self,
        cache_key: &str,
        endpoint: &str,
    ) -> Result<Vec<Page>, AppError> {
        // Check for cached full result
        if let Some(cached) = self.cache.get(cache_key).await {
            log::debug!("Cache hit: {}", cache_key);
            let raw_pages: Vec<String> =
                serde_json::from_str(&cached).map_err(|e| AppError::InternalError {
                    message: format!("Cache deserialization failed for {}: {}", cache_key, e),
                    source: None,
                })?;
            let mut all_pages = Vec::new();
            for raw in raw_pages {
                let api_resp = ApiResponse {
                    data: raw,
                    status: reqwest::StatusCode::OK,
                    url: String::new(),
                };
                let parsed = parser::parse_pages_pagination(api_resp)?;
                all_pages.extend(parsed.results);
            }
            super::client::sort_pages_by_date_desc(&mut all_pages);
            return Ok(all_pages);
        }

        // Fetch all pages with raw response collection
        log::debug!("Cache miss: {}", cache_key);
        let mut all_pages = Vec::new();
        let mut raw_responses = Vec::new();
        let mut cursor: Option<String> = None;
        let page_size = u32::try_from(NOTION_API_PAGE_SIZE).unwrap_or(100);

        loop {
            let mut query = serde_json::json!({
                "page_size": page_size,
            });
            if let Some(ref c) = cursor {
                query["start_cursor"] = serde_json::json!(c);
            }

            let response = self.inner.post(endpoint, &query).await?;
            let api_response = extract_response_text(response).await?;
            raw_responses.push(api_response.data.clone());

            let parsed = parser::parse_pages_pagination(api_response)?;
            let has_more = parsed.has_more;
            cursor = parsed.next_cursor;
            all_pages.extend(parsed.results);

            if !has_more || cursor.is_none() {
                break;
            }
        }

        super::client::sort_pages_by_date_desc(&mut all_pages);

        // Cache the raw page responses
        if let Ok(cache_data) = serde_json::to_string(&raw_responses) {
            self.cache.set(cache_key, &cache_data).await;
        }

        Ok(all_pages)
    }
}

#[async_trait::async_trait]
impl super::NotionRepository for CachedNotionClient {
    async fn retrieve_page(&self, id: &NotionId) -> Result<Page, AppError> {
        let cache_key = format!("page_{}", id.as_str());
        let endpoint = format!("pages/{}", id.to_hyphenated());
        let result = self.cached_get(&cache_key, &endpoint).await?;
        parser::parse_page_response(result)
    }

    async fn retrieve_database(&self, id: &NotionId) -> Result<Database, AppError> {
        let cache_key = format!("db_{}", id.as_str());
        let endpoint = format!("databases/{}", id.to_hyphenated());
        let result = self.cached_get(&cache_key, &endpoint).await?;
        parser::parse_database_response(result)
    }

    async fn retrieve_block(&self, id: &NotionId) -> Result<Block, AppError> {
        let cache_key = format!("block_{}", id.as_str());
        let endpoint = format!("blocks/{}", id.to_hyphenated());
        let result = self.cached_get(&cache_key, &endpoint).await?;
        parser::parse_block_response(result)
    }

    async fn retrieve_children(&self, parent: &NotionId) -> Result<Vec<Block>, AppError> {
        let cache_key = format!("children_{}", parent.as_str());
        let base_endpoint = format!("blocks/{}/children", parent.to_hyphenated());
        self.cached_get_paginated_blocks(&cache_key, &base_endpoint)
            .await
    }

    async fn query_rows(&self, database: &NotionId) -> Result<Vec<Page>, AppError> {
        let cache_key = format!("rows_{}", database.as_str());
        let endpoint = format!("databases/{}/query", database.to_hyphenated());
        self.cached_post_paginated_pages(&cache_key, &endpoint)
            .await
    }
}
