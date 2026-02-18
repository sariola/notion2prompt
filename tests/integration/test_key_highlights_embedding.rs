// Focused test for Key Highlights database embedding
// This test specifically checks the embedding process using test fixtures

use notion2prompt::{
    parse_block_response, parse_database_response,
    Block, NotionObject, ObjectGraph, NotionId,
};
use notion2prompt::api::client::ApiResponse;
use reqwest::StatusCode;

#[test]
fn test_key_highlights_fixture_structure() {
    println!("🧪 Testing Key Highlights fixture structure");
    
    // Load the blocks fixture
    let blocks_json = include_str!("../fixtures/api_responses/blocks_aie_agents_nyc.json");
    let blocks_response = serde_json::from_str::<serde_json::Value>(blocks_json)
        .expect("Failed to parse blocks fixture");
    
    // Load the database fixture
    let database_json = include_str!("../fixtures/api_responses/database_key_highlights.json");
    let database_response = serde_json::from_str::<serde_json::Value>(database_json)
        .expect("Failed to parse database fixture");
    
    println!("✅ Both fixtures loaded successfully");
    
    // Extract child database blocks
    let results = blocks_response["results"].as_array()
        .expect("Expected results array in blocks fixture");
    
    let mut child_db_blocks = Vec::new();
    for result in results {
        if result["type"].as_str() == Some("child_database") {
            child_db_blocks.push(result);
        }
    }
    
    println!("📊 Found {} child database blocks", child_db_blocks.len());
    assert!(!child_db_blocks.is_empty(), "Should have at least one child database block");
    
    // Find the Key Highlights block
    let key_highlights_block = child_db_blocks.iter().find(|block| {
        block["child_database"]["title"].as_str() == Some("Key Highlights")
    }).expect("Should find Key Highlights child database block");
    
    let block_id = key_highlights_block["id"].as_str()
        .expect("Block should have ID");
    let database_id = database_response["id"].as_str()
        .expect("Database should have ID");
    
    println!("🔍 Key Highlights Analysis:");
    println!("  Block ID: {}", block_id);
    println!("  Database ID: {}", database_id);
    println!("  IDs match: {}", block_id == database_id);
    
    // This is the critical assertion - the IDs should match for embedding to work
    assert_eq!(block_id, database_id, "Block ID and Database ID should match for proper embedding");
    
    // Check database structure
    let db_properties = database_response["properties"].as_object()
        .expect("Database should have properties");
    
    println!("  Database properties: {:?}", db_properties.keys().collect::<Vec<_>>());
    
    // Test the actual parsing
    println!("🔧 Testing API response parsing...");
    
    // Parse the child database block
    let block_json = serde_json::to_string(key_highlights_block).unwrap();
    let block_api_response = ApiResponse {
        data: block_json,
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };
    let block_result = parse_block_response(block_api_response);
    match block_result {
        Ok(block) => {
            println!("✅ Block parsing successful");
            
            if let Block::ChildDatabase(child_db) = &block {
                println!("  Title: '{}'", child_db.title);
                println!("  ID: {}", child_db.common.id.as_str());
                
                // The block should NOT have an embedded database at this point
                // (that happens during object assembly)
                if child_db.content.as_database().is_some() {
                    println!("  ⚠️  Block already has embedded database (unexpected at parse time)");
                } else {
                    println!("  ✅ Block has no embedded database (expected at parse time)");
                }
            } else {
                panic!("Expected ChildDatabase block, got: {:?}", block.block_type());
            }
        }
        Err(e) => {
            panic!("Failed to parse child database block: {}", e);
        }
    }
    
    // Parse the database
    let database_json = serde_json::to_string(&database_response).unwrap();
    let database_api_response = ApiResponse {
        data: database_json,
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };
    let database_result = parse_database_response(database_api_response);
    match database_result {
        Ok(database) => {
            println!("✅ Database parsing successful");
            println!("  Title: '{}'", database.title());
            println!("  ID: {}", database.id.as_str());
            println!("  Properties: {}", database.properties.len());
            println!("  Sample properties: {:?}", 
                     database.properties.keys().take(3).collect::<Vec<_>>());
        }
        Err(e) => {
            panic!("Failed to parse database: {}", e);
        }
    }
    
    println!("✅ All fixture parsing tests passed");
}

