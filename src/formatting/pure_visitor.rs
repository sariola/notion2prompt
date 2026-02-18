// src/formatting/pure_visitor.rs
//! Pure functional visitor implementation for block formatting.
//!
//! This module provides an immutable, functional approach to visiting
//! and formatting Notion blocks, following functional programming principles.

use super::block_renderer::RenderContext;
use super::rich_text::rich_text_to_markdown;
use super::state::FormatContext;
use crate::error::AppError;
use crate::model::*;

/// Table of contents entry
#[derive(Debug, Clone)]
struct TocEntry {
    level: u8,
    text: String,
}

/// The result of rendering a single block — content plus updated context.
#[derive(Debug, Clone)]
pub struct BlockRenderResult {
    pub content: String,
    pub context: FormatContext,
}

/// Trait for formatting blocks into output strings.
pub trait BlockRenderer {
    /// Format a block and return the rendered result.
    fn render_block(
        &self,
        block: &Block,
        context: FormatContext,
    ) -> Result<BlockRenderResult, AppError>;

    /// Format children blocks sequentially, threading context through siblings.
    fn render_children(
        &self,
        blocks: &[Block],
        context: FormatContext,
    ) -> Result<Vec<BlockRenderResult>, AppError> {
        let mut results = Vec::with_capacity(blocks.len());
        let mut current_context = context;

        for block in blocks {
            let result = self.render_block(block, current_context)?;
            current_context = result.context.clone();
            results.push(result);
        }

        Ok(results)
    }
}

/// Formats Notion blocks as markdown.
pub struct MarkdownBlockRenderer<'a> {
    config: &'a RenderContext<'a>,
    /// All blocks in the document (for TOC generation)
    document_blocks: Option<&'a [Block]>,
}

impl<'a> MarkdownBlockRenderer<'a> {
    #[allow(dead_code)]
    pub fn new(config: &'a RenderContext<'a>) -> Self {
        Self {
            config,
            document_blocks: None,
        }
    }

    pub fn with_document_blocks(
        config: &'a RenderContext<'a>,
        document_blocks: &'a [Block],
    ) -> Self {
        Self {
            config,
            document_blocks: Some(document_blocks),
        }
    }

    /// Format a block and its children, returning the complete formatted content
    #[allow(dead_code)]
    pub fn format(&self, block: &Block, context: FormatContext) -> Result<String, AppError> {
        let result = self.render_block(block, context)?;
        Ok(result.content)
    }

    /// Format a block and return both content and updated context
    pub fn format_with_context(
        &self,
        block: &Block,
        context: FormatContext,
    ) -> Result<BlockRenderResult, AppError> {
        self.render_block(block, context)
    }

    /// Format text content with prefix - pure function
    fn format_text_content(
        &self,
        content: &TextBlockContent,
        prefix: &str,
    ) -> Result<String, AppError> {
        let text = rich_text_to_markdown(&content.rich_text)?;
        Ok(if text.trim().is_empty() {
            format!("{}\n", prefix)
        } else {
            format!("{}{}\n", prefix, text)
        })
    }

    /// Format heading with appropriate markdown level
    fn format_heading(&self, level: u8, content: &TextBlockContent) -> Result<String, AppError> {
        let prefix = "#".repeat(level as usize);
        self.format_text_content(content, &format!("{} ", prefix))
    }

    /// Format code block with language
    fn format_code_block(&self, code: &CodeBlock) -> Result<String, AppError> {
        let lang = &code.language;
        let caption = if !code.caption.is_empty() {
            rich_text_to_markdown(&code.caption)?
        } else {
            String::new()
        };

        let mut result = format!("```{}\n", lang);
        for item in &code.content.rich_text {
            result.push_str(&item.plain_text);
        }
        result.push_str("\n```\n");

        if !caption.is_empty() {
            result.push_str(&format!("*{}*\n", caption));
        }

        Ok(result)
    }

