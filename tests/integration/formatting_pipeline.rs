// tests/integration/formatting_pipeline.rs
//! Integration tests for the complete formatting pipeline

use notion2prompt::model::*;
use notion2prompt::formatting::core::format_notion_object;
use notion2prompt::formatting::state::FormatContext;
use notion2prompt::config::FormatConfig;
use notion2prompt::types::{BlockId, NotionId, RichTextItem, Color};

fn create_test_page() -> NotionObject {
    let page_id = NotionId::parse("12345678-1234-1234-1234-123456789abc").unwrap();
    
    let blocks = vec![
        Box::new(Block::Heading1(HeadingBlock {
            common: BlockCommon::new(BlockId::parse("heading1").unwrap()),
            content: HeadingContent {
                rich_text: vec![RichTextItem {
                    plain_text: "Test Document".to_string(),
                    ..Default::default()
                }],
            },
            color: Color::Default,
            is_toggleable: false,
        })),
        Box::new(Block::Paragraph(ParagraphBlock {
            common: BlockCommon::new(BlockId::parse("para1").unwrap()),
            content: ParagraphContent {
                rich_text: vec![RichTextItem {
                    plain_text: "This is a test paragraph.".to_string(),
                    ..Default::default()
                }],
                color: Color::Default,
            },
        })),
        Box::new(Block::BulletedListItem(ListItemBlock {
            common: BlockCommon::new(BlockId::parse("bullet1").unwrap()),
            content: TextBlockContent {
                rich_text: vec![RichTextItem {
                    plain_text: "First bullet point".to_string(),
                    ..Default::default()
                }],
                color: Color::Default,
            },
        })),
        Box::new(Block::BulletedListItem(ListItemBlock {
            common: BlockCommon::new(BlockId::parse("bullet2").unwrap()),
            content: TextBlockContent {
                rich_text: vec![RichTextItem {
                    plain_text: "Second bullet point".to_string(),
                    ..Default::default()
                }],
                color: Color::Default,
            },
        })),
        Box::new(Block::Code(CodeBlock {
            common: BlockCommon::new(BlockId::parse("code1").unwrap()),
            content: CodeContent {
                rich_text: vec![RichTextItem {
                    plain_text: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
                    ..Default::default()
                }],
                caption: vec![],
            },
            language: "rust".to_string(),
        })),
    ];
    
    NotionObject::Page(Page {
        id: page_id.into(),
        title: PageTitle::from_plain_text("Test Page"),
        children: blocks,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn format_complete_page() {
        let page = create_test_page();
        let config = FormatConfig::default();
        let mut output = String::new();
        let context = FormatContext::new();
        
        let result = format_notion_object(
            &page,
            &mut output,
            &context,
            Some(&config),
        );
        
        assert!(result.is_ok());
        
        // Verify output contains expected elements
        assert!(output.contains("# Test Document"));
        assert!(output.contains("This is a test paragraph."));
        assert!(output.contains("- First bullet point"));
        assert!(output.contains("- Second bullet point"));
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main()"));
    }
    
    #[test]
    fn format_nested_blocks() {
        let nested_bullet = Box::new(Block::BulletedListItem(ListItemBlock {
            common: BlockCommon {
                id: BlockId::parse("nested").unwrap(),
                children: vec![
                    Box::new(Block::Paragraph(ParagraphBlock {
                        common: BlockCommon::new(BlockId::parse("nested-para").unwrap()),
                        content: ParagraphContent {
                            rich_text: vec![RichTextItem {
                                plain_text: "Nested paragraph".to_string(),
                                ..Default::default()
                            }],
                            color: Color::Default,
                        },
                    }))
                ],
                has_children: true,
                archived: false,
            },
            content: TextBlockContent {
                rich_text: vec![RichTextItem {
                    plain_text: "Parent bullet".to_string(),
                    ..Default::default()
                }],
                color: Color::Default,
            },
        }));
        
        let page = NotionObject::Page(Page {
            id: NotionId::parse("page123").unwrap().into(),
            title: PageTitle::from_plain_text("Nested Test"),
            children: vec![nested_bullet],
            ..Default::default()
        });
        
        let config = FormatConfig::default();
        let mut output = String::new();
        let context = FormatContext::new();
        
        let result = format_notion_object(&page, &mut output, &context, Some(&config));
        assert!(result.is_ok());
        
        assert!(output.contains("- Parent bullet"));
        assert!(output.contains("Nested paragraph"));
    }
    
    #[test]
    fn format_with_sanitization() {
        let page = NotionObject::Page(Page {
            id: NotionId::parse("page123").unwrap().into(),
            title: PageTitle::from_plain_text("HTML Test"),
            children: vec![
                Box::new(Block::Paragraph(ParagraphBlock {
                    common: BlockCommon::new(BlockId::parse("para").unwrap()),
                    content: ParagraphContent {
                        rich_text: vec![RichTextItem {
                            plain_text: "Text with <script>alert('xss')</script> tags".to_string(),
                            ..Default::default()
                        }],
                        color: Color::Default,
                    },
                }))
            ],
            ..Default::default()
        });
        
        let config = FormatConfig {
            enable_sanitization: true,
            ..Default::default()
        };
        
        let mut output = String::new();
        let context = FormatContext::new();
        
        let result = format_notion_object(&page, &mut output, &context, Some(&config));
        assert!(result.is_ok());
        
        // Script tags should be sanitized
        assert!(!output.contains("<script>"));
        assert!(output.contains("Text with"));
    }
}