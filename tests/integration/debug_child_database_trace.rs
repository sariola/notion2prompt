// Comprehensive debugging test for child database embedding
// This test traces the complete flow from API fetch to final output

use notion2prompt::{
    PipelineConfig, Block, NotionObject, NotionId, ApiKey,
};

use std::env;
use std::sync::Arc;

#[tokio::test]
#[ignore] // Requires API key - run with: cargo test --test debug_child_database_trace --ignored
async fn trace_complete_child_database_flow() {
    // Setup logging for detailed tracing
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY environment variable not set");
    let notion_id = "1abcd412853380849d72c1cd98f9e8ef"; // AIE Agents at Work NYC 2025

    println!("🚀 STARTING COMPREHENSIVE CHILD DATABASE TRACE");
    println!("===============================================");
    println!("Target ID: {}", notion_id);

    // Configure for maximum depth and database fetching
    let config = PipelineConfig {
        notion_id: NotionId::parse(notion_id).unwrap(),
        api_key: ApiKey::new(api_key),
        depth: 50,
        limit: 1000,
        always_fetch_databases: true,
        verbose: true,
        ..Default::default()
    };

    println!("⚙️  Configuration:");
    println!("  Depth: {}", config.depth);
    println!("  Limit: {}", config.limit);
    println!("  Always fetch databases: {}", config.always_fetch_databases);

    // STAGE 1: API FETCH
    println!("\n📡 STAGE 1: API FETCH");
    println!("=====================");

    let client = Arc::new(
        NotionClient::with_config(config.api_key.clone(), config.clone())
            .expect("Failed to create Notion client"),
    );

    let fetch_result = fetch_notion_object_recursive(client, &config.notion_id, &config).await;

    match fetch_result {
        Ok(notion_object) => {
            println!("✅ Fetch successful: {}", notion_object.object_type_name());

            // STAGE 2: STRUCTURE ANALYSIS
            println!("\n🔍 STAGE 2: STRUCTURE ANALYSIS");
            println!("===============================");

            analyze_notion_object_structure(&notion_object, 0);

            // STAGE 3: CHILD DATABASE SPECIFIC ANALYSIS
            println!("\n🎯 STAGE 3: CHILD DATABASE ANALYSIS");
            println!("====================================");

            analyze_child_databases(&notion_object);

            // STAGE 4: KEY HIGHLIGHTS SPECIFIC SEARCH
            println!("\n🔑 STAGE 4: KEY HIGHLIGHTS SEARCH");
            println!("==================================");

            search_for_key_highlights(&notion_object);

            // STAGE 5: FORMATTING TEST
            println!("\n📝 STAGE 5: FORMATTING TEST");
            println!("============================");

            test_formatting(&notion_object);
        }
        Err(e) => {
            println!("❌ Fetch failed: {}", e);
            panic!("Failed to fetch Notion object: {}", e);
        }
    }

    println!("\n✅ COMPREHENSIVE TRACE COMPLETE");
}

fn analyze_notion_object_structure(obj: &NotionObject, depth: usize) {
    let indent = "  ".repeat(depth);
    
    match obj {
        NotionObject::Page(page) => {
            println!("{}📄 Page: '{}' (ID: {})", indent, page.title(), page.id.as_str());
            println!("{}   {} blocks total", indent, page.blocks.len());
            
            for (i, block) in page.blocks.iter().enumerate() {
                analyze_block_structure(block, depth + 1, i);
            }
        }
        NotionObject::Database(db) => {
            println!("{}📊 Database: '{}' (ID: {})", indent, db.title(), db.id.as_str());
            println!("{}   {} rows, {} properties", indent, db.pages.len(), db.properties.len());
            
            for (i, row) in db.pages.iter().enumerate() {
                if i < 3 { // Only show first 3 rows to avoid spam
                    println!("{}   📄 Row {}: '{}'", indent, i + 1, row.title());
                }
            }
        }
        NotionObject::Block(block) => {
            analyze_block_structure(&Box::new(block.clone()), depth, 0);
        }
    }
}