    /// Generate table of contents from document headings
    fn generate_table_of_contents(&self) -> Result<String, AppError> {
        let Some(blocks) = self.document_blocks else {
            return Ok("[Table of Contents]\n".to_string());
        };

        let mut toc_entries = Vec::new();
        self.collect_headings_recursive(blocks, &mut toc_entries)?;

        if toc_entries.is_empty() {
            return Ok("[Table of Contents - No headings found]\n".to_string());
        }

        let mut result = String::new();
        result.push_str("## Table of Contents\n\n");

        for entry in toc_entries {
            let indent = "  ".repeat((entry.level as usize).saturating_sub(1));
            let anchor = self.create_anchor_link(&entry.text);
            result.push_str(&format!("{}* [{}](#{})\n", indent, entry.text, anchor));
        }
        result.push('\n');

        Ok(result)
    }

    /// Recursively collect headings from blocks
    #[allow(clippy::only_used_in_recursion)]
    fn collect_headings_recursive(
        &self,
        blocks: &[Block],
        toc_entries: &mut Vec<TocEntry>,
    ) -> Result<(), AppError> {
        for block in blocks {
            match block {
                Block::Heading1(h) => {
                    let text = rich_text_to_markdown(&h.content.rich_text)?;
                    if !text.trim().is_empty() {
                        toc_entries.push(TocEntry {
                            level: 1,
                            text: text.trim().to_string(),
                        });
                    }
                }
                Block::Heading2(h) => {
                    let text = rich_text_to_markdown(&h.content.rich_text)?;
                    if !text.trim().is_empty() {
                        toc_entries.push(TocEntry {
                            level: 2,
                            text: text.trim().to_string(),
                        });
                    }
                }
                Block::Heading3(h) => {
                    let text = rich_text_to_markdown(&h.content.rich_text)?;
                    if !text.trim().is_empty() {
                        toc_entries.push(TocEntry {
                            level: 3,
                            text: text.trim().to_string(),
                        });
                    }
                }
                _ => {}
            }

            if block.has_children() {
                self.collect_headings_recursive(block.children(), toc_entries)?;
            }
        }
        Ok(())
    }

    /// Create an anchor link from heading text
    fn create_anchor_link(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else {
                    // Skip special characters
                    '\0'
                }
            })
            .filter(|&c| c != '\0')
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
}

/// Indents each non-empty line of `text` by `indent`, preserving blank lines.
fn indent_block_content(text: &str, indent: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Extracts the URL from a `FileObject` (external or uploaded).
fn extract_file_url(file: &FileObject) -> &str {
    match file {
        FileObject::External { external } => &external.url,
        FileObject::File { file } => &file.url,
    }
}

impl BlockRenderer for MarkdownBlockRenderer<'_> {
    fn render_block(
        &self,
        block: &Block,
        context: FormatContext,
    ) -> Result<BlockRenderResult, AppError> {
        let content = match block {
            Block::Paragraph(b) => {
                self.format_text_with_children(&b.content, "", &b.common.children, &context)?
            }
            Block::Heading1(b) => {
                self.format_heading_block(1, &b.content, &b.common.children, &context)?
            }
            Block::Heading2(b) => {
                self.format_heading_block(2, &b.content, &b.common.children, &context)?
            }
            Block::Heading3(b) => {
                self.format_heading_block(3, &b.content, &b.common.children, &context)?
            }
            Block::BulletedListItem(b) => self.format_bulleted_list_item(b, &context)?,
            Block::NumberedListItem(b) => self.format_numbered_list_item(b, &context)?,
            Block::ToDo(b) => self.format_todo(b, &context)?,
            Block::Toggle(b) => self.format_toggle(b, &context)?,
            Block::Quote(b) => {
                self.format_text_with_children(&b.content, "> ", &b.common.children, &context)?
            }
            Block::Callout(b) => self.format_callout(b, &context)?,
            Block::Code(b) => self.format_code_block(b)?,
            Block::Divider(_) => "---\n".to_string(),
            Block::Equation(b) => format!("$$\n{}\n$$\n", b.expression),
            Block::Image(b) => self.format_image(b)?,
            Block::Video(b) => format!("[Video: {}]\n", extract_file_url(&b.video)),
            Block::File(b) => self.format_file(b)?,
            Block::Pdf(b) => format!("[PDF: {}]\n", extract_file_url(&b.pdf)),
            Block::Bookmark(b) => self.format_bookmark(b)?,
            Block::Embed(b) => format!("[Embed: {}]\n", b.url),
            Block::ChildPage(b) => format!("📄 [[{}]]\n", b.title),
            Block::ChildDatabase(b) => self.format_child_database(b)?,
            Block::LinkToPage(b) => format!("[[{}]]\n", b.page_id.as_str()),
            Block::Table(b) => {
                self.format_children(&b.common.children, context.enter_table(b.table_width))?
            }
            Block::TableRow(b) => self.format_table_row(b, &context)?,
            Block::ColumnList(_) => {
                self.format_children(block.children(), context.enter_columns())?
            }
            Block::Column(_) => self.format_children(block.children(), context.clone())?,
            Block::Synced(b) => self.format_synced(b, &context)?,
            Block::Template(b) => self.format_text_with_children(
                &b.content,
                "[Template] ",
                &b.common.children,
                &context,
            )?,
            Block::LinkPreview(b) => format!("[Link Preview: {}]\n", b.url),
            Block::Breadcrumb(_) => "[Breadcrumb]\n".to_string(),
            Block::TableOfContents(_) => self.generate_table_of_contents()?,
            Block::Unsupported(b) => format!("[Unsupported block type: {}]\n", b.block_type),
        };

        // Determine final context based on block type
        let final_context = match block {
            Block::NumberedListItem(_) => context.increment_list_number(),
            Block::TableRow(_) => context.process_table_row(),
            Block::BulletedListItem(_) | Block::ToDo(_) => context,
            _ => context,
        };

        Ok(BlockRenderResult {
            content,
            context: final_context,
        })
    }
}

