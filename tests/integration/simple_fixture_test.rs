// Simple test to verify fixture structure and basic parsing
// This test checks if our test fixtures are properly structured

use notion2prompt::{parse_block_response, parse_database_response, ApiResponse, Block};
use reqwest::StatusCode;

#[test]
fn test_fixture_parsing() {
    println!("🧪 Testing fixture parsing");

    // Test 1: Parse blocks fixture
    let blocks_json = include_str!("../fixtures/api_responses/blocks_aie_agents_nyc.json");
    let blocks_response: serde_json::Value =
        serde_json::from_str(blocks_json).expect("Failed to parse blocks fixture");

    println!("✅ Blocks fixture loaded");

    // Find child database blocks
    let results = blocks_response["results"]
        .as_array()
        .expect("Expected results array");

    let mut child_db_blocks = Vec::new();
    for result in results {
        if result["type"].as_str() == Some("child_database") {
            child_db_blocks.push(result);
        }
    }

    println!("📊 Found {} child database blocks", child_db_blocks.len());
    assert!(
        !child_db_blocks.is_empty(),
        "Should have child database blocks"
    );

    // Test 2: Parse Key Highlights database fixture
    let database_json = include_str!("../fixtures/api_responses/database_key_highlights.json");
    let database_response: serde_json::Value =
        serde_json::from_str(database_json).expect("Failed to parse database fixture");

    println!("✅ Database fixture loaded");

    // Test 3: Verify ID matching
    let key_highlights_block = child_db_blocks
        .iter()
        .find(|block| block["child_database"]["title"].as_str() == Some("Key Highlights"))
        .expect("Should find Key Highlights block");

    let block_id = key_highlights_block["id"].as_str().unwrap();
    let database_id = database_response["id"].as_str().unwrap();

    println!("🔍 ID Verification:");
    println!("  Block ID: {}", block_id);
    println!("  Database ID: {}", database_id);
    println!("  Match: {}", block_id == database_id);

    assert_eq!(block_id, database_id, "Block and database IDs should match");

    // Test 4: Parse with actual parsers
    println!("🔧 Testing actual API parsers...");

    let block_json = serde_json::to_string(key_highlights_block).unwrap();
    let api_response = ApiResponse {
        data: block_json.clone(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };
    match parse_block_response(api_response) {
        Ok(block) => {
            println!("✅ Block parsing successful");
            if let Block::ChildDatabase(child_db) = &block {
                println!("  Title: '{}'", child_db.title);
                println!(
                    "  Embedded database: {}",
                    child_db.content.as_database().is_some()
                );
            }
        }
        Err(e) => panic!("Block parsing failed: {}", e),
    }

    let database_json = serde_json::to_string(&database_response).unwrap();
    let api_response = ApiResponse {
        data: database_json.clone(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };
    match parse_database_response(api_response) {
        Ok(database) => {
            println!("✅ Database parsing successful");
            println!("  Title: '{}'", database.title());
            println!("  Properties: {}", database.properties.len());
        }
        Err(e) => panic!("Database parsing failed: {}", e),
    }

    println!("✅ All fixture tests passed");
}

#[test]
fn test_database_pages_structure() {
    println!("🧪 Testing database pages structure");

    let pages_json = include_str!("../fixtures/api_responses/pages_key_highlights.json");
    let pages_response: serde_json::Value =
        serde_json::from_str(pages_json).expect("Failed to parse pages fixture");

    let results = pages_response["results"]
        .as_array()
        .expect("Expected results array");

    println!("📄 Found {} pages", results.len());
    assert!(!results.is_empty(), "Should have at least one page");

    // Check page structure
    for (i, page) in results.iter().enumerate() {
        if let Some(properties) = page["properties"].as_object() {
            println!("  Page {}: {} properties", i + 1, properties.len());

            // Look for Name property
            if let Some(name_prop) = properties.get("Name") {
                if let Some(title_array) = name_prop["title"].as_array() {
                    if let Some(first_title) = title_array.first() {
                        if let Some(content) = first_title["text"]["content"].as_str() {
                            println!("    Name: '{}'", content);
                        }
                    }
                }
            }
        }
    }

    println!("✅ Database pages structure test passed");
}
