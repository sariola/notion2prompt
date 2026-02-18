// tests/unit/child_database_embedding.rs
//! Unit tests for child database embedding functionality.

use notion2prompt::model::{
    Block, ChildDatabaseBlock, Database, DatabaseTitle, NotionObject, Page, PageTitle,
};
use notion2prompt::types::{BlockId, DatabaseId, NotionId, PageId, RichTextItem, Color};
use notion2prompt::model::blocks::{BlockCommon, TextBlockContent, ToggleBlock};
use std::collections::HashMap;

/// Creates a test page with ID
fn create_test_page(id: &str, title: &str) -> Page {
    Page {
        id: PageId::parse(id).unwrap(),
        title: PageTitle::new(title),
        url: format!("https://notion.so/{}", id),
        properties: HashMap::new(),
        blocks: vec![],
        parent: None,
        archived: false,
    }
}

/// Creates a test database with ID  
fn create_test_database(id: &str, title: &str) -> Database {
    use notion2prompt::types::RichTextItem;
    
    Database {
        id: DatabaseId::parse(id).unwrap(),
        title: DatabaseTitle::new(vec![RichTextItem {
            plain_text: title.to_string(),
            href: None,
            annotations: Default::default(),
            text_type: notion2prompt::types::RichTextType::Text {
                content: title.to_string(),
                link: None,
            },
        }]),
        url: format!("https://notion.so/{}", id),
        pages: vec![],
        properties: HashMap::new(),
        parent: None,
        archived: false,
    }
}

/// Creates a test ChildDatabaseBlock
fn create_child_database_block(id: &str, title: &str) -> Block {
    Block::ChildDatabase(ChildDatabaseBlock {
        common: BlockCommon {
            id: BlockId::parse(id).unwrap(),
            has_children: false,
            children: vec![],
            created_time: chrono::Utc::now(),
            last_edited_time: chrono::Utc::now(),
            archived: false,
            in_trash: false,
        },
        title: title.to_string(),
        content: notion2prompt::model::blocks::ChildDatabaseContent::NotFetched,
    })
}

#[test]
fn test_child_database_block_creation() {
    let block = create_child_database_block(
        "12345678123412341234123456789012",
        "Test Child Database"
    );
    
    match &block {
        Block::ChildDatabase(child_db) => {
            assert_eq!(child_db.title, "Test Child Database");
            assert!(matches!(child_db.content, notion2prompt::model::blocks::ChildDatabaseContent::NotFetched));
            assert!(!child_db.common.has_children);
        }
        _ => panic!("Expected ChildDatabase block"),
    }
}

#[test]
fn test_child_database_with_embedded_content() {
    let db_block_id = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let database = create_test_database(db_block_id, "Embedded Database");
    
    // Create a ChildDatabaseBlock with embedded database
    let block = Block::ChildDatabase(ChildDatabaseBlock {
        common: BlockCommon {
            id: BlockId::parse(db_block_id).unwrap(),
            has_children: false,
            children: vec![],
            created_time: chrono::Utc::now(),
            last_edited_time: chrono::Utc::now(),
            archived: false,
            in_trash: false,
        },
        title: "Child Database".to_string(),
        content: notion2prompt::model::blocks::ChildDatabaseContent::Fetched(Box::new(database)),
    });
    
    // Verify the embedded database
    match &block {
        Block::ChildDatabase(child_db) => {
            assert!(child_db.content.as_database().is_some());
            let embedded = child_db.content.as_database().unwrap();
            assert_eq!(embedded.title.as_plain_text(), "Embedded Database");
        }
        _ => panic!("Expected ChildDatabase block"),
    }
}

#[test]
fn test_page_with_child_database_block() {
    let page_id = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let db_block_id = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    
    // Create a child database block with embedded database
    let database = create_test_database(db_block_id, "Test Database Content");
    let child_db_block = Block::ChildDatabase(ChildDatabaseBlock {
        common: BlockCommon {
            id: BlockId::parse(db_block_id).unwrap(),
            has_children: false,
            children: vec![],
            created_time: chrono::Utc::now(),
            last_edited_time: chrono::Utc::now(),
            archived: false,
            in_trash: false,
        },
        title: "Child Database".to_string(),
        content: notion2prompt::model::blocks::ChildDatabaseContent::Fetched(Box::new(database)),
    });
    
    // Create a page with the child database block
    let mut page = create_test_page(page_id, "Parent Page");
    page.blocks.push(Box::new(child_db_block));
    
    // Verify the structure
    assert_eq!(page.blocks.len(), 1);
    match page.blocks[0].as_ref() {
        Block::ChildDatabase(child_db) => {
            assert!(child_db.content.as_database().is_some());
            assert_eq!(child_db.title, "Child Database");
        }
        _ => panic!("Expected ChildDatabase block"),
    }
}

