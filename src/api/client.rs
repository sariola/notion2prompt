// src/api/client.rs
//! Pure HTTP client wrapper for Notion API.
//!
//! This module provides a thin wrapper around reqwest for making
//! HTTP requests to the Notion API. It handles authentication and
//! basic request/response operations without parsing or business logic.

use crate::error::AppError;
use crate::types::ApiKey;
use reqwest::{header, Client, Response};
use serde::Serialize;

const NOTION_VERSION: &str = "2022-06-28";
const API_BASE_URL: &str = "https://api.notion.com/v1";

/// A thin wrapper around reqwest Client for Notion API requests.
#[derive(Clone)]
pub struct NotionHttpClient {
    client: Client,
}

impl NotionHttpClient {
    /// Creates a new HTTP client with Notion API authentication.
    pub fn new(api_key: &ApiKey) -> Result<Self, AppError> {
        let client = Client::builder()
            .default_headers(Self::create_headers(api_key)?)
            .build()?;
        Ok(Self { client })
    }

    /// Creates the default headers for Notion API requests.
    fn create_headers(api_key: &ApiKey) -> Result<header::HeaderMap, AppError> {
        let mut headers = header::HeaderMap::new();

        let auth_header = format!("Bearer {}", api_key.as_str());
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_header).map_err(|e| {
                AppError::MissingConfiguration(format!("Invalid API token format: {}", e))
            })?,
        );

        headers.insert(
            "Notion-Version",
            header::HeaderValue::from_static(NOTION_VERSION),
        );

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        Ok(headers)
    }

    /// Makes a GET request to the specified endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The API endpoint path (without base URL)
    ///
    /// # Returns
    ///
    /// A `Response` from the Notion API, or an `AppError` if the request fails.
    pub async fn get(&self, endpoint: &str) -> Result<Response, AppError> {
        let url = format!("{}/{}", API_BASE_URL, endpoint);

        // Enhanced logging for database-related requests
        if endpoint.contains("databases") {
            log::info!("🌐 HTTP GET DATABASE: {}", url);
        } else {
            log::debug!("GET {}", url);
        }

        let response = self.client.get(url).send().await?;

        // Log response status for database requests
        if endpoint.contains("databases") {
            log::info!(
                "📡 DATABASE GET RESPONSE: {} (status: {})",
                endpoint,
                response.status()
            );
        }

        Ok(response)
    }

    /// Makes a POST request with JSON body to the specified endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The API endpoint path (without base URL)
    /// * `body` - The request body to serialize as JSON
    ///
    /// # Returns
    ///
    /// A `Response` from the Notion API, or an `AppError` if the request fails.
    pub async fn post<T: Serialize>(&self, endpoint: &str, body: &T) -> Result<Response, AppError> {
        let url = format!("{}/{}", API_BASE_URL, endpoint);

        // Enhanced logging for database queries
        if endpoint.contains("databases") && endpoint.contains("query") {
            log::info!("🔍 HTTP POST DATABASE QUERY: {}", url);
            log::info!(
                "   Query body: {}",
                serde_json::to_string_pretty(body)
                    .unwrap_or_else(|_| "Failed to serialize".to_string())
            );
        } else {
            log::debug!("POST {}", url);
        }

        let response = self.client.post(url).json(body).send().await?;

        // Log response status for database queries
        if endpoint.contains("databases") && endpoint.contains("query") {
            log::info!(
                "📊 DATABASE QUERY RESPONSE: {} (status: {})",
                endpoint,
                response.status()
            );
        }

        Ok(response)
    }

    /// Makes a PATCH request with JSON body to the specified endpoint.
    #[allow(dead_code)]
    pub async fn patch<T: Serialize>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<Response, AppError> {
        let url = format!("{}/{}", API_BASE_URL, endpoint);
        log::debug!("PATCH {}", url);
        Ok(self.client.patch(url).json(body).send().await?)
    }
}

#[async_trait::async_trait]
impl super::NotionRepository for NotionHttpClient {
    async fn retrieve_page(
        &self,
        id: &crate::types::NotionId,
    ) -> Result<crate::model::Page, AppError> {
        let endpoint = format!("pages/{}", id.to_hyphenated());
        let response = self.get(&endpoint).await?;
        let result = extract_response_text(response).await?;
        super::parser::parse_page_response(result)
    }

    async fn retrieve_database(
        &self,
        id: &crate::types::NotionId,
    ) -> Result<crate::model::Database, AppError> {
        let endpoint = format!("databases/{}", id.to_hyphenated());
        let response = self.get(&endpoint).await?;
        let result = extract_response_text(response).await?;
        super::parser::parse_database_response(result)
    }

    async fn retrieve_block(
        &self,
        id: &crate::types::NotionId,
    ) -> Result<crate::model::Block, AppError> {
        let endpoint = format!("blocks/{}", id.to_hyphenated());
        let response = self.get(&endpoint).await?;
        let result = extract_response_text(response).await?;
        super::parser::parse_block_response(result)
    }