fn analyze_block_structure(block: &Box<Block>, depth: usize, index: usize) {
    let indent = "  ".repeat(depth);
    
    match block.as_ref() {
        Block::ChildDatabase(child_db) => {
            println!("{}🎯 #{} ChildDatabase: '{}' (ID: {})", 
                     indent, index + 1, child_db.title, child_db.common.id.as_str());
            
            if let notion2prompt::ChildDatabaseContent::Fetched(ref database) = child_db.content {
                println!("{}    ✅ EMBEDDED DATABASE:", indent);
                println!("{}      Title: '{}'", indent, database.title());
                println!("{}      ID: {}", indent, database.id.as_str());
                println!("{}      Rows: {}", indent, database.pages.len());
                println!("{}      Properties: {}", indent, database.properties.len());
                
                // Show first few property names
                let prop_names: Vec<String> = database.properties.keys().take(3).cloned().collect();
                println!("{}      First properties: [{}]", indent, prop_names.join(", "));
            } else {
                println!("{}    ❌ NO EMBEDDED DATABASE", indent);
            }
        }
        _ => {
            println!("{}📝 #{} {}: {} (ID: {})", 
                     indent, index + 1, block.block_type(), 
                     block.id().as_str(), block.id().as_str());
            
            if block.has_children() && !block.children().is_empty() {
                println!("{}    {} children", indent, block.children().len());
                for (i, child) in block.children().iter().enumerate() {
                    if i < 2 { // Only show first 2 children to avoid spam
                        analyze_block_structure(child, depth + 1, i);
                    }
                }
            }
        }
    }
}

fn analyze_child_databases(obj: &NotionObject) {
    let mut child_db_blocks = Vec::new();
    collect_child_database_blocks(obj, &mut child_db_blocks);
    
    println!("📊 Total ChildDatabase blocks found: {}", child_db_blocks.len());
    
    for (i, child_db) in child_db_blocks.iter().enumerate() {
        println!("  {}. '{}' (ID: {})", i + 1, child_db.title, child_db.common.id.as_str());
        
        if let notion2prompt::ChildDatabaseContent::Fetched(ref database) = child_db.content {
            println!("     ✅ Has embedded database with {} rows", database.pages.len());

            // Show some sample data
            if !database.pages.is_empty() {
                println!("     📋 Sample rows:");
                for (j, page) in database.pages.iter().take(3).enumerate() {
                    println!("       {}. {}", j + 1, page.title());
                }
            }
        } else {
            println!("     ❌ Missing embedded database");
        }
    }
}

fn collect_child_database_blocks(obj: &NotionObject, collector: &mut Vec<&crate::model::ChildDatabaseBlock>) {
    match obj {
        NotionObject::Page(page) => {
            for block in &page.blocks {
                collect_child_database_blocks_from_block(block, collector);
            }
        }
        NotionObject::Database(db) => {
            for page in &db.pages {
                collect_child_database_blocks(&NotionObject::Page(page.clone()), collector);
            }
        }
        NotionObject::Block(block) => {
            collect_child_database_blocks_from_block(&Box::new(block.clone()), collector);
        }
    }
}

fn collect_child_database_blocks_from_block(block: &Box<Block>, collector: &mut Vec<&crate::model::ChildDatabaseBlock>) {
    match block.as_ref() {
        Block::ChildDatabase(child_db) => {
            collector.push(child_db);
        }
        _ => {
            if block.has_children() {
                for child in block.children() {
                    collect_child_database_blocks_from_block(child, collector);
                }
            }
        }
    }
}

fn search_for_key_highlights(obj: &NotionObject) {
    println!("🔍 Searching for 'Key Highlights' specifically...");
    
    let mut found_key_highlights = false;
    let mut all_child_db_titles = Vec::new();
    
    search_key_highlights_recursive(obj, &mut found_key_highlights, &mut all_child_db_titles);
    
    if found_key_highlights {
        println!("✅ FOUND: Key Highlights child database block!");
    } else {
        println!("❌ NOT FOUND: Key Highlights child database block");
        
        println!("📋 All ChildDatabase titles found:");
        for (i, title) in all_child_db_titles.iter().enumerate() {
            println!("  {}. '{}'", i + 1, title);
        }
        
        // Check for partial matches
        let partial_matches: Vec<&String> = all_child_db_titles.iter()
            .filter(|title| title.to_lowercase().contains("key") || 
                           title.to_lowercase().contains("highlight"))
            .collect();
            
        if !partial_matches.is_empty() {
            println!("🔍 Partial matches found:");
            for title in partial_matches {
                println!("  - '{}'", title);
            }
        }
    }
}