#[test]
fn test_has_children_uses_api_flag() {
    let mut block = create_child_database_block(
        "12345678123412341234123456789012",
        "Test Block"
    );
    
    // Initially has_children is false and children vector is empty
    assert!(!block.has_children());
    assert!(block.children().is_empty());
    
    // Set has_children flag to true (simulating API response)
    if let Block::ChildDatabase(ref mut child_db) = block {
        child_db.common.has_children = true;
    }
    
    // Now has_children() should return true even though children vector is empty
    assert!(block.has_children());
    assert!(block.children().is_empty());
}

#[test]
fn test_multiple_child_databases_in_page() {
    let page_id = NotionId::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let db1_id = NotionId::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    let db2_id = NotionId::parse("cccccccccccccccccccccccccccccccc").unwrap();
    
    // Create blocks with two child databases
    let blocks = vec![
        Box::new(create_child_database_block(db1_id.as_str(), "Database 1")),
        Box::new(create_child_database_block(db2_id.as_str(), "Database 2")),
    ];
    
    // Build the graph
    let mut graph = ObjectGraph::new();
    
    // Add the page
    let page = create_test_page(page_id.as_str(), "Parent Page");
    graph = graph.with_object(NotionObject::Page(page));
    
    // Add the blocks
    graph = graph.with_blocks(page_id.clone(), blocks);
    
    // Add both databases
    let db1 = create_test_database(db1_id.as_str(), "Database 1 Content");
    let db2 = create_test_database(db2_id.as_str(), "Database 2 Content");
    graph = graph.with_object(NotionObject::Database(db1));
    graph = graph.with_object(NotionObject::Database(db2));
    
    // Assemble and verify
    let assembled = graph.assemble(page_id.clone()).unwrap();
    
    match assembled {
        NotionObject::Page(page) => {
            assert_eq!(page.blocks.len(), 2);
            
            // Both databases should be embedded
            for (i, block) in page.blocks.iter().enumerate() {
                match block.as_ref() {
                    Block::ChildDatabase(child_db) => {
                        assert!(child_db.content.as_database().is_some(), "Database {} should be embedded", i + 1);
                        let embedded_db = child_db.content.as_database().unwrap();
                        assert!(embedded_db.title.as_plain_text().starts_with("Database"));
                    }
                    _ => panic!("Expected ChildDatabase block"),
                }
            }
        }
        _ => panic!("Expected Page"),
    }
}

#[test]
fn test_nested_child_database_blocks() {
    let toggle_id = "dddddddddddddddddddddddddddddddd";
    let db_id = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    
    // Create a toggle block with a child database inside
    let toggle_block = Block::Toggle(ToggleBlock {
        common: BlockCommon {
            id: BlockId::parse(toggle_id).unwrap(),
            has_children: true,
            children: vec![
                Box::new(create_child_database_block(db_id, "Nested Database"))
            ],
            created_time: chrono::Utc::now(),
            last_edited_time: chrono::Utc::now(),
            archived: false,
            in_trash: false,
        },
        content: TextBlockContent {
            rich_text: vec![],
            color: Color::Default,
        },
    });
    
    // Verify the structure
    match &toggle_block {
        Block::Toggle(toggle) => {
            assert!(toggle.common.has_children);
            assert_eq!(toggle.common.children.len(), 1);
            
            // Check the nested child database
            match toggle.common.children[0].as_ref() {
                Block::ChildDatabase(child_db) => {
                    assert_eq!(child_db.title, "Nested Database");
                }
                _ => panic!("Expected ChildDatabase block inside toggle"),
            }
        }
        _ => panic!("Expected Toggle block"),
    }
}