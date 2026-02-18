use notion2prompt::formatting::{format_blocks_with_config, FormatConfig};
use notion2prompt::model::{Block, ChildDatabaseBlock, BlockCommon, Database, Page};
use notion2prompt::types::BlockId;
use std::collections::HashMap;

#[test]
fn test_child_database_with_embedded_content() {
    // Create a test database
    let mut database = Database::new("db-123".to_string());
    database.set_title(vec![]); // Empty for now
    
    // Create a child database block with embedded database
    let mut child_db_block = ChildDatabaseBlock {
        common: BlockCommon {
            id: BlockId::new("block-123".to_string()),
            has_children: false,
            created_time: chrono::Utc::now(),
            last_edited_time: chrono::Utc::now(),
            children: vec![],
        },
        title: "Key Highlights".to_string(),
        content: notion2prompt::model::blocks::ChildDatabaseContent::Fetched(Box::new(database)),
    };
    
    let blocks = vec![Box::new(Block::ChildDatabase(child_db_block))];
    
    // Format without external databases
    let config = FormatConfig {
        app_config: None,
        enable_sanitization: false,
        enable_parallel: false,
        databases: None,
    };
    
    let result = format_blocks_with_config(&blocks, &config).unwrap();
    
    // Should show database content, not just the placeholder
    assert!(!result.contains("🗄️ [[Key Highlights]]"), "Should not show placeholder when database is embedded");
    assert!(result.contains("Key Highlights"), "Should contain the database title");
}

#[test] 
fn test_child_database_without_embedded_content() {
    // Create a child database block without embedded database
    let child_db_block = ChildDatabaseBlock {
        common: BlockCommon {
            id: BlockId::new("block-123".to_string()),
            has_children: false,
            created_time: chrono::Utc::now(),
            last_edited_time: chrono::Utc::now(),
            children: vec![],
        },
        title: "Key Highlights".to_string(),
        content: notion2prompt::model::blocks::ChildDatabaseContent::NotFetched,
    };
    
    let blocks = vec![Box::new(Block::ChildDatabase(child_db_block))];
    
    // Format without external databases
    let config = FormatConfig {
        app_config: None,
        enable_sanitization: false,
        enable_parallel: false,
        databases: None,
    };
    
    let result = format_blocks_with_config(&blocks, &config).unwrap();
    
    // Should show placeholder when no database is embedded
    assert!(result.contains("🗄️ [[Key Highlights]]"), "Should show placeholder when database is not embedded");
}

#[test]
fn test_child_database_with_external_lookup() {
    // Create a child database block without embedded database
    let child_db_block = ChildDatabaseBlock {
        common: BlockCommon {
            id: BlockId::new("block-123".to_string()),
            has_children: false,
            created_time: chrono::Utc::now(),
            last_edited_time: chrono::Utc::now(),
            children: vec![],
        },
        title: "Key Highlights".to_string(),
        content: notion2prompt::model::blocks::ChildDatabaseContent::NotFetched,
    };
    
    let blocks = vec![Box::new(Block::ChildDatabase(child_db_block))];
    
    // Create external database map
    let mut databases = HashMap::new();
    let mut database = Database::new("block-123".to_string()); // Same ID as block
    database.set_title(vec![]);
    databases.insert("block-123".to_string(), database);
    
    // Format with external databases
    let config = FormatConfig {
        app_config: None,
        enable_sanitization: false,
        enable_parallel: false,
        databases: Some(&databases),
    };
    
    let result = format_blocks_with_config(&blocks, &config).unwrap();
    
    // Should show database content from external lookup
    assert!(!result.contains("🗄️ [[Key Highlights]]"), "Should not show placeholder when database found in external lookup");
}