fn search_key_highlights_recursive(
    obj: &NotionObject, 
    found: &mut bool, 
    all_titles: &mut Vec<String>
) {
    match obj {
        NotionObject::Page(page) => {
            for block in &page.blocks {
                search_key_highlights_in_block(block, found, all_titles);
            }
        }
        NotionObject::Database(db) => {
            for page in &db.pages {
                search_key_highlights_recursive(&NotionObject::Page(page.clone()), found, all_titles);
            }
        }
        NotionObject::Block(block) => {
            search_key_highlights_in_block(&Box::new(block.clone()), found, all_titles);
        }
    }
}

fn search_key_highlights_in_block(
    block: &Box<Block>, 
    found: &mut bool, 
    all_titles: &mut Vec<String>
) {
    match block.as_ref() {
        Block::ChildDatabase(child_db) => {
            all_titles.push(child_db.title.clone());
            
            let title_lower = child_db.title.to_lowercase();
            if title_lower.contains("key") && title_lower.contains("highlight") {
                *found = true;
                println!("🎯 MATCH: Found Key Highlights block: '{}'", child_db.title);
                
                if let notion2prompt::ChildDatabaseContent::Fetched(ref database) = child_db.content {
                    println!("  ✅ Block has embedded database with {} rows", database.pages.len());
                } else {
                    println!("  ❌ Block has NO embedded database");
                }
            }
        }
        _ => {
            if block.has_children() {
                for child in block.children() {
                    search_key_highlights_in_block(child, found, all_titles);
                }
            }
        }
    }
}

fn test_formatting(obj: &NotionObject) {
    use notion2prompt::presentation::{MarkdownPresenter, NotionPresenter};
    
    println!("🎨 Testing markdown formatting...");
    
    let presenter = MarkdownPresenter;
    
    match presenter.format(obj) {
        Ok(formatted) => {
            let lines = formatted.lines().count();
            let contains_table = formatted.contains("|") && formatted.contains("---");
            let contains_highlights = formatted.to_lowercase().contains("key") && 
                                    formatted.to_lowercase().contains("highlight");
            
            println!("✅ Formatting successful:");
            println!("  - {} lines generated", lines);
            println!("  - Contains tables: {}", contains_table);
            println!("  - Contains Key Highlights: {}", contains_highlights);
            
            if contains_highlights {
                println!("🔍 Key Highlights context in formatted output:");
                for (i, line) in formatted.lines().enumerate() {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("key") && line_lower.contains("highlight") {
                        // Show context around the match
                        let start = i.saturating_sub(2);
                        let end = (i + 3).min(formatted.lines().count());
                        
                        println!("  Lines {}-{}:", start + 1, end);
                        for (j, context_line) in formatted.lines().skip(start).take(end - start).enumerate() {
                            let line_num = start + j + 1;
                            let marker = if start + j == i { " >>> " } else { "     " };
                            println!("{}{}:{}", marker, line_num, context_line);
                        }
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Formatting failed: {}", e);
        }
    }
}

#[test]
fn test_with_fixtures() {
    println!("🧪 Testing with fixtures...");
    
    // Test with the blocks fixture
    let blocks_fixture = include_str!("../fixtures/api_responses/blocks_aie_agents_nyc.json");
    
    println!("📋 Analyzing blocks fixture...");
    
    if blocks_fixture.contains("child_database") {
        println!("✅ Fixture contains child_database blocks");
        
        // Count occurrences
        let child_db_count = blocks_fixture.matches("child_database").count();
        println!("  Found {} child_database references", child_db_count);
        
        // Check for Key Highlights
        if blocks_fixture.to_lowercase().contains("key") && 
           blocks_fixture.to_lowercase().contains("highlight") {
            println!("✅ Fixture contains Key Highlights references");
        } else {
            println!("❌ Fixture does NOT contain Key Highlights references");
        }
    } else {
        println!("❌ Fixture does NOT contain child_database blocks");
    }
    
    // Test database fixture if available
    let db_fixture_path = "../fixtures/api_responses/database_key_highlights.json";
    if let Ok(db_fixture) = std::fs::read_to_string(format!("tests/fixtures/api_responses/database_key_highlights.json")) {
        println!("📊 Analyzing Key Highlights database fixture...");
        
        // Basic structure check
        if db_fixture.contains("\"object\": \"database\"") {
            println!("✅ Valid database fixture structure");
        } else {
            println!("❌ Invalid database fixture structure");
        }
        
        if db_fixture.contains("properties") {
            let prop_count = db_fixture.matches("\"id\":").count();
            println!("  Contains {} property definitions", prop_count);
        }
    } else {
        println!("❌ Key Highlights database fixture not found");
    }
}