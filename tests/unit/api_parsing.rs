// tests/unit/api_parsing.rs
//! Unit tests for API response parsing

use notion2prompt::api::responses::{PageResponse, DatabaseResponse, BlockResponse};
use notion2prompt::model::{Page, Database, Block};
use serde_json;

#[cfg(test)]
mod page_parsing_tests {
    use super::*;
    
    #[test]
    fn parse_page_flow_ai_jetbrains() {
        // Load the fixture
        let json = include_str!("../fixtures/api_responses/page_flow_ai_jetbrains.json");
        
        // Parse as API response
        let response: PageResponse = serde_json::from_str(json)
            .expect("Failed to parse PageResponse");
        
        // Verify API response fields
        assert_eq!(response.object, "page");
        assert_eq!(response.id, "216cd412-8533-8087-a989-cf37889137c3");
        assert_eq!(response.url, "https://www.notion.so/Flow-AI-x-JetBrains-216cd41285338087a989cf37889137c3");
        assert!(!response.archived);
        
        // Convert to domain model
        let page = response.to_domain()
            .expect("Failed to convert to domain model");
        
        // Verify domain model
        assert_eq!(page.title().as_str(), "Flow AI x JetBrains");
        assert_eq!(page.id.as_str(), "216cd412-8533-8087-a989-cf37889137c3");
        assert_eq!(page.url, "https://www.notion.so/Flow-AI-x-JetBrains-216cd41285338087a989cf37889137c3");
        assert!(!page.archived);
        assert!(page.parent.is_some());
    }
    
    #[test]
    fn parse_page_with_missing_title() {
        let json = r#"{
            "object": "page",
            "id": "test-page-id",
            "created_time": "2025-06-20T00:00:00.000Z",
            "last_edited_time": "2025-06-20T00:00:00.000Z",
            "created_by": {"object": "user", "id": "user-id"},
            "last_edited_by": {"object": "user", "id": "user-id"},
            "parent": {"type": "workspace"},
            "archived": false,
            "properties": {},
            "url": "https://www.notion.so/test-page"
        }"#;
        
        let response: PageResponse = serde_json::from_str(json)
            .expect("Failed to parse PageResponse");
        
        let page = response.to_domain()
            .expect("Failed to convert to domain model");
        
        // Should default to "Untitled"
        assert_eq!(page.title().as_str(), "Untitled");
    }
    
    #[test]
    fn parse_page_with_complex_properties() {
        let json = r#"{
            "object": "page",
            "id": "complex-page-id",
            "created_time": "2025-06-20T00:00:00.000Z",
            "last_edited_time": "2025-06-20T00:00:00.000Z",
            "created_by": {"object": "user", "id": "user-id"},
            "last_edited_by": {"object": "user", "id": "user-id"},
            "parent": {"type": "database_id", "database_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"},
            "archived": false,
            "properties": {
                "title": {
                    "id": "title",
                    "type": "title",
                    "title": [
                        {
                            "type": "text",
                            "text": {"content": "Complex Page", "link": null},
                            "plain_text": "Complex Page",
                            "href": null,
                            "annotations": {
                                "bold": true,
                                "italic": false,
                                "strikethrough": false,
                                "underline": false,
                                "code": false,
                                "color": "default"
                            }
                        }
                    ]
                },
                "Status": {
                    "id": "status",
                    "type": "select",
                    "select": {
                        "id": "active-id",
                        "name": "Active",
                        "color": "green"
                    }
                },
                "Tags": {
                    "id": "tags",
                    "type": "multi_select",
                    "multi_select": [
                        {"id": "tag1", "name": "Important", "color": "red"},
                        {"id": "tag2", "name": "Review", "color": "blue"}
                    ]
                }
            },
            "url": "https://www.notion.so/complex-page"
        }"#;
        
        let response: PageResponse = serde_json::from_str(json)
            .expect("Failed to parse PageResponse with complex properties");
        
        let page = response.to_domain()
            .expect("Failed to convert complex page");
        
        assert_eq!(page.title().as_str(), "Complex Page");
        assert!(matches!(page.parent, Some(notion2prompt::model::Parent::Database { .. })));
    }
}