impl MarkdownBlockRenderer<'_> {
    // --- Block-type formatters ---

    /// Format text content followed by its children (used by Paragraph, Quote, Template).
    fn format_text_with_children(
        &self,
        content: &TextBlockContent,
        prefix: &str,
        children: &[Block],
        context: &FormatContext,
    ) -> Result<String, AppError> {
        let text = self.format_text_content(content, prefix)?;
        let child_md = self.format_children(children, context.clone())?;
        Ok(format!("{}{}", text, child_md))
    }

    /// Format a heading block (h1–h3) with its children.
    fn format_heading_block(
        &self,
        level: u8,
        content: &TextBlockContent,
        children: &[Block],
        context: &FormatContext,
    ) -> Result<String, AppError> {
        let text = self.format_heading(level, content)?;
        let child_md = self.format_children(children, context.clone())?;
        Ok(format!("{}{}", text, child_md))
    }

    /// Format a bulleted list item with indented children.
    fn format_bulleted_list_item(
        &self,
        b: &BulletedListItemBlock,
        context: &FormatContext,
    ) -> Result<String, AppError> {
        let text = self.format_text_content(&b.content, "- ")?;
        let children = self.format_indented_children(
            &b.common.children,
            context.enter_bulleted_list().enter_children(),
            "   ",
        )?;
        Ok(format!("{}{}", text, children))
    }

    /// Format a numbered list item with counter and indented children.
    fn format_numbered_list_item(
        &self,
        b: &NumberedListItemBlock,
        context: &FormatContext,
    ) -> Result<String, AppError> {
        let number = format!("{}. ", context.current_list_number());
        let text = self.format_text_content(&b.content, &number)?;
        let child_context = context.enter_children().enter_numbered_list();
        let children = self.format_indented_children(&b.common.children, child_context, "   ")?;
        Ok(format!("{}{}", text, children))
    }

    /// Format a to-do item with checkbox and indented children.
    fn format_todo(&self, b: &ToDoBlock, context: &FormatContext) -> Result<String, AppError> {
        let checkbox = if b.checked { "[x]" } else { "[ ]" };
        let text = self.format_text_content(&b.content, &format!("- {} ", checkbox))?;
        let children = self.format_indented_children(&b.common.children, context.clone(), "  ")?;
        Ok(format!("{}{}", text, children))
    }

    /// Format a toggle block with indented children.
    fn format_toggle(&self, b: &ToggleBlock, context: &FormatContext) -> Result<String, AppError> {
        let text = self.format_text_content(&b.content, "▸ ")?;
        let children =
            self.format_indented_children(&b.common.children, context.enter_toggle(), "  ")?;
        Ok(format!("{}{}", text, children))
    }

    /// Format a callout block with optional icon.
    fn format_callout(
        &self,
        b: &CalloutBlock,
        context: &FormatContext,
    ) -> Result<String, AppError> {
        let emoji = match &b.icon {
            Some(Icon::Emoji { emoji }) => format!("{} ", emoji),
            _ => String::new(),
        };
        let text = self.format_text_content(&b.content, &format!("> {} ", emoji))?;
        let child_md = self.format_children(&b.common.children, context.enter_callout())?;
        Ok(format!("{}{}", text, child_md))
    }

    /// Format an image block with caption.
    fn format_image(&self, b: &ImageBlock) -> Result<String, AppError> {
        let url = extract_file_url(&b.image);
        let caption = if b.caption.is_empty() {
            String::from("Image")
        } else {
            rich_text_to_markdown(&b.caption)?
        };
        Ok(format!("![{}]({})\n", caption, url))
    }

    /// Format a file block with caption.
    fn format_file(&self, b: &FileBlock) -> Result<String, AppError> {
        let url = extract_file_url(&b.file);
        let caption = if b.caption.is_empty() {
            String::from("File")
        } else {
            rich_text_to_markdown(&b.caption)?
        };
        Ok(format!("[{}: {}]\n", caption, url))
    }

    /// Format a bookmark block with optional caption.
    fn format_bookmark(&self, b: &BookmarkBlock) -> Result<String, AppError> {
        let caption_text = if b.caption.is_empty() {
            String::new()
        } else {
            format!(" - {}", rich_text_to_markdown(&b.caption)?)
        };
        Ok(format!("[🔖 {}{}]\n", b.url, caption_text))
    }

    /// Format a table row, adding a header separator after the first row.
    fn format_table_row(
        &self,
        b: &TableRowBlock,
        context: &FormatContext,
    ) -> Result<String, AppError> {
        let mut row = String::from("|");
        for cell in &b.cells {
            let content = rich_text_to_markdown(cell)?;
            row.push_str(&format!(" {} |", content));
        }
        row.push('\n');

        if context.is_first_table_row() {
            let separator = "|".to_string() + &vec![" --- |"; b.cells.len()].join("");
            row.push_str(&separator);
            row.push('\n');
        }
        Ok(row)
    }

    /// Format a synced block, showing the source reference and children.
    fn format_synced(&self, b: &SyncedBlock, context: &FormatContext) -> Result<String, AppError> {
        let mut result = String::new();
        if let Some(ref synced_from) = b.synced_from {
            result.push_str(&format!(
                "[Synced from: {}]\n",
                synced_from.block_id.as_str()
            ));
        }
        let child_md = self.format_children(&b.common.children, context.clone())?;
        result.push_str(&child_md);
        Ok(result)
    }

    // --- Helpers ---

    /// Format children blocks and indent the result.
    fn format_indented_children(
        &self,
        children: &[Block],
        context: FormatContext,
        indent: &str,
    ) -> Result<String, AppError> {
        if children.is_empty() {
            return Ok(String::new());
        }
        let formatted = self.format_children(children, context)?;
        Ok(indent_block_content(&formatted, indent))
    }

    /// Format a child database block based on its resolution state.
    fn format_child_database(&self, b: &ChildDatabaseBlock) -> Result<String, AppError> {
        use crate::model::blocks::ChildDatabaseContent;

        match &b.content {
            ChildDatabaseContent::Fetched(ref db) => {
                log::debug!(
                    "Formatting embedded child database '{}' ({} pages)",
                    b.title,
                    db.pages.len()
                );
                self.format_database_or_fallback(db, &b.title)
            }
            ChildDatabaseContent::LinkedDatabase => {
                log::debug!(
                    "Child database '{}' is a linked database (not retrievable via API)",
                    b.title
                );
                Ok(format!(
                    "🗄️ **{}** _(linked database — not retrievable via API)_\n",
                    b.title
                ))
            }
            ChildDatabaseContent::Inaccessible { reason } => {
                log::debug!("Database '{}' inaccessible: {}", b.title, reason);
                Ok(format!("🗄️ [[{}]]\n", b.title))
            }
            ChildDatabaseContent::NotFetched => {
                // Try external database lookup as fallback
                if let Some(lookup) = self.config.databases {
                    let db_id: crate::types::NotionId = b.common.id.clone().into();
                    if let Some(db) = lookup.find_database(&db_id) {
                        log::debug!(
                            "Formatting external child database '{}' ({} pages)",
                            b.title,
                            db.pages.len()
                        );
                        return self.format_database_or_fallback(db, &b.title);
                    }
                }
                log::debug!("No database data found for '{}'", b.title);
                Ok(format!("🗄️ [[{}]]\n", b.title))
            }
        }
    }

    /// Format a database inline, falling back to a placeholder on error.
    fn format_database_or_fallback(
        &self,
        db: &crate::model::Database,
        title: &str,
    ) -> Result<String, AppError> {
        match crate::formatting::databases::format_database_inline(db, &db.pages, "") {
            Ok(formatted) => Ok(formatted),
            Err(e) => {
                log::warn!("Failed to format child database '{}': {}", title, e);
                Ok(format!("🗄️ [[{}]]\n", title))
            }
        }
    }

    /// Format children blocks with proper context
    fn format_children(
        &self,
        children: &[Block],
        context: FormatContext,
    ) -> Result<String, AppError> {
        if children.is_empty() {
            return Ok(String::new());
        }

        let results = self.render_children(children, context)?;
        Ok(results
            .into_iter()
            .map(|r| r.content)
            .collect::<Vec<_>>()
            .join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::blocks::*;
    use crate::types::{BlockId, Color, RichTextItem};

    fn create_test_rich_text(text: &str) -> Vec<RichTextItem> {
        vec![RichTextItem::plain_text(text)]
    }

    fn create_numbered_list_item(id: &str, text: &str, children: Vec<Block>) -> Block {
        Block::NumberedListItem(NumberedListItemBlock {
            common: crate::model::BlockCommon {
                id: BlockId::parse(id).unwrap_or_else(|_| BlockId::new_v4()),
                has_children: !children.is_empty(),
                children,
                archived: false,
            },
            content: TextBlockContent {
                rich_text: create_test_rich_text(text),
                color: Color::Default,
            },
        })
    }

    fn create_bulleted_list_item(id: &str, text: &str, children: Vec<Block>) -> Block {
        Block::BulletedListItem(BulletedListItemBlock {
            common: crate::model::BlockCommon {
                id: BlockId::parse(id).unwrap_or_else(|_| BlockId::new_v4()),
                has_children: !children.is_empty(),
                children,
                archived: false,
            },
            content: TextBlockContent {
                rich_text: create_test_rich_text(text),
                color: Color::Default,
            },
        })
    }

    #[test]
    fn test_nested_list_formatting() {
        let config = RenderContext::default();
        let formatter = MarkdownBlockRenderer::new(&config);
        let context = FormatContext::new();

        // Create a numbered list item with bulleted children
        let block = create_numbered_list_item(
            "12345678-1234-1234-1234-123456789abc",
            "Managing complex use cases",
            vec![
                create_bulleted_list_item(
                    "12345678-1234-1234-1234-123456789a1a",
                    "Document analysis workflows",
                    vec![],
                ),
                create_bulleted_list_item(
                    "12345678-1234-1234-1234-123456789a1b",
                    "Legal/compliance requirements",
                    vec![],
                ),
            ],
        );

        let result = formatter.format(&block, context).unwrap();

        // Check that the output contains proper indentation
        assert!(result.contains("1. Managing complex use cases\n"));
        assert!(
            result.contains("   - Document analysis workflows\n"),
            "Expected child list items to be indented with 3 spaces. Got:\n{}",
            result
        );
        assert!(
            result.contains("   - Legal/compliance requirements\n"),
            "Expected child list items to be indented with 3 spaces. Got:\n{}",
            result
        );
    }

    #[test]
    fn test_deeply_nested_lists() {
        let config = RenderContext::default();
        let formatter = MarkdownBlockRenderer::new(&config);
        let context = FormatContext::new();

        // Create a more deeply nested structure
        let block = create_numbered_list_item(
            "12345678-1234-1234-1234-123456789abc",
            "Level 1",
            vec![create_bulleted_list_item(
                "12345678-1234-1234-1234-123456789a1a",
                "Level 2",
                vec![create_bulleted_list_item(
                    "12345678-1234-1234-1234-123456789a11",
                    "Level 3",
                    vec![],
                )],
            )],
        );

        let result = formatter.format(&block, context).unwrap();

        assert!(result.contains("1. Level 1\n"));
        assert!(result.contains("   - Level 2\n"));
        assert!(
            result.contains("      - Level 3\n"),
            "Expected deeply nested items to have additional indentation. Got:\n{}",
            result
        );
    }

    #[test]
    fn test_numbered_list_counter_increments() {
        let config = RenderContext::default();
        let _formatter = MarkdownBlockRenderer::new(&config);

        // Create multiple numbered list items
        let blocks = vec![
            create_numbered_list_item("12345678-1234-1234-1234-123456789001", "First item", vec![]),
            create_numbered_list_item(
                "12345678-1234-1234-1234-123456789002",
                "Second item",
                vec![],
            ),
            create_numbered_list_item("12345678-1234-1234-1234-123456789003", "Third item", vec![]),
        ];

        // Test using render_blocks_with which should handle list context
        let output = crate::formatting::block_renderer::render_blocks(&blocks, &config).unwrap();

        assert_eq!(
            output, "1. First item\n2. Second item\n3. Third item\n",
            "Expected numbered list counter to increment. Got:\n{}",
            output
        );
    }

    fn create_heading1(id: &str, text: &str) -> Block {
        Block::Heading1(Heading1Block {
            common: crate::model::BlockCommon {
                id: BlockId::parse(id).unwrap_or_else(|_| BlockId::new_v4()),
                has_children: false,
                children: vec![],
                archived: false,
            },
            content: TextBlockContent {
                rich_text: create_test_rich_text(text),
                color: Color::Default,
            },
        })
    }

    fn create_heading2(id: &str, text: &str) -> Block {
        Block::Heading2(Heading2Block {
            common: crate::model::BlockCommon {
                id: BlockId::parse(id).unwrap_or_else(|_| BlockId::new_v4()),
                has_children: false,
                children: vec![],
                archived: false,
            },
            content: TextBlockContent {
                rich_text: create_test_rich_text(text),
                color: Color::Default,
            },
        })
    }

    fn create_table_of_contents(id: &str) -> Block {
        Block::TableOfContents(TableOfContentsBlock {
            common: crate::model::BlockCommon {
                id: BlockId::parse(id).unwrap_or_else(|_| BlockId::new_v4()),
                has_children: false,
                children: vec![],
                archived: false,
            },
        })
    }

    #[test]
    fn test_table_of_contents_generation() {
        let config = RenderContext::default();

        // Create a document with headings and a TOC block
        let blocks = vec![
            create_table_of_contents("12345678-1234-1234-1234-123456789toc"),
            create_heading1("12345678-1234-1234-1234-123456789h01", "Introduction"),
            create_heading2("12345678-1234-1234-1234-123456789h02", "Overview"),
            create_heading1("12345678-1234-1234-1234-123456789h03", "Main Content"),
            create_heading2("12345678-1234-1234-1234-123456789h04", "Section A"),
        ];

        let output = crate::formatting::block_renderer::render_blocks(&blocks, &config).unwrap();

        // Check that TOC was generated instead of placeholder
        assert!(output.contains("## Table of Contents"));
        assert!(output.contains("* [Introduction](#introduction)"));
        assert!(output.contains("  * [Overview](#overview)"));
        assert!(output.contains("* [Main Content](#main-content)"));
        assert!(output.contains("  * [Section A](#section-a)"));

        // Make sure it's not the old placeholder
        assert!(!output.contains("[Table of Contents]"));

        println!("Generated TOC output:\n{}", output);
    }
}
