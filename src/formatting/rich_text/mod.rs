// src/formatting/rich_text/mod.rs
//! Handles formatting of Notion RichTextItem arrays into various formats.
//!
//! This module provides a structured approach to processing rich text,
//! with separate handling for different text types and mention types.

mod annotations;
mod handlers;
mod types;

// Re-export the public interface
pub use types::{
    EquationContent, FormattedText, MentionContent, TextContent, TextSegment, TextStyle,
    ValidatedUrl,
};

use crate::error::AppError;
use crate::types::RichTextItem;
use crate::types::{EquationData, Link};
use annotations::{annotations_to_style, MarkdownStyleRenderer};
use handlers::{extract_notion_id, is_database_reference, MentionHandlerRegistry};

// --- Main Formatting Function ---

/// Formats an array of RichTextItems into Markdown.
/// This is the main entry point maintaining backward compatibility.
pub fn rich_text_to_markdown(items: &[RichTextItem]) -> Result<String, AppError> {
    let formatted = format_rich_text_items(items)?;
    Ok(render_to_markdown(&formatted))
}

/// Formats rich text items into a structured representation.
pub fn format_rich_text_items(items: &[RichTextItem]) -> Result<FormattedText, AppError> {
    let mut formatted = FormattedText::new();
    let mention_registry = MentionHandlerRegistry::new();

    for item in items {
        let segment = format_single_item(item, &mention_registry)?;
        if !segment.is_empty() {
            formatted = formatted.with_segment(segment);
        }
    }

    Ok(formatted)
}

/// Formats a single rich text item.
fn format_single_item(
    item: &RichTextItem,
    mention_registry: &MentionHandlerRegistry,
) -> Result<TextSegment, AppError> {
    // Extract base style from annotations
    let mut style = annotations_to_style(&item.annotations);

    // Add href as link if present
    if let Some(href) = &item.href {
        if let Ok(url) = ValidatedUrl::parse(href) {
            style.link = Some(url);
        }
    }

    // Process content based on the rich text type — no string matching needed,
    // the enum variant carries its data and the match is exhaustive.
    use crate::types::RichTextType;
    let content = match &item.text_type {
        RichTextType::Equation(EquationData { expression }) => {
            TextContent::Equation(EquationContent {
                expression: expression.clone(),
                inline: true,
            })
        }

        RichTextType::Mention(mention_data) => {
            let mention_content = mention_registry.handle(mention_data, &item.plain_text);

            // Check for database references in mentions
            if let MentionContent::Link { ref url, ref text } = mention_content {
                if is_database_reference(text, url) {
                    if let Some(notion_id) = extract_notion_id(url) {
                        return Ok(TextSegment {
                            content: TextContent::Mention(MentionContent::Database {
                                id: notion_id,
                                title: text.clone(),
                            }),
                            style,
                        });
                    }
                }
            }

            TextContent::Mention(mention_content)
        }

        RichTextType::Text { content, link } => {
            let text = content.clone();

            // Apply link from text object if present
            if let Some(Link { url }) = link {
                if let Ok(validated_url) = ValidatedUrl::parse(url) {
                    // Check for database references
                    if is_database_reference(&text, &validated_url) {
                        if let Some(notion_id) = extract_notion_id(&validated_url) {
                            return Ok(TextSegment {
                                content: TextContent::Mention(MentionContent::Database {
                                    id: notion_id,
                                    title: text.clone(),
                                }),
                                style,
                            });
                        }
                    }
                    style.link = Some(validated_url);
                }
            }

            TextContent::Plain(text)
        }
    };

    Ok(TextSegment { content, style })
}

/// Renders formatted text to Markdown.
pub fn render_to_markdown(formatted: &FormattedText) -> String {
    let mut output = String::new();

    for segment in &formatted.segments {
        let rendered = render_segment_markdown(segment);
        output.push_str(&rendered);
    }

    output
}

/// Renders a single text segment to Markdown.
fn render_segment_markdown(segment: &TextSegment) -> String {
    match &segment.content {
        TextContent::Plain(text) => MarkdownStyleRenderer::apply_styles(text, &segment.style),
        TextContent::Equation(eq) => {
            // Equations typically ignore styling
            if eq.inline {
                format!("${}$", eq.expression)
            } else {
                format!("$$\n{}\n$$", eq.expression)
            }
        }
        TextContent::Mention(mention) => render_mention_markdown(mention, &segment.style),
    }
}

/// Renders a mention to Markdown.
fn render_mention_markdown(mention: &MentionContent, style: &TextStyle) -> String {
    let base = match mention {
        MentionContent::User { name, .. } => format!("@{}", name),
        MentionContent::Page { id, title } => {
            let url = format!("https://www.notion.so/{}", id.value_hyphenated());
            format!("[{}]({})", title, url)
        }
        MentionContent::Database { id, title } => {
            let url = format!("https://www.notion.so/{}", id.value_hyphenated());
            format!("📊 **Child Database:** [{}]({})", title, url)
        }
        MentionContent::Date { start, end } => {
            if let Some(end) = end {
                format!("**{} → {}**", start, end)
            } else {
                format!("**{}**", start)
            }
        }
        MentionContent::Link { url, text } => {
            if MarkdownStyleRenderer::is_markdown_link(text) {
                text.clone()
            } else {
                format!("[{}]({})", text, url.as_str())
            }
        }
    };

    // Apply additional styling if needed (mentions usually have limited styling)
    if style.has_styling() && !matches!(mention, MentionContent::Link { .. }) {
        // Only apply non-link styles to non-link mentions
        let mut modified_style = style.clone();
        modified_style.link = None;
        MarkdownStyleRenderer::apply_styles(&base, &modified_style)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Annotations;
    use crate::types::RichTextItem;

    #[test]
    fn test_plain_text_formatting() {
        let items = vec![RichTextItem::plain_text("Hello World")];

        let result = rich_text_to_markdown(&items).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_bold_italic_formatting() {
        let items = vec![RichTextItem {
            text_type: crate::types::RichTextType::Text {
                content: "Bold Italic".to_string(),
                link: None,
            },
            plain_text: "Bold Italic".to_string(),
            href: None,
            annotations: Annotations {
                bold: true,
                italic: true,
                ..Default::default()
            },
        }];

        let result = rich_text_to_markdown(&items).unwrap();
        assert_eq!(result, "***Bold Italic***");
    }

    #[test]
    fn test_equation_formatting() {
        let items = vec![RichTextItem {
            text_type: crate::types::RichTextType::Equation(EquationData {
                expression: "E = mc^2".to_string(),
            }),
            plain_text: "E = mc^2".to_string(),
            href: None,
            annotations: Annotations::default(),
        }];

        let result = rich_text_to_markdown(&items).unwrap();
        assert_eq!(result, "$E = mc^2$");
    }
}
