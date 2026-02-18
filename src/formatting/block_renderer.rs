// src/formatting/block_renderer.rs
//! Block rendering engine — converts Notion blocks to markdown.
//!
//! This module implements the recursive rendering logic for Notion blocks
//! using a data-oriented approach with immutable state transitions.

use super::pure_visitor::MarkdownBlockRenderer;
use super::state::FormatContext;
use crate::config::PipelineConfig;
use crate::constants::CHARS_PER_BLOCK_ESTIMATE;
use crate::error::AppError;
use crate::model::{Block, Database, NotionObject, Page};
use std::fmt::Write;

// --- Core Types ---

/// Something that can resolve a database by its ID.
pub trait DatabaseResolver {
    fn find_database(&self, id: &crate::types::NotionId) -> Option<&Database>;
}

impl DatabaseResolver for std::collections::HashMap<crate::types::NotionId, Database> {
    fn find_database(&self, id: &crate::types::NotionId) -> Option<&Database> {
        self.get(id)
    }
}

/// Context passed through the rendering pipeline.
#[derive(Clone, Default)]
pub struct RenderContext<'a> {
    pub app_config: Option<&'a PipelineConfig>,
    /// Resolver for looking up child databases during rendering
    pub databases: Option<&'a dyn DatabaseResolver>,
}

impl std::fmt::Debug for RenderContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderContext")
            .field("app_config", &self.app_config)
            .field("databases", &self.databases.is_some())
            .finish()
    }
}

// --- Public API ---

/// Renders a slice of blocks into markdown.
pub fn render_blocks(blocks: &[Block], config: &RenderContext) -> Result<String, AppError> {
    let formatter = MarkdownBlockRenderer::with_document_blocks(config, blocks);
    let initial_context = FormatContext::new();

    let estimated_capacity = blocks.len() * CHARS_PER_BLOCK_ESTIMATE;
    let mut output = String::with_capacity(estimated_capacity);

    let mut context = initial_context;
    let mut list_context_stack = Vec::new();

    log::debug!(
        "Rendering {} blocks (databases available: {})",
        blocks.len(),
        config.databases.is_some()
    );

    for (i, block) in blocks.iter().enumerate() {
        let is_list_item = matches!(
            block,
            Block::NumberedListItem(_) | Block::BulletedListItem(_)
        );
        let next_is_list = i + 1 < blocks.len()
            && matches!(
                blocks[i + 1],
                Block::NumberedListItem(_) | Block::BulletedListItem(_)
            );

        if is_list_item
            && (i == 0
                || !matches!(
                    blocks[i - 1],
                    Block::NumberedListItem(_) | Block::BulletedListItem(_)
                ))
        {
            match block {
                Block::NumberedListItem(_) => {
                    list_context_stack.push(context.clone());
                    context = context.enter_numbered_list();
                }
                Block::BulletedListItem(_) => {
                    list_context_stack.push(context.clone());
                    context = context.enter_bulleted_list();
                }
                _ => {}
            }
        }

        let result = formatter.format_with_context(block, context)?;

        output.push_str(&result.content);
        context = result.context;

        if is_list_item && !next_is_list {
            if let Some(saved_context) = list_context_stack.pop() {
                context = saved_context;
            }
        }
    }

    Ok(output)
}

// --- Object-Level Rendering ---

/// Composes a Notion page into markdown: title, properties, content, metadata.
pub fn compose_page_markdown(page: &Page, config: &RenderContext) -> Result<String, AppError> {
    let title = compose_title_section(page);
    let properties = compose_properties_section(page, config)?;
    let content = compose_content_section(page, config)?;
    let metadata = compose_metadata_section(page);
    Ok([title, properties, content, metadata].concat())
}

fn compose_title_section(page: &Page) -> String {
    format!("# {}\n\n", page.title().as_str())
}

fn compose_properties_section(page: &Page, config: &RenderContext) -> Result<String, AppError> {
    let include_properties = config
        .app_config
        .map(|cfg| cfg.include_properties)
        .unwrap_or(true);

    if !include_properties {
        return Ok(String::new());
    }

    let mut out = String::from("## Properties\n\n");
    for (key, value) in &page.properties {
        if matches!(
            value.type_specific_value,
            crate::model::PropertyTypeValue::Title { .. }
        ) {
            continue;
        }

        let formatted = super::properties::render_property_value(Some(value))?;
        if !formatted.is_empty() {
            writeln!(out, "- **{}**: {}", key, formatted)?;
        }
    }
    out.push('\n');
    Ok(out)
}

fn compose_content_section(page: &Page, config: &RenderContext) -> Result<String, AppError> {
    if page.blocks.is_empty() {
        return Ok(String::new());
    }
    let blocks_md = render_blocks(&page.blocks, config)?;
    log::debug!(
        "Rendered page '{}': {} bytes",
        page.title().as_str(),
        blocks_md.len()
    );
    Ok(format!("{}\n", blocks_md))
}

fn compose_metadata_section(page: &Page) -> String {
    format!(
        "## Metadata\n\n- **Page ID**: {}\n- **URL**: {}\n",
        page.id.as_str(),
        page.url
    )
}

/// Composes a database summary as markdown: title, schema, data count, metadata.
pub fn compose_database_summary(db: &Database) -> Result<String, AppError> {
    let mut content = String::new();
    let title = db.title().as_plain_text();

    writeln!(content, "# {}", title)?;
    writeln!(content)?;

    writeln!(content, "## Schema")?;
    writeln!(content)?;

    if !db.properties.is_empty() {
        writeln!(content, "| Property | Type |")?;
        writeln!(content, "|----------|------|")?;

        for (name, schema) in &db.properties {
            writeln!(
                content,
                "| {} | {} |",
                name.as_str().replace('|', "\\|"),
                schema.property_type
            )?;
        }
        writeln!(content)?;
    }

    writeln!(content, "## Data")?;
    writeln!(content)?;

    if !db.pages.is_empty() {
        writeln!(content, "Database contains {} pages.", db.pages.len())?;
    } else {
        writeln!(content, "*Database has no rows.*")?;
    }

    writeln!(content, "## Metadata")?;
    writeln!(content)?;
    writeln!(content, "- **Database ID**: {}", db.id.as_str())?;

    Ok(content)
}

/// Composes a single block into markdown with a heading.
pub fn compose_block_markdown(block: &Block, config: &RenderContext) -> Result<String, AppError> {
    let block_vec = vec![block.clone()];
    let block_md = render_blocks(&block_vec, config)?;
    Ok(format!("# Block {}\n\n{}", block.id().as_str(), block_md))
}

/// Composes any NotionObject into markdown — dispatches by variant.
///
/// Uses the database summary format (page count, not full table).
/// For full database tables, use `tabulate_rows` directly.
#[allow(dead_code)]
pub fn compose_notion_markdown(
    obj: &NotionObject,
    config: &RenderContext,
) -> Result<String, AppError> {
    match obj {
        NotionObject::Page(page) => compose_page_markdown(page, config),
        NotionObject::Database(db) => compose_database_summary(db),
        NotionObject::Block(block) => compose_block_markdown(block, config),
    }
}
