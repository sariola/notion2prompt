// src/main.rs

// Modules defined in the crate
mod analytics;
mod api;
mod config;
mod constants;
mod error;
mod error_recovery;
mod formatting;
mod model;
mod output;
mod pipeline;
mod types;

// Specific imports
use crate::analytics::{embedded_database_count, measure_content};
use crate::config::{CommandLineInput, PipelineConfig};
use crate::error::AppError;
use crate::pipeline::{ContentSource, PromptComposer, PromptDelivery};
use clap::Parser;
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Config,
};
use model::NotionObject;
use output::{deliver, DeliveryTarget, OutputReport};
use std::fs;
use types::RenderedPrompt;

/// Sets up logging configuration.
fn setup_logging(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };

    let log_file_path = std::env::temp_dir().join("notion_fetcher.log");
    if let Some(parent) = log_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let pattern = if verbose {
        "{d(%Y-%m-%d %H:%M:%S)} [{l}] - {m}{n}"
    } else {
        "{m}{n}"
    };

    let stdout_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] - {m}{n}",
        )))
        .build(&log_file_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout_appender)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Debug)))
                .build("file", Box::new(file_appender)),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(log_level),
        )?;

    log4rs::init_config(config)?;
    log::info!("Logging initialized. Log file: {}", log_file_path.display());
    Ok(())
}

/// Executes the three-stage Notion-to-prompt pipeline: fetch → render → deliver.
async fn execute_pipeline(config: &PipelineConfig) -> Result<(), AppError> {
    let pipeline = NotionToPrompt::new(config);

    let content = pipeline.fetch(&config.notion_id).await?;
    let prompt = pipeline.compose(&content)?;
    let report = pipeline.deliver(prompt)?;
    pipeline.report_completion(&content, &report)?;

    Ok(())
}

/// Orchestrates the retrieval, rendering, and delivery of Notion content as prompts.
struct NotionToPrompt<'a> {
    config: &'a PipelineConfig,
}

impl<'a> NotionToPrompt<'a> {
    fn new(config: &'a PipelineConfig) -> Self {
        Self { config }
    }

    /// Delivers the rendered prompt to configured outputs (file, clipboard, stdout).
    fn deliver_prompt(&self, prompt: RenderedPrompt) -> Result<OutputReport, AppError> {
        let prompt_str = prompt.into_string();
        let mut plan = output::OutputPlan::new();

        if self.config.pipe {
            plan = plan.with_operation(DeliveryTarget::PrintToStdout {
                content: prompt_str.clone(),
            });
        } else {
            if let Some(output_path) = &self.config.output_file {
                plan = plan.with_operation(DeliveryTarget::WriteFile {
                    path: output_path.clone(),
                    content: prompt_str.clone(),
                });
            }

            if self.config.clipboard {
                plan = plan.with_operation(DeliveryTarget::CopyToClipboard {
                    content: prompt_str,
                });
            }
        }

        let report = deliver(plan)?;

        if !report.is_success() {
            return Err(AppError::DeliveryFailed {
                failures: report.failed.iter().map(|f| f.error.clone()).collect(),
            });
        }

        Ok(report)
    }

    /// Reports completion to the user with stats and delivery confirmations.
    fn report_completion(
        &self,
        content: &NotionObject,
        report: &OutputReport,
    ) -> Result<(), AppError> {
        if self.config.pipe {
            return Ok(());
        }

        let stats = measure_content(content);

        if self.config.depth > 0 && stats.deepest_nesting >= self.config.depth as usize {
            eprintln!(
                "⚠️  Maximum recursion depth ({}) reached. Some deeply nested content may be missing.",
                self.config.depth
            );
        }

        if stats.total_objects >= self.config.limit as usize {
            eprintln!(
                "⚠️  Item limit reached ({}/{}). Some content may be missing.",
                stats.total_objects, self.config.limit
            );
        } else {
            println!("📄 Fetched {} objects from Notion.", stats.total_objects);
        }

        for completed in &report.completed {
            match &completed.operation {
                DeliveryTarget::WriteFile { path, .. } => {
                    println!("✓ Prompt saved to {}", path.display());
                }
                DeliveryTarget::CopyToClipboard { .. } => {
                    println!("✓ Prompt copied to clipboard");
                }
                _ => {}
            }
        }

        if report.completed.is_empty() && !self.config.pipe {
            println!("✓ Prompt generated (no output file or clipboard requested).");
        }

        Ok(())
    }

    /// Logs details about retrieved content.
    fn log_retrieved_content(&self, obj: &NotionObject) {
        match obj {
            NotionObject::Database(db) => {
                log::info!(
                    "  {} rows, {} properties",
                    db.pages.len(),
                    db.properties.len()
                );
            }
            NotionObject::Page(page) => {
                let db_count = embedded_database_count(&page.blocks);
                if db_count > 0 {
                    log::info!("  Contains {} child database(s)", db_count);
                }
            }
            NotionObject::Block(block) => {
                log::info!("  Block type: {}", block.block_type());
            }
        }
    }
}

#[async_trait::async_trait]
impl ContentSource for NotionToPrompt<'_> {
    async fn fetch(&self, id: &types::NotionId) -> Result<NotionObject, AppError> {
        log::info!("Retrieving content for {}", id.as_str());

        let http_client = api::NotionHttpClient::new(&self.config.api_key)?;
        let client: std::sync::Arc<dyn api::NotionRepository> = if self.config.no_cache {
            log::info!("Cache disabled — all requests go to Notion API");
            std::sync::Arc::new(http_client)
        } else {
            log::info!("Cache enabled (TTL: {}s)", self.config.cache_ttl);
            std::sync::Arc::new(
                api::CachedNotionClient::new(http_client, self.config.cache_ttl).await?,
            )
        };
        let fetcher = api::NotionFetcher::new(client, self.config);
        let result = fetcher.fetch_recursive(id).await?;

        log::info!(
            "Retrieved {} '{}' ({} items, depth {})",
            result.data.object_type_name(),
            result.data.display_title(),
            result.metadata.items_fetched,
            result.metadata.max_depth_reached,
        );
        for warning in &result.metadata.warnings {
            log::warn!("Fetch warning: {}", warning);
        }
        self.log_retrieved_content(&result.data);

        Ok(result.data)
    }
}

impl PromptComposer for NotionToPrompt<'_> {
    fn compose(&self, content: &NotionObject) -> Result<RenderedPrompt, AppError> {
        formatting::render_prompt(content, self.config)
    }
}

impl PromptDelivery for NotionToPrompt<'_> {
    fn deliver(&self, prompt: RenderedPrompt) -> Result<OutputReport, AppError> {
        self.deliver_prompt(prompt)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CommandLineInput::parse();

    setup_logging(cli.verbose)?;

    let config = PipelineConfig::resolve(cli)?;

    execute_pipeline(&config).await?;

    Ok(())
}
