// src/config.rs
use crate::error::AppError;
use crate::types::{ApiKey, NotionId, TemplateName};
use clap::Parser;
use std::default::Default;
use std::path::PathBuf;

/// Creates a unique temporary content directory.
fn create_temp_content_dir() -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    std::env::temp_dir().join(format!("notion_fetcher_{}", timestamp))
}

/// Parsed and validated command-line input.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineInput {
    /// Notion page/database URL or ID (e.g., "https://www.notion.so/...")
    pub notion_input: String,

    /// Directory to store intermediate content (defaults to temp dir)
    #[arg(short = 'd', long)]
    pub content_dir: Option<String>,

    /// Output file for the final prompt (optional)
    #[arg(short, long)]
    pub output_file: Option<String>,

    /// Copy the generated prompt to the clipboard
    #[arg(short = 'b', long, default_value_t = true)]
    // Changed default to true for convenience
    pub clipboard: bool,

    /// Enable verbose logging (debug level)
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Use specific template name (e.g., 'claude-xml', 'default') excluding .hbs extension. Defaults to 'claude-xml'.
    #[arg(long, default_value = "claude-xml")]
    pub template: String,

    /// Custom instruction text to include in the prompt
    #[arg(long)]
    pub instruction: Option<String>,

    /// Pipe mode - output prompt directly to stdout for piping
    #[arg(short = 'p', long, default_value_t = false)]
    pub pipe: bool,

    /// Maximum recursion depth when fetching Notion content (0 = no recursion)
    #[arg(long, default_value_t = 5)]
    pub depth: u8,

    /// Maximum number of items (pages, blocks) to fetch
    #[arg(long, default_value_t = 1000)]
    pub limit: u32,

    /// Parse and include child pages when encountered (default true)
    #[arg(long, default_value_t = true)]
    pub parse_child_pages: bool,

    /// Create separate sections for child pages in the markdown output
    #[arg(long, default_value_t = true)]
    pub separate_child_page: bool,

    /// Always fetch child databases regardless of depth limit
    #[arg(long, default_value_t = false)]
    pub always_fetch_databases: bool,

    /// Include Properties section in the output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub include_properties: bool,

    /// Exclude Properties section from the output
    #[arg(long = "no-properties", action = clap::ArgAction::SetTrue)]
    pub no_properties: bool,

    /// Disable response caching (always fetch fresh data)
    #[arg(long, default_value_t = false)]
    pub no_cache: bool,

    /// Cache TTL in seconds (default: 300 = 5 minutes)
    #[arg(long, default_value_t = 300)]
    pub cache_ttl: u64,

    /// Number of concurrent API workers (default: auto, max 32)
    #[arg(long)]
    pub concurrency: Option<usize>,
}

/// Resolved pipeline configuration — validated and ready to drive all three stages.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub notion_id: NotionId,
    pub api_key: ApiKey,
    pub depth: u8,
    pub limit: u32,
    pub template: TemplateName,
    #[allow(dead_code)] // Used by bin crate
    pub content_dir: PathBuf,
    pub output_file: Option<PathBuf>,
    pub clipboard: bool,
    pub pipe: bool,
    #[allow(dead_code)] // Used by bin crate
    pub verbose: bool,
    pub always_fetch_databases: bool,
    pub include_properties: bool,
    pub instruction: Option<String>,
    pub no_cache: bool,
    pub cache_ttl: u64,
    pub concurrency: Option<usize>,
    /// The raw URL/input string — preserved for type-hint detection.
    pub raw_input: String,
}

impl PipelineConfig {
    /// Resolves a complete pipeline configuration from CLI input and environment.
    pub fn resolve(cli: CommandLineInput) -> Result<Self, AppError> {
        let api_key_str = std::env::var("NOTION_API_KEY").map_err(|_| {
            AppError::MissingConfiguration(
                "NOTION_API_KEY environment variable not set".to_string(),
            )
        })?;

        let api_key = ApiKey::new(api_key_str)?;
        let notion_id = NotionId::parse(&cli.notion_input)?;

        let content_dir_base = cli
            .content_dir
            .map(PathBuf::from)
            .unwrap_or_else(create_temp_content_dir);

        let notion_specific_content_dir = content_dir_base.join(notion_id.value_hyphenated());
        let template = TemplateName::new(cli.template)?;

        Ok(PipelineConfig {
            notion_id,
            api_key,
            content_dir: notion_specific_content_dir,
            output_file: cli.output_file.map(PathBuf::from),
            clipboard: cli.clipboard,
            template,
            instruction: cli.instruction,
            pipe: cli.pipe,
            verbose: cli.verbose,
            depth: cli.depth,
            limit: cli.limit,
            always_fetch_databases: cli.always_fetch_databases,
            include_properties: cli.include_properties && !cli.no_properties,
            no_cache: cli.no_cache,
            cache_ttl: cli.cache_ttl,
            concurrency: cli.concurrency,
            raw_input: cli.notion_input,
        })
    }

    /// Returns the full template path.
    pub fn get_template_path(&self) -> PathBuf {
        PathBuf::from("./templates").join(format!("{}.hbs", self.template.as_str()))
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            notion_id: Self::example_notion_id(),
            api_key: ApiKey::new("secret_default_key_for_testing_only")
                .expect("Default API key should be valid"),
            depth: 10,
            limit: 1000,
            template: TemplateName::new("default").expect("Default template name should be valid"),
            content_dir: PathBuf::from(".content"),
            output_file: None,
            clipboard: false,
            pipe: false,
            verbose: false,
            always_fetch_databases: false,
            include_properties: true,
            instruction: None,
            no_cache: false,
            cache_ttl: 300,
            concurrency: None,
            raw_input: String::new(),
        }
    }
}

impl PipelineConfig {
    /// Returns a valid example NotionId for use in tests and defaults.
    fn example_notion_id() -> NotionId {
        NotionId::parse("12345678123456781234567812345678")
            .expect("Example NotionId should always be valid")
    }
}
