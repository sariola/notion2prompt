// src/formatting/direct_template.rs
//! Composes prompts directly from in-memory NotionObject structures.
//!
//! This module renders templates directly from NotionObject data,
//! bypassing file intermediates to preserve child database content.

use crate::config::PipelineConfig;
use crate::error::AppError;
use crate::formatting::block_renderer::RenderContext;
use crate::model::{Database, NotionObject, Page};
use crate::output::create_clean_filename;
use crate::types::RenderedPrompt;
use handlebars::Handlebars;
use serde_json::json;

/// Template data structure for direct rendering.
#[allow(dead_code)] // Used by bin target via compose_prompt
#[derive(Debug, Clone)]
pub struct PromptContent {
    pub absolute_content_path: String,
    pub source_tree: Option<String>,
    pub files: Vec<RenderedDocument>,
    pub main_content: String,
    pub instructions: Option<String>,
}

/// File data structure for template rendering.
#[allow(dead_code)] // Used by bin target via compose_prompt
#[derive(Debug, Clone)]
pub struct RenderedDocument {
    pub path: String,
    pub code: String,
}

/// Renders a NotionObject into a prompt string using Handlebars templates.
#[allow(dead_code)] // Used by bin target (main.rs)
pub fn render_prompt(
    notion_object: &NotionObject,
    config: &PipelineConfig,
) -> Result<RenderedPrompt, AppError> {
    log::info!("Composing prompt via direct template rendering");

    let template_engine = load_template(config)?;
    let content = gather_renderable_content(notion_object, config)?;
    let prompt = render_with_template(&template_engine, &content, config)?;

    log::info!(
        "Prompt composed: {} bytes from {} files",
        prompt.len(),
        content.files.len()
    );

    Ok(RenderedPrompt::new(prompt))
}

// --- render_prompt helpers ---

/// Loads and registers a Handlebars template from the configured path.
fn load_template(config: &PipelineConfig) -> Result<Handlebars<'static>, AppError> {
    let template_path = config.get_template_path();
    let template_content =
        std::fs::read_to_string(&template_path).map_err(|e| AppError::TemplateNotFound {
            path: template_path.display().to_string(),
            source: e,
        })?;

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(config.template.as_str(), &template_content)
        .map_err(|e| AppError::TemplateRenderError {
            name: config.template.to_string(),
            message: e.to_string(),
        })?;

    Ok(handlebars)
}

/// Gathers all renderable content from a NotionObject tree.
fn gather_renderable_content(
    notion_object: &NotionObject,
    config: &PipelineConfig,
) -> Result<PromptContent, AppError> {
    let databases = crate::formatting::gather_embedded_databases(notion_object);

    log::debug!("Template data: {} databases available", databases.len());

    let render_config = RenderContext {
        app_config: Some(config),
        databases: Some(&databases),
    };

    let mut files = Vec::new();
    let main_content = collect_rendered_files(notion_object, &mut files, &render_config)?;

    let source_tree = build_source_tree(&files);

    Ok(PromptContent {
        absolute_content_path: "/direct_template".to_string(),
        source_tree: Some(source_tree),
        files,
        main_content,
        instructions: config.instruction.clone(),
    })
}

/// Renders the final prompt by applying the template engine to the prepared content.
fn render_with_template(
    handlebars: &Handlebars,
    data: &PromptContent,
    config: &PipelineConfig,
) -> Result<String, AppError> {
    let template_name = config.template.as_str();

    let json_data = json!({
        "absolute_content_path": data.absolute_content_path,
        "source_tree": data.source_tree,
        "files": data.files.iter().map(|f| json!({
            "path": f.path,
            "code": f.code
        })).collect::<Vec<_>>(),
        "main_content": data.main_content,
        "instructions": data.instructions
    });

    handlebars
        .render(template_name, &json_data)
        .map_err(|e| AppError::TemplateRenderError {
            name: config.template.to_string(),
            message: e.to_string(),
        })
}

// --- Content rendering ---

/// Recursively renders file data from a NotionObject tree.
fn collect_rendered_files(
    notion_object: &NotionObject,
    files: &mut Vec<RenderedDocument>,
    render_config: &RenderContext,
) -> Result<String, AppError> {
    match notion_object {
        NotionObject::Page(page) => {
            let content = render_page_content(page, render_config)?;
            let filename = clean_filename(page.title().as_str(), page.id.as_str());

            files.push(RenderedDocument {
                path: filename,
                code: content.clone(),
            });

            Ok(content)
        }
        NotionObject::Database(db) => {
            let content = render_database_content(db, render_config)?;
            let title = db.title().as_plain_text();
            let filename = clean_filename(&title, db.id.as_str());

            files.push(RenderedDocument {
                path: filename,
                code: content.clone(),
            });

            for page in &db.pages {
                let page_obj = NotionObject::Page(page.clone());
                collect_rendered_files(&page_obj, files, render_config)?;
            }

            Ok(content)
        }
        NotionObject::Block(block) => {
            let content = render_block_content(block, render_config)?;
            let filename = format!("block_{}.md", block.id().as_str());

            files.push(RenderedDocument {
                path: filename,
                code: content.clone(),
            });

            Ok(content)
        }
    }
}

fn render_page_content(page: &Page, render_config: &RenderContext) -> Result<String, AppError> {
    crate::formatting::block_renderer::compose_page_markdown(page, render_config)
}

fn render_database_content(
    db: &Database,
    _render_config: &RenderContext,
) -> Result<String, AppError> {
    crate::formatting::block_renderer::compose_database_summary(db)
}

fn render_block_content(
    block: &crate::model::Block,
    render_config: &RenderContext,
) -> Result<String, AppError> {
    crate::formatting::block_renderer::compose_block_markdown(block, render_config)
}

// --- Helpers ---

/// Creates a clean filename from a title and ID, using the shared path utility.
fn clean_filename(title: &str, id: &str) -> String {
    create_clean_filename(title, id, false)
}

fn build_source_tree(files: &[RenderedDocument]) -> String {
    let mut tree = String::new();
    tree.push_str("direct_template/\n");
    for file in files {
        tree.push_str(&format!("└── {}\n", file.path));
    }
    tree
}
