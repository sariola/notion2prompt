#[cfg(test)]
mod tests {
    use notion2prompt::ApiResponse;
    use notion2prompt::{
        parse_blocks_pagination, parse_database_response, parse_pages_pagination, Block,
    };
    use reqwest::StatusCode;
    use std::fs;

    #[test]
    fn test_child_database_block_parsing() {
        // Test that the child database block is correctly parsed from fixtures
        let blocks_json = include_str!("../fixtures/api_responses/blocks_aie_agents_nyc.json");
        let api_response = ApiResponse {
            data: blocks_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let result = parse_blocks_pagination(api_response).unwrap();

        assert_eq!(result.results.len(), 3);

        // Find the child database block
        let child_db_block = result
            .results
            .iter()
            .find(|block| matches!(block, Block::ChildDatabase(_)))
            .expect("Should contain a child database block");

        if let Block::ChildDatabase(db) = child_db_block {
            assert_eq!(db.title, "Key Highlights");
            println!("✅ Found child database block: {}", db.title);

            // For now, just verify the block exists and has the right title
            // The formatting test will be added once we have a working format function
            println!("📄 Child database block found with title: {}", db.title);
        }
    }

    #[test]
    fn test_database_and_pages_fixtures() {
        // Test parsing the new database and pages fixtures
        let database_json = include_str!("../fixtures/api_responses/database_key_highlights.json");
        let pages_json = include_str!("../fixtures/api_responses/pages_key_highlights.json");

        let database_api_response = ApiResponse {
            data: database_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let database = parse_database_response(database_api_response).unwrap();

        let pages_api_response = ApiResponse {
            data: pages_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let pages_result = parse_pages_pagination(pages_api_response).unwrap();

        assert_eq!(database.title().as_plain_text(), "Key Highlights");
        assert_eq!(pages_result.results.len(), 3);

        // Check the pages content
        let page_titles: Vec<String> = pages_result
            .results
            .iter()
            .map(|p| p.title().as_str().to_string())
            .collect();

        assert!(
            page_titles.contains(&"Agent Engineering is the new Software Engineering".to_string())
        );
        assert!(
            page_titles.contains(&"Evaluation frameworks critical for agent success".to_string())
        );
        assert!(
            page_titles.contains(&"Enterprise AI must show real ROI, not just demos".to_string())
        );

        println!(
            "✅ Database fixture parsed successfully: {}",
            database.title().as_plain_text()
        );
        println!(
            "✅ Pages fixture parsed successfully: {} pages",
            pages_result.results.len()
        );
    }

    #[test]
    fn test_child_database_in_integration() {
        // This test demonstrates how child databases should work in the full pipeline
        // We'll simulate the embedding process that should happen during object assembly

        let blocks_json = include_str!("../fixtures/api_responses/blocks_aie_agents_nyc.json");
        let database_json = include_str!("../fixtures/api_responses/database_key_highlights.json");
        let pages_json = include_str!("../fixtures/api_responses/pages_key_highlights.json");

        let blocks_api_response = ApiResponse {
            data: blocks_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let blocks_result = parse_blocks_pagination(blocks_api_response).unwrap();

        let database_api_response = ApiResponse {
            data: database_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let database = parse_database_response(database_api_response).unwrap();

        let pages_api_response = ApiResponse {
            data: pages_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let pages_result = parse_pages_pagination(pages_api_response).unwrap();

        // Find the child database block
        let child_db_block = blocks_result
            .results
            .iter()
            .find(|b| matches!(b, notion2prompt::Block::ChildDatabase(_)))
            .expect("Should have child database block");

        // In a real scenario, the ObjectGraph would embed the database here
        // For now, we just verify the components exist
        if let notion2prompt::Block::ChildDatabase(db_block) = child_db_block {
            assert_eq!(db_block.title, database.title().as_plain_text());
            println!(
                "✅ Child database block title matches database title: {}",
                db_block.title
            );
        }

        // Verify the database has the expected content
        assert_eq!(pages_result.results.len(), 3);
        println!(
            "✅ Database has {} pages of content",
            pages_result.results.len()
        );

        // Save a sample of what the output should look like
        let expected_output = format!(
            "## {}\n\n| Name | Category | Priority |\n|------|----------|----------|\n{}\n",
            database.title().as_plain_text(),
            pages_result
                .results
                .iter()
                .map(|page| {
                    let title = page.title().as_str();
                    // Extract category and priority from properties (simplified)
                    format!("| {} | | |", title)
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        fs::write("test_expected_child_database_output.md", &expected_output)
            .expect("Failed to write expected output");

        println!("✅ Expected output saved to test_expected_child_database_output.md");
    }
}
