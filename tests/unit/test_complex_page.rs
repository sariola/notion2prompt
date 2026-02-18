#[cfg(test)]
mod tests {
    use notion2prompt::{parse_page_response, parse_blocks_pagination};
    use notion2prompt::api::client::ApiResponse;
    use reqwest::StatusCode;
    
    #[test]
    fn test_parse_aie_agents_page() {
        // Test parsing the page with proper title extraction
        let page_json = include_str!("../fixtures/api_responses/page_aie_agents_nyc.json");
        let api_response = ApiResponse {
            data: page_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let page = parse_page_response(api_response).unwrap();
        
        assert_eq!(page.title().as_str(), "AIE Agents at Work - NYC 2025 ");
        assert_eq!(page.id.as_str(), "1abcd412853380849d72c1cd98f9e8ef");
        assert!(page.parent.is_some());
    }
    
    #[test]
    fn test_parse_aie_blocks_with_child_database() {
        // Test parsing blocks including child database
        let blocks_json = include_str!("../fixtures/api_responses/blocks_aie_agents_nyc.json");
        let api_response = ApiResponse {
            data: blocks_json.to_string(),
            status: StatusCode::OK,
            url: "test_url".to_string(),
        };
        let result = parse_blocks_pagination(api_response).unwrap();
        
        assert_eq!(result.results.len(), 3);
        
        // Check the child database block
        match &result.results[0] {
            notion2prompt::Block::ChildDatabase(db) => {
                assert_eq!(db.title, "Key Highlights");
                assert_eq!(db.common.id.as_str(), "1abcd4128533800c984cf7a33514bc7d");
            }
            _ => panic!("Expected ChildDatabase block"),
        }
        
        // Check the heading block
        match &result.results[2] {
            notion2prompt::Block::Heading2(h) => {
                assert_eq!(h.content.rich_text.len(), 1);
                assert_eq!(h.content.rich_text[0].plain_text, "Videos (not from conference)");
            }
            _ => panic!("Expected Heading2 block"),
        }
    }
}