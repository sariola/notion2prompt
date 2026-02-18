//! Python wrapper types for notion2prompt domain objects.

use notion2prompt::{
    ApiKey, Block, Database, NotionId, NotionObject, Page, PipelineConfig, TemplateName,
};
use pyo3::prelude::*;

/// Configuration for the notion2prompt pipeline.
///
/// Mirrors PipelineConfig but with Python-friendly construction.
#[pyclass(name = "PipelineConfig")]
#[derive(Clone)]
pub struct PyPipelineConfig {
    pub(crate) inner: PipelineConfig,
}

#[pymethods]
impl PyPipelineConfig {
    /// Create a new pipeline configuration.
    ///
    /// Args:
    ///     notion_id: Notion page/database URL or 32-char hex ID
    ///     api_key: Notion API key (starts with "secret_" or "ntn_")
    ///     depth: Maximum recursion depth (default 5)
    ///     limit: Maximum items to fetch (default 1000)
    ///     template: Template name (default "claude-xml")
    ///     always_fetch_databases: Always fetch child databases regardless of depth
    ///     include_properties: Include properties section in output
    ///     instruction: Custom instruction text for the prompt
    ///     no_cache: Disable response caching
    ///     cache_ttl: Cache TTL in seconds (default 300)
    ///     concurrency: Number of concurrent API workers
    #[new]
    #[pyo3(signature = (
        notion_id,
        api_key,
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
    fn new(
        notion_id: &str,
        api_key: &str,
        depth: u8,
        limit: u32,
        template: &str,
        always_fetch_databases: bool,
        include_properties: bool,
        instruction: Option<String>,
        no_cache: bool,
        cache_ttl: u64,
        concurrency: Option<usize>,
    ) -> PyResult<Self> {
        let notion_id = NotionId::parse(notion_id).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid Notion ID: {}", e))
        })?;

        let api_key = ApiKey::new(api_key).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid API key: {}", e))
        })?;

        let template = TemplateName::new(template).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid template name: {}", e))
        })?;

        let content_dir =
            std::env::temp_dir().join(format!("notion2prompt_py_{}", notion_id.value_hyphenated()));

        Ok(Self {
            inner: PipelineConfig {
                notion_id,
                api_key,
                depth,
                limit,
                template,
                content_dir,
                output_file: None,
                clipboard: false,
                pipe: false,
                verbose: false,
                always_fetch_databases,
                include_properties,
                instruction,
                no_cache,
                cache_ttl,
                concurrency,
                raw_input: String::new(),
            },
        })
    }

    #[getter]
    fn depth(&self) -> u8 {
        self.inner.depth
    }

    #[getter]
    fn limit(&self) -> u32 {
        self.inner.limit
    }

    #[getter]
    fn template(&self) -> &str {
        self.inner.template.as_str()
    }

    #[getter]
    fn always_fetch_databases(&self) -> bool {
        self.inner.always_fetch_databases
    }

    #[getter]
    fn include_properties(&self) -> bool {
        self.inner.include_properties
    }

    fn __repr__(&self) -> String {
        format!(
            "PipelineConfig(notion_id='{}', depth={}, limit={}, template='{}')",
            self.inner.notion_id.as_str(),
            self.inner.depth,
            self.inner.limit,
            self.inner.template.as_str(),
        )
    }
}

/// Fetched Notion content — a page, database, or block.
///
/// This is an opaque handle to the Rust NotionObject that can be
/// passed to `render_content()` for template rendering.
#[pyclass(name = "NotionContent")]
#[derive(Clone)]
pub struct PyNotionContent {
    pub(crate) inner: NotionObject,
}

#[pymethods]
impl PyNotionContent {
    /// The type of the content: "page", "database", or "block".
    #[getter]
    fn content_type(&self) -> &str {
        match &self.inner {
            NotionObject::Page(_) => "page",
            NotionObject::Database(_) => "database",
            NotionObject::Block(_) => "block",
        }
    }

    /// The display title of the content.
    #[getter]
    fn title(&self) -> String {
        self.inner.display_title().to_string()
    }

    /// Serialize the content to a JSON string.
    fn to_json(&self) -> PyResult<String> {
        match &self.inner {
            NotionObject::Page(page) => page_to_json(page),
            NotionObject::Database(db) => database_to_json(db),
            NotionObject::Block(block) => block_to_json(block),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "NotionContent(type='{}', title='{}')",
            self.content_type(),
            self.title(),
        )
    }
}

fn page_to_json(page: &Page) -> PyResult<String> {
    let val = serde_json::json!({
        "type": "page",
        "id": page.id.as_str(),
        "title": page.title().as_str(),
        "url": page.url,
        "blocks_count": page.blocks.len(),
        "properties_count": page.properties.len(),
    });
    serde_json::to_string_pretty(&val)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

fn database_to_json(db: &Database) -> PyResult<String> {
    let val = serde_json::json!({
        "type": "database",
        "id": db.id.as_str(),
        "title": db.title().as_plain_text(),
        "url": db.url,
        "pages_count": db.pages.len(),
        "properties_count": db.properties.len(),
    });
    serde_json::to_string_pretty(&val)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

fn block_to_json(block: &Block) -> PyResult<String> {
    let val = serde_json::json!({
        "type": "block",
        "id": block.id().as_str(),
        "block_type": block.block_type(),
    });
    serde_json::to_string_pretty(&val)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Resolve a PipelineConfig from environment and arguments.
///
/// Reads NOTION_API_KEY from environment if api_key is None.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_config(
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
) -> PyResult<PipelineConfig> {
    let api_key_str = match api_key {
        Some(key) => key.to_string(),
        None => std::env::var("NOTION_API_KEY").map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(
                "NOTION_API_KEY not set. Pass api_key= or set the environment variable.",
            )
        })?,
    };

    let notion_id = NotionId::parse(notion_id).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Invalid Notion ID: {}", e))
    })?;

    let api_key = ApiKey::new(&api_key_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid API key: {}", e)))?;

    let template = TemplateName::new(template)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid template: {}", e)))?;

    let content_dir =
        std::env::temp_dir().join(format!("notion2prompt_py_{}", notion_id.value_hyphenated()));

    Ok(PipelineConfig {
        notion_id,
        api_key,
        depth,
        limit,
        template,
        content_dir,
        output_file: None,
        clipboard: false,
        pipe: false,
        verbose: false,
        always_fetch_databases,
        include_properties,
        instruction,
        no_cache,
        cache_ttl,
        concurrency,
        raw_input: String::new(),
    })
}
