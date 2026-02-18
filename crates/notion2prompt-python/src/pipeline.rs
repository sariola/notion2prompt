//! Python-exposed pipeline functions: fetch, compose, and the combined fetch_and_render.

use crate::types::{resolve_config, PyNotionContent};
use notion2prompt::{
    CachedNotionClient, NotionFetcher, NotionHttpClient, NotionObject, NotionRepository,
    PipelineConfig,
};
use pyo3::prelude::*;
use std::sync::Arc;

/// Fetch a Notion page/database and render it to a prompt string in one call.
///
/// This is the main entry point — equivalent to the CLI pipeline.
///
/// Args:
///     notion_id: Notion page/database URL or 32-char hex ID
///     api_key: Notion API key (reads NOTION_API_KEY env var if None)
///     depth: Maximum recursion depth (default 5)
///     limit: Maximum items to fetch (default 1000)
///     template: Template name (default "claude-xml")
///     always_fetch_databases: Always fetch child databases regardless of depth
///     include_properties: Include properties section in output
///     instruction: Custom instruction text for the prompt
///     no_cache: Disable response caching
///     cache_ttl: Cache TTL in seconds (default 300)
///     concurrency: Number of concurrent API workers
///
/// Returns:
///     The rendered prompt string.
#[pyfunction]
#[pyo3(signature = (
    notion_id,
    api_key = None,
    depth = 5,
    limit = 1000,
    template = "claude-xml",
    always_fetch_databases = false,
    include_properties = false,
    instruction = None,
    no_cache = false,
    cache_ttl = 300,
    concurrency = None,
))]
#[allow(clippy::too_many_arguments)]
pub fn fetch_and_render<'py>(
    py: Python<'py>,
    notion_id: &str,
    api_key: Option<&str>,
    depth: u8,
    limit: u32,
    template: &str,
    always_fetch_databases: bool,
    include_properties: bool,
    instruction: Option<String>,
    no_cache: bool,
    cache_ttl: u64,
    concurrency: Option<usize>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = resolve_config(
        notion_id,
        api_key,
        depth,
        limit,
        template,
        always_fetch_databases,
        include_properties,
        instruction,
        no_cache,
        cache_ttl,
        concurrency,
    )?;

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let content = fetch_notion_content(&config).await?;
        let prompt = compose_prompt(&content, &config)?;
        Ok(prompt)
    })
}

/// Fetch Notion content without rendering.
///
/// Returns a NotionContent handle that can be passed to render_content().
///
/// Args:
///     notion_id: Notion page/database URL or 32-char hex ID
///     api_key: Notion API key (reads NOTION_API_KEY env var if None)
///     depth: Maximum recursion depth (default 5)
///     limit: Maximum items to fetch (default 1000)
///     always_fetch_databases: Always fetch child databases regardless of depth
///     no_cache: Disable response caching
///     cache_ttl: Cache TTL in seconds (default 300)
///     concurrency: Number of concurrent API workers
#[pyfunction]
#[pyo3(signature = (
    notion_id,
    api_key = None,
    depth = 5,
    limit = 1000,
    always_fetch_databases = false,
    no_cache = false,
    cache_ttl = 300,
    concurrency = None,
))]
#[allow(clippy::too_many_arguments)]
pub fn fetch_content<'py>(
    py: Python<'py>,
    notion_id: &str,
    api_key: Option<&str>,
    depth: u8,
    limit: u32,
    always_fetch_databases: bool,
    no_cache: bool,
    cache_ttl: u64,
    concurrency: Option<usize>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = resolve_config(
        notion_id,
        api_key,
        depth,
        limit,
        "default",
        always_fetch_databases,
        false,
        None,
        no_cache,
        cache_ttl,
        concurrency,
    )?;

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let content = fetch_notion_content(&config).await?;
        Ok(PyNotionContent { inner: content })
    })
}

/// Render previously fetched content to a prompt string.
///
/// Args:
///     content: NotionContent from fetch_content()
///     template: Template name (default "claude-xml")
///     include_properties: Include properties section in output
///     instruction: Custom instruction text for the prompt
#[pyfunction]
#[pyo3(signature = (content, template = "claude-xml", include_properties = false, instruction = None))]
pub fn render_content(
    content: &PyNotionContent,
    template: &str,
    include_properties: bool,
    instruction: Option<String>,
) -> PyResult<String> {
    let template = notion2prompt::TemplateName::new(template)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid template: {}", e)))?;
    let config = PipelineConfig {
        template,
        include_properties,
        instruction,
        ..PipelineConfig::default()
    };

    compose_prompt(&content.inner, &config)
}

// --- Internal helpers ---

async fn fetch_notion_content(config: &PipelineConfig) -> PyResult<NotionObject> {
    let http_client = NotionHttpClient::new(&config.api_key).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create HTTP client: {}", e))
    })?;

    let client: Arc<dyn NotionRepository> = if config.no_cache {
        Arc::new(http_client)
    } else {
        Arc::new(
            CachedNotionClient::new(http_client, config.cache_ttl)
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to create cache: {}",
                        e
                    ))
                })?,
        )
    };

    let fetcher = NotionFetcher::new(client, config);
    let result = fetcher
        .fetch_recursive(&config.notion_id)
        .await
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Fetch failed: {}", e)))?;

    Ok(result.data)
}

fn compose_prompt(content: &NotionObject, config: &PipelineConfig) -> PyResult<String> {
    let rendered = notion2prompt::render_prompt(content, config)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Render failed: {}", e)))?;
    Ok(rendered.into_string())
}