#[cfg(test)]
mod database_parsing_tests {
    use super::*;
    
    #[test]
    fn parse_basic_database() {
        let json = r#"{
            "object": "database",
            "id": "test-database-id",
            "created_time": "2025-06-20T00:00:00.000Z",
            "last_edited_time": "2025-06-20T00:00:00.000Z",
            "created_by": {"object": "user", "id": "user-id"},
            "last_edited_by": {"object": "user", "id": "user-id"},
            "title": [
                {
                    "type": "text",
                    "text": {"content": "Test Database", "link": null},
                    "plain_text": "Test Database",
                    "href": null,
                    "annotations": null
                }
            ],
            "parent": {"type": "page_id", "page_id": "b2c3d4e5-f678-9012-abcd-ef1234567890"},
            "properties": {
                "Name": {
                    "id": "title",
                    "name": "Name",
                    "type": "title",
                    "title": {}
                },
                "Status": {
                    "id": "status",
                    "name": "Status",
                    "type": "select",
                    "select": {
                        "options": [
                            {"id": "1", "name": "To Do", "color": "red"},
                            {"id": "2", "name": "In Progress", "color": "yellow"},
                            {"id": "3", "name": "Done", "color": "green"}
                        ]
                    }
                }
            },
            "url": "https://www.notion.so/test-database",
            "archived": false
        }"#;
        
        let response: DatabaseResponse = serde_json::from_str(json)
            .expect("Failed to parse DatabaseResponse");
        
        let database = response.to_domain()
            .expect("Failed to convert to domain model");
        
        assert_eq!(database.title.as_plain_text(), "Test Database");
        assert_eq!(database.id.as_str(), "test-database-id");
        assert!(!database.archived);
    }
}

#[cfg(test)]
mod block_parsing_tests {
    use super::*;
    
    #[test]
    fn parse_paragraph_block() {
        let json = r#"{
            "object": "block",
            "id": "paragraph-block-id",
            "parent": {"type": "page_id", "page_id": "b2c3d4e5-f678-9012-abcd-ef1234567890"},
            "created_time": "2025-06-20T00:00:00.000Z",
            "last_edited_time": "2025-06-20T00:00:00.000Z",
            "created_by": {"object": "user", "id": "user-id"},
            "last_edited_by": {"object": "user", "id": "user-id"},
            "has_children": false,
            "archived": false,
            "type": "paragraph",
            "paragraph": {
                "rich_text": [
                    {
                        "type": "text",
                        "text": {"content": "This is a paragraph.", "link": null},
                        "plain_text": "This is a paragraph.",
                        "href": null,
                        "annotations": {
                            "bold": false,
                            "italic": false,
                            "strikethrough": false,
                            "underline": false,
                            "code": false,
                            "color": "default"
                        }
                    }
                ],
                "color": "default"
            }
        }"#;
        
        let response: BlockResponse = serde_json::from_str(json)
            .expect("Failed to parse BlockResponse");
        
        assert_eq!(response.object, "block");
        assert_eq!(response.id, "paragraph-block-id");
        assert_eq!(response.block_type, "paragraph");
        assert!(!response.has_children);
        
        let block = response.to_domain()
            .expect("Failed to convert to domain model");
        
        // Verify it created a block (even if simplified)
        assert!(matches!(block, Block::Paragraph(_)));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use notion2prompt::api::parser::parse_api_response;
    use notion2prompt::api::client::ApiResponse;
    
    #[test]
    fn parse_api_response_for_page() {
        let json = include_str!("../fixtures/api_responses/page_flow_ai_jetbrains.json");
        
        let api_response = ApiResponse {
            data: json.to_string(),
            status: reqwest::StatusCode::OK,
            url: "https://api.notion.com/v1/pages/test".to_string(),
        };
        
        let page: Page = parse_api_response(api_response)
            .expect("Failed to parse page through parse_api_response");
        
        assert_eq!(page.title().as_str(), "Flow AI x JetBrains");
    }
}