// tests/unit/formatting.rs
//! Unit tests for formatting improvements

use notion2prompt::formatting::state::FormatContext;
use notion2prompt::formatting::pure_visitor::{PureBlockVisitor, PureMarkdownFormatter, FormattedBlock};
use notion2prompt::formatting::registry::{FormatterRegistry, FormatterRegistryBuilder};
use notion2prompt::formatting::effects::{IOEffect, Pure};
use notion2prompt::model::*;
use notion2prompt::types::{BlockId, RichTextItem};
use notion2prompt::config::FormatConfig;

fn test_block_id() -> BlockId {
    BlockId::parse("12345678-1234-1234-1234-123456789abc").unwrap()
}

fn test_paragraph_block() -> Block {
    Block::Paragraph(ParagraphBlock {
        common: BlockCommon::new(test_block_id()),
        content: ParagraphContent {
            rich_text: vec![RichTextItem {
                plain_text: "Test paragraph".to_string(),
                ..Default::default()
            }],
            color: notion2prompt::types::Color::Default,
        },
    })
}

#[cfg(test)]
mod format_context_tests {
    use super::*;
    
    #[test]
    fn format_context_immutable_transitions() {
        let context = FormatContext::new();
        assert_eq!(context.indent_level(), 0);
        assert!(!context.is_in_list());
        
        // Test immutable state transitions
        let list_context = context.enter_list();
        assert!(list_context.is_in_list());
        assert_eq!(context.indent_level(), 0); // Original unchanged
        assert_eq!(list_context.indent_level(), 0);
        
        let nested_context = list_context.enter_children();
        assert_eq!(nested_context.indent_level(), 1);
        assert!(nested_context.is_in_list());
        
        let deeper_context = nested_context.enter_children();
        assert_eq!(deeper_context.indent_level(), 2);
        
        let exit_context = deeper_context.exit_children();
        assert_eq!(exit_context.indent_level(), 1);
    }
    
    #[test]
    fn format_context_list_tracking() {
        let context = FormatContext::new();
        
        let bullet_context = context.enter_bulleted_list();
        assert!(bullet_context.is_in_list());
        assert!(bullet_context.is_in_bulleted_list());
        assert!(!bullet_context.is_in_numbered_list());
        
        let numbered_context = context.enter_numbered_list();
        assert!(numbered_context.is_in_list());
        assert!(!numbered_context.is_in_bulleted_list());
        assert!(numbered_context.is_in_numbered_list());
        
        let exit_context = bullet_context.exit_list();
        assert!(!exit_context.is_in_list());
    }
}

#[cfg(test)]
mod pure_visitor_tests {
    use super::*;
    
    #[test]
    fn pure_markdown_formatter_paragraph() {
        let config = FormatConfig::default();
        let formatter = PureMarkdownFormatter::new(&config);
        let block = test_paragraph_block();
        let context = FormatContext::new();
        
        let result = formatter.visit_block(&block, context.clone()).unwrap();
        assert_eq!(result.content, "Test paragraph\n\n");
        assert_eq!(result.context.indent_level(), context.indent_level());
    }
    
    #[test]
    fn pure_markdown_formatter_nested() {
        let config = FormatConfig::default();
        let formatter = PureMarkdownFormatter::new(&config);
        let block = test_paragraph_block();
        
        // Test with nested context
        let context = FormatContext::new()
            .enter_bulleted_list()
            .enter_children();
        
        let result = formatter.visit_block(&block, context.clone()).unwrap();
        assert!(result.content.contains("Test paragraph"));
        assert_eq!(result.context.indent_level(), context.indent_level());
    }
}

#[cfg(test)]
mod formatter_registry_tests {
    use super::*;
    
    #[test]
    fn formatter_registry_builder() {
        let registry = FormatterRegistryBuilder::new()
            .add_formatter("paragraph", |block, _ctx| {
                Ok(FormattedBlock {
                    content: "Custom paragraph".to_string(),
                    context: FormatContext::new(),
                })
            })
            .add_formatter("heading", |block, _ctx| {
                Ok(FormattedBlock {
                    content: "# Custom heading".to_string(),
                    context: FormatContext::new(),
                })
            })
            .set_default_formatter(|block, _ctx| {
                Ok(FormattedBlock {
                    content: "Unknown block".to_string(),
                    context: FormatContext::new(),
                })
            })
            .build();
        
        let block = test_paragraph_block();
        let result = registry.format(&block, FormatContext::new()).unwrap();
        assert_eq!(result.content, "Custom paragraph");
    }
    
    #[test]
    fn formatter_registry_default() {
        let registry = FormatterRegistryBuilder::new()
            .set_default_formatter(|block, _ctx| {
                Ok(FormattedBlock {
                    content: format!("Default: {}", block.block_type()),
                    context: FormatContext::new(),
                })
            })
            .build();
        
        let block = test_paragraph_block();
        let result = registry.format(&block, FormatContext::new()).unwrap();
        assert_eq!(result.content, "Default: paragraph");
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn io_effect_variants() {
        let effects = vec![
            IOEffect::WriteFile {
                path: PathBuf::from("/tmp/test.txt"),
                content: "test content".to_string(),
            },
            IOEffect::ReadFile {
                path: PathBuf::from("/tmp/test.txt"),
            },
            IOEffect::CreateDirectory {
                path: PathBuf::from("/tmp/test_dir"),
            },
            IOEffect::CopyToClipboard {
                content: "clipboard content".to_string(),
            },
        ];
        
        // Just verify they can be created
        assert_eq!(effects.len(), 4);
    }
    
    #[test]
    fn pure_monad() {
        // Test Pure monad operations
        let pure_value = Pure::of(42);
        
        let mapped = pure_value.map(|x| x * 2);
        assert_eq!(mapped.value(), &84);
        
        let flat_mapped = Pure::of(10).flat_map(|x| Pure::of(x + 5));
        assert_eq!(flat_mapped.value(), &15);
        
        // Test Apply
        let pure_fn = Pure::of(|x: i32| x * 3);
        let result = Pure::apply(pure_fn, Pure::of(7));
        assert_eq!(result.value(), &21);
    }
}