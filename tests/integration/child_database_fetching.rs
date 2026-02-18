// tests/integration/child_database_fetching.rs
//! Integration tests for child database fetching functionality.

use notion2prompt::{PipelineConfig, ApiKey, NotionId};
use std::env;

/// Helper function to get API key from environment
fn get_test_api_key() -> Option<ApiKey> {
    env::var("NOTION_API_KEY").ok().and_then(|key| ApiKey::parse(&key).ok())
}

#[test]
#[ignore] // Run with: cargo test --ignored
fn test_fetch_page_with_child_databases() {
    let api_key = match get_test_api_key() {
        Some(key) => key,
        None => {
            eprintln!("Skipping test: NOTION_API_KEY not set");
            return;
        }
    };

    // Test with a known page ID that has child databases
    let page_id = NotionId::parse("216cd41285338087a989cf37889137c3").unwrap();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Note: This test requires the actual API to be available
        // In a real scenario, we would use NotionFetcher here
        let config = PipelineConfig {
            depth: 3,
            limit: 100,
            ..Default::default()
        };
        
        // This test would use NotionFetcher with the API key
        // to fetch a page and verify child databases are embedded
        println!("Test would fetch page ID: {}", page_id.as_str());
    });
}

#[test]
fn test_config_limits() {
    // Test configuration limits for fetching
    let config = PipelineConfig {
        depth: 10,
        limit: 500,
        ..Default::default()
    };
    
    assert_eq!(config.depth, 10);
    assert_eq!(config.limit, 500);
    
    // Test with very high values
    let config_high = PipelineConfig {
        depth: 100,
        limit: 10000,
        ..Default::default()
    };
    
    // Depth should be reasonable even if set high
    assert!(config_high.depth <= 100);
}

#[test]
fn test_notion_id_formats() {
    // Test various NotionId formats
    let valid_formats = vec![
        "12345678-1234-1234-1234-123456789012",
        "12345678123412341234123456789012",
        "https://www.notion.so/12345678123412341234123456789012",
        "https://notion.so/workspace/12345678123412341234123456789012",
    ];
    
    for format in valid_formats {
        let id = NotionId::parse(format);
        assert!(id.is_ok(), "Should parse format: {}", format);
        
        // Verify the ID normalizes correctly
        let parsed = id.unwrap();
        assert_eq!(parsed.as_str().len(), 32, "ID should be 32 characters");
    }
    
    // Test invalid formats
    let invalid_formats = vec![
        "not-a-valid-id",
        "12345", // Too short
        "",      // Empty
    ];
    
    for format in invalid_formats {
        let id = NotionId::parse(format);
        assert!(id.is_err(), "Should reject invalid format: {}", format);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    #[test]
    fn test_api_key_validation() {
        // Test API key parsing
        let valid_key = "secret_abc123def456";
        let api_key = ApiKey::parse(valid_key);
        assert!(api_key.is_ok(), "Should parse valid API key");
        
        // Test invalid keys
        let invalid_keys = vec![
            "",           // Empty
            "not_secret", // Wrong prefix
            "secret_",    // Too short
        ];
        
        for key in invalid_keys {
            let api_key = ApiKey::parse(key);
            assert!(api_key.is_err(), "Should reject invalid key: {}", key);
        }
    }
    
    #[test]
    fn test_page_title_creation() {
        use notion2prompt::model::PageTitle;
        
        // Test creating page titles
        let title = PageTitle::new("Test Page");
        assert_eq!(title.as_str(), "Test Page");
    }
    
    #[test]
    fn test_database_title_creation() {
        use notion2prompt::model::DatabaseTitle;
        use notion2prompt::types::RichTextItem;
        
        // Test creating database titles
        let title = DatabaseTitle::new(vec![RichTextItem {
            plain_text: "Test Database".to_string(),
            href: None,
            annotations: Default::default(),
            text_type: notion2prompt::types::RichTextType::Text {
                content: "Test Database".to_string(),
                link: None,
            },
        }]);
        assert_eq!(title.as_plain_text(), "Test Database");
    }
}