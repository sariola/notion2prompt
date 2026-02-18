// tests/unit/model.rs
//! Unit tests for model improvements

use notion2prompt::model::*;
use notion2prompt::types::{BlockId, Color, RichTextItem};

/// Helper function to create a test block ID
fn test_block_id(suffix: &str) -> BlockId {
    BlockId::parse(&format!("12345678-1234-1234-1234-{:0>12}", suffix))
        .expect("Test block ID should be valid")
}

/// Helper function to create test rich text
fn test_rich_text(text: &str) -> Vec<RichTextItem> {
    vec![RichTextItem {
        plain_text: text.to_string(),
        ..Default::default()
    }]
}

#[cfg(test)]
mod block_macro_tests {
    use super::*;
    
    #[test]
    fn block_id_accessor_works_for_all_variants() {
        let test_blocks = vec![
            Block::Paragraph(ParagraphBlock {
                common: BlockCommon {
                    id: test_block_id("paragraph"),
                    children: vec![],
                    has_children: false,
                    archived: false,
                },
                content: ParagraphContent {
                    rich_text: test_rich_text("test"),
                    color: Color::Default,
                },
            }),
            Block::Heading1(HeadingBlock {
                common: BlockCommon {
                    id: test_block_id("heading1"),
                    children: vec![],
                    has_children: false,
                    archived: false,
                },
                content: HeadingContent {
                    rich_text: test_rich_text("test"),
                },
                color: Color::Default,
                is_toggleable: false,
            }),
            Block::Divider(DividerBlock {
                common: BlockCommon {
                    id: test_block_id("divider"),
                    children: vec![],
                    has_children: false,
                    archived: false,
                },
            }),
        ];
        
        // Test that id() method works for all variants
        assert_eq!(test_blocks[0].id().as_str(), "12345678-1234-1234-1234-paragraph000");
        assert_eq!(test_blocks[1].id().as_str(), "12345678-1234-1234-1234-heading10000");
        assert_eq!(test_blocks[2].id().as_str(), "12345678-1234-1234-1234-divider00000");
    }
    
    #[test]
    fn block_children_accessors() {
        let child = Box::new(Block::Divider(DividerBlock {
            common: BlockCommon {
                id: test_block_id("child"),
                children: vec![],
                has_children: false,
                archived: false,
            },
        }));
        
        let mut parent = Block::Paragraph(ParagraphBlock {
            common: BlockCommon {
                id: test_block_id("parent"),
                children: vec![child.clone()],
                has_children: true,
                archived: false,
            },
            content: ParagraphContent {
                rich_text: test_rich_text("parent"),
                color: Color::Default,
            },
        });
        
        // Test immutable children access
        assert_eq!(parent.children().len(), 1);
        assert!(parent.has_children());
        
        // Test mutable children access
        parent.children_mut().push(child.clone());
        assert_eq!(parent.children().len(), 2);
        
        // Test set_children
        parent.set_children(vec![]);
        assert_eq!(parent.children().len(), 0);
    }
    
    #[test]
    fn block_common_accessors() {
        let block = Block::Quote(QuoteBlock {
            common: BlockCommon {
                id: test_block_id("quote"),
                children: vec![],
                has_children: false,
                archived: true,
            },
            content: TextBlockContent {
                rich_text: test_rich_text("quoted text"),
                color: Color::Blue,
            },
        });
        
        // Test common() accessor
        let common = block.common();
        assert_eq!(common.id.as_str(), "12345678-1234-1234-1234-quote0000000");
        assert!(common.archived);
        assert!(!common.has_children);
        
        // Test block_type()
        assert_eq!(block.block_type(), "quote");
    }
    
    #[test]
    fn block_type_names() {
        let test_cases = vec![
            (Block::Paragraph(Default::default()), "paragraph"),
            (Block::Heading1(Default::default()), "heading_1"),
            (Block::Heading2(Default::default()), "heading_2"),
            (Block::Heading3(Default::default()), "heading_3"),
            (Block::BulletedListItem(Default::default()), "bulleted_list_item"),
            (Block::NumberedListItem(Default::default()), "numbered_list_item"),
            (Block::Toggle(Default::default()), "toggle"),
            (Block::Quote(Default::default()), "quote"),
            (Block::Callout(Default::default()), "callout"),
            (Block::Code(Default::default()), "code"),
            (Block::Divider(Default::default()), "divider"),
            (Block::Table(Default::default()), "table"),
            (Block::Unsupported(Default::default()), "unsupported"),
        ];
        
        for (block, expected_type) in test_cases {
            assert_eq!(block.block_type(), expected_type);
        }
    }
}

#[cfg(test)]
mod block_common_tests {
    use super::*;
    
    #[test]
    fn block_common_default() {
        let common = BlockCommon::default();
        assert!(!common.id.as_str().is_empty());
        assert!(common.children.is_empty());
        assert!(!common.has_children);
        assert!(!common.archived);
    }
    
    #[test]
    fn block_common_new() {
        let id = test_block_id("test");
        let common = BlockCommon::new(id.clone());
        assert_eq!(common.id, id);
        assert!(common.children.is_empty());
        assert!(!common.has_children);
        assert!(!common.archived);
    }
}