#[test]
fn test_embedding_simulation() {
    println!("🧪 Testing database embedding simulation");
    
    // This test simulates the embedding process using the object graph logic
    
    // Load and parse fixtures
    let blocks_json = include_str!("../fixtures/api_responses/blocks_aie_agents_nyc.json");
    let blocks_response = serde_json::from_str::<serde_json::Value>(blocks_json).unwrap();
    
    let database_json = include_str!("../fixtures/api_responses/database_key_highlights.json");
    let database_response = serde_json::from_str::<serde_json::Value>(database_json).unwrap();
    
    // Parse blocks
    let results = blocks_response["results"].as_array().unwrap();
    let mut parsed_blocks = Vec::new();
    
    for result in results {
        let block_json = serde_json::to_string(result).unwrap();
        let block_api_response = ApiResponse {
            data: block_json,
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        if let Ok(block) = parse_block_response(block_api_response) {
            parsed_blocks.push(Box::new(block));
        }
    }
    
    println!("📦 Parsed {} blocks from fixture", parsed_blocks.len());
    
    // Parse database
    let database_json = serde_json::to_string(&database_response).unwrap();
    let database_api_response = ApiResponse {
        data: database_json,
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };
    let database = parse_database_response(database_api_response).unwrap();
    println!("📊 Parsed database: '{}'", database.title());
    
    // Simulate the object graph assembly process
    let mut graph = ObjectGraph::with_capacity(10);
    
    // Add database to graph
    let db_id = NotionId::parse(database.id.as_str()).unwrap();
    graph = graph.with_object(NotionObject::Database(database));
    
    // Add blocks to graph (simulating the page structure) 
    let page_id = NotionId::parse("1abcd412-8533-8084-9d72-c1cd98f9e8ef").unwrap();
    graph = graph.with_blocks(page_id.clone(), parsed_blocks);
    
    // Check if the child database block to database mapping was created
    let mappings = graph.child_db_block_to_database();
    println!("🔗 Block to database mappings: {}", mappings.len());
    
    for (block_id, mapped_db_id) in mappings {
        println!("  {} -> {}", block_id.as_str(), mapped_db_id.as_str());
    }
    
    // The mapping should exist for our Key Highlights block
    let key_highlights_block_id = NotionId::parse("1abcd412-8533-800c-984c-f7a33514bc7d").unwrap();
    
    if let Some(mapped_db_id) = mappings.get(&key_highlights_block_id) {
        println!("✅ Found mapping for Key Highlights block");
        assert_eq!(*mapped_db_id, db_id, "Mapped database ID should match");
    } else {
        panic!("❌ No mapping found for Key Highlights block");
    }
    
    println!("✅ Embedding simulation test passed");
}

#[test] 
fn test_database_pages_fixture() {
    println!("🧪 Testing database pages fixture");
    
    // Load the pages fixture for Key Highlights
    let pages_json = include_str!("../fixtures/api_responses/pages_key_highlights.json");
    let pages_response = serde_json::from_str::<serde_json::Value>(pages_json)
        .expect("Failed to parse pages fixture");
    
    let results = pages_response["results"].as_array()
        .expect("Expected results array in pages fixture");
    
    println!("📄 Found {} pages in Key Highlights database", results.len());
    
    // Check that we have some sample data
    assert!(!results.is_empty(), "Should have at least one page in Key Highlights database");
    
    for (i, page) in results.iter().enumerate() {
        if let Some(properties) = page["properties"].as_object() {
            // Extract the Name/title property
            if let Some(name_prop) = properties.get("Name") {
                if let Some(title_array) = name_prop["title"].as_array() {
                    if let Some(first_title) = title_array.get(0) {
                        if let Some(content) = first_title["text"]["content"].as_str() {
                            println!("  {}. {}", i + 1, content);
                        }
                    }
                }
            }
        }
    }
    
    println!("✅ Database pages fixture test passed");
}