    async fn retrieve_children(
        &self,
        parent: &crate::types::NotionId,
    ) -> Result<Vec<crate::model::Block>, AppError> {
        let endpoint = format!("blocks/{}/children", parent.to_hyphenated());
        let client = self.clone();
        let pagination_result = super::simple_pagination::fetch_all_pages_simple(
            |page_size, cursor| {
                let client = client.clone();
                let endpoint = endpoint.clone();
                async move {
                    let mut query = serde_json::json!({ "page_size": page_size });
                    if let Some(cursor) = cursor {
                        query["start_cursor"] = serde_json::json!(cursor);
                    }
                    let response = client.get(&endpoint).await?;
                    let result = extract_response_text(response).await?;
                    super::parser::parse_blocks_pagination(result)
                }
            },
            None,
        )
        .await?;
        Ok(pagination_result.items)
    }

    async fn query_rows(
        &self,
        database: &crate::types::NotionId,
    ) -> Result<Vec<crate::model::Page>, AppError> {
        let endpoint = format!("databases/{}/query", database.to_hyphenated());
        let client = self.clone();
        let pagination_result = super::simple_pagination::fetch_all_pages_simple(
            |page_size, cursor| {
                let client = client.clone();
                let endpoint = endpoint.clone();
                async move {
                    let mut query = serde_json::json!({
                        "page_size": page_size
                    });
                    if let Some(cursor) = cursor {
                        query["start_cursor"] = serde_json::json!(cursor);
                    }
                    let response = client.post(&endpoint, &query).await?;
                    let result = extract_response_text(response).await?;
                    super::parser::parse_pages_pagination(result)
                }
            },
            None,
        )
        .await?;
        let mut pages = pagination_result.items;
        sort_pages_by_date_desc(&mut pages);
        Ok(pages)
    }
}

/// Sorts pages by their first date-like property, newest first.
/// Pages without a date value sort to the bottom.
///
/// Extracts dates from native Date properties, Rollup dates, Formula dates,
/// and timestamp properties (CreatedTime, LastEditedTime).
pub(super) fn sort_pages_by_date_desc(pages: &mut [crate::model::Page]) {
    use crate::model::PropertyTypeValue;

    /// Attempts to extract a NaiveDate from any property type.
    fn extract_date(val: &PropertyTypeValue) -> Option<chrono::NaiveDate> {
        match val {
            PropertyTypeValue::Date { date: Some(d) } => Some(d.start),
            PropertyTypeValue::Rollup {
                rollup: crate::types::RollupResult::Date { date: Some(d) },
            } => Some(d.start),
            PropertyTypeValue::Rollup {
                rollup: crate::types::RollupResult::Array { array },
            } => array.iter().find_map(|item| match item {
                crate::types::RollupArrayItem::Date(d) => Some(d.start),
                _ => None,
            }),
            PropertyTypeValue::Formula {
                formula: crate::types::FormulaResult::Date(d),
            } => Some(d.start),
            PropertyTypeValue::CreatedTime { created_time } => Some(created_time.date_naive()),
            PropertyTypeValue::LastEditedTime { last_edited_time } => {
                Some(last_edited_time.date_naive())
            }
            _ => None,
        }
    }

    // Find the first property name that has a date-like value,
    // preferring native Date properties over rollup/formula/timestamp.
    let date_prop = pages
        .iter()
        .find_map(|page| {
            page.properties.iter().find_map(|(name, val)| {
                if matches!(
                    val.type_specific_value,
                    PropertyTypeValue::Date { date: Some(_) }
                ) {
                    Some(name.clone())
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            // Fallback: any property that yields a date
            pages.iter().find_map(|page| {
                page.properties.iter().find_map(|(name, val)| {
                    if extract_date(&val.type_specific_value).is_some() {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
            })
        });

    let Some(date_prop) = date_prop else {
        return;
    };

    pages.sort_by(|a, b| {
        let get_date = |page: &crate::model::Page| {
            page.properties
                .get(&date_prop)
                .and_then(|v| extract_date(&v.type_specific_value))
        };
        match (get_date(a), get_date(b)) {
            (Some(a), Some(b)) => b.cmp(&a), // descending: newest first
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
}

/// Result of an HTTP operation with response metadata.
#[derive(Debug)]
pub struct ApiResponse<T> {
    pub data: T,
    pub status: reqwest::StatusCode,
    pub url: String,
}

/// Extracts the response body as text with metadata.
///
/// # Arguments
///
/// * `response` - The HTTP response to extract text from
///
/// # Returns
///
/// An `ApiResponse<String>` containing the response text along with status and URL metadata.
pub async fn extract_response_text(response: Response) -> Result<ApiResponse<String>, AppError> {
    let status = response.status();
    let url = response.url().to_string();
    let text = response.text().await?;

    Ok(ApiResponse {
        data: text,
        status,
        url,
    })
}
