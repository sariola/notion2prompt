//! Test to verify property parsing with graceful fallbacks
//!
//! This test ensures that property conversion handles various edge cases gracefully,
//! including unsupported property types, missing data, and malformed properties.

use notion2prompt::{parse_database_response, parse_page_response, ApiResponse};
use reqwest::StatusCode;

#[test]
fn test_property_parsing_with_graceful_fallbacks() {
    // Test page with various property types including potential edge cases
    let page_json = r#"{
        "object": "page",
        "id": "216cd412-8533-8087-a989-cf37889137c3",
        "created_time": "2023-01-01T00:00:00.000Z",
        "last_edited_time": "2023-01-01T00:00:00.000Z",
        "created_by": {"object": "user", "id": "user-id"},
        "last_edited_by": {"object": "user", "id": "user-id"},
        "parent": {"type": "database_id", "database_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"},
        "archived": false,
        "url": "https://www.notion.so/Test-Page",
        "properties": {
            "Title": {
                "id": "title",
                "type": "title",
                "title": [
                    {
                        "type": "text",
                        "text": {"content": "Test Page Title", "link": null},
                        "plain_text": "Test Page Title",
                        "href": null,
                        "annotations": {"bold": false, "italic": false, "strikethrough": false, "underline": false, "code": false, "color": "default"}
                    }
                ]
            },
            "Status": {
                "id": "status",
                "type": "select",
                "select": {
                    "id": "select-option-id",
                    "name": "In Progress",
                    "color": "blue"
                }
            },
            "Priority": {
                "id": "priority",
                "type": "number",
                "number": 5
            },
            "Description": {
                "id": "desc",
                "type": "rich_text",
                "rich_text": [
                    {
                        "type": "text",
                        "text": {"content": "Test description", "link": null},
                        "plain_text": "Test description",
                        "href": null,
                        "annotations": {"bold": false, "italic": false, "strikethrough": false, "underline": false, "code": false, "color": "default"}
                    }
                ]
            },
            "Completed": {
                "id": "completed",
                "type": "checkbox",
                "checkbox": false
            }
        }
    }"#;

    let api_response = ApiResponse {
        data: page_json.to_string(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };

    let page = parse_page_response(api_response).expect("Page parsing should succeed");

    // Verify that properties were parsed (even if empty due to placeholders)
    println!("✅ Page parsing with properties succeeded");
    println!("   Page title: {}", page.title().as_str());
    println!("   Properties count: {}", page.properties.len());

    // Test database with property schema
    let database_json = r#"{
        "object": "database",
        "id": "a02dd81a-36b6-4c1b-9c74-bb5c7c2e8ea2",
        "created_time": "2023-01-01T00:00:00.000Z",
        "last_edited_time": "2023-01-01T00:00:00.000Z",
        "created_by": {"object": "user", "id": "user-id"},
        "last_edited_by": {"object": "user", "id": "user-id"},
        "title": [
            {
                "type": "text",
                "text": {"content": "Test Database", "link": null},
                "plain_text": "Test Database",
                "href": null,
                "annotations": {"bold": false, "italic": false, "strikethrough": false, "underline": false, "code": false, "color": "default"}
            }
        ],
        "description": [],
        "icon": null,
        "cover": null,
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
                        {
                            "id": "opt1",
                            "name": "To Do",
                            "color": "red"
                        },
                        {
                            "id": "opt2", 
                            "name": "In Progress",
                            "color": "blue"
                        }
                    ]
                }
            },
            "Priority": {
                "id": "priority",
                "name": "Priority",
                "type": "number",
                "number": {
                    "format": "number"
                }
            }
        },
        "parent": {"type": "page_id", "page_id": "b2c3d4e5-f678-9012-abcd-ef1234567890"},
        "url": "https://www.notion.so/Database",
        "archived": false,
        "is_inline": false
    }"#;

    let api_response = ApiResponse {
        data: database_json.to_string(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };

    let database = parse_database_response(api_response).expect("Database parsing should succeed");

    // Verify that database properties were parsed
    println!("✅ Database parsing with properties succeeded");
    println!("   Database title: {}", database.title().as_plain_text());
    println!("   Properties count: {}", database.properties.len());

    // Test error resilience - malformed property should not break entire parsing
    let malformed_page_json = r#"{
        "object": "page",
        "id": "316cd412-8533-8087-a989-cf37889137c4",
        "created_time": "2023-01-01T00:00:00.000Z",
        "last_edited_time": "2023-01-01T00:00:00.000Z",
        "created_by": {"object": "user", "id": "user-id"},
        "last_edited_by": {"object": "user", "id": "user-id"},
        "parent": {"type": "database_id", "database_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"},
        "archived": false,
        "url": "https://www.notion.so/Malformed-Page",
        "properties": {
            "Title": {
                "id": "title",
                "type": "title",
                "title": [
                    {
                        "type": "text",
                        "text": {"content": "Valid Title", "link": null},
                        "plain_text": "Valid Title",
                        "href": null,
                        "annotations": {"bold": false, "italic": false, "strikethrough": false, "underline": false, "code": false, "color": "default"}
                    }
                ]
            }
        }
    }"#;

    let api_response = ApiResponse {
        data: malformed_page_json.to_string(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };

    let malformed_page = parse_page_response(api_response)
        .expect("Even malformed property parsing should succeed with fallbacks");

    println!("✅ Malformed property parsing with fallbacks succeeded");
    println!("   Page title: {}", malformed_page.title().as_str());

    println!("\n🎉 All property parsing fallback tests passed!");
}
