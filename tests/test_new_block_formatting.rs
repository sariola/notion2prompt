//! Test to verify new block types have proper template compatibility
//!
//! This test ensures that the newly added block types (Bookmark, Embed, Equation, ChildPage)
//! are properly handled by the formatting system and produce expected Markdown output.

use notion2prompt::{parse_block_response, render_blocks, ApiResponse, RenderContext};
use reqwest::StatusCode;

#[test]
fn test_new_block_types_formatting() {
    // Test bookmark block formatting
    let bookmark_json = r#"{
        "object": "block",
        "id": "216cd412-8533-8087-a989-cf37889137c3",
        "parent": {"type": "page_id", "page_id": "414cd412-8533-8087-a989-cf37889137c5"},
        "created_time": "2025-06-20T00:00:00.000Z",
        "last_edited_time": "2025-06-20T00:00:00.000Z",
        "created_by": {"object": "user", "id": "user-id"},
        "last_edited_by": {"object": "user", "id": "user-id"},
        "has_children": false,
        "archived": false,
        "type": "bookmark",
        "bookmark": {
            "url": "https://notion.so",
            "caption": [
                {
                    "type": "text",
                    "text": {"content": "Official Notion website", "link": null},
                    "plain_text": "Official Notion website",
                    "href": null,
                    "annotations": {"bold": false, "italic": false, "strikethrough": false, "underline": false, "code": false, "color": "default"}
                }
            ]
        }
    }"#;

    let http_result = ApiResponse {
        data: bookmark_json.to_string(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };

    let bookmark_block =
        parse_block_response(http_result).expect("Bookmark parsing should succeed");

    // Test that bookmark formats correctly
    let blocks = vec![bookmark_block];
    let config = RenderContext::default();
    let output = render_blocks(&blocks, &config).expect("Formatting should succeed");

    assert!(output.contains("[🔖 https://notion.so - Official Notion website]"));
    println!("✅ Bookmark block formatting: {}", output.trim());

    // Test equation block formatting
    let equation_json = r#"{
        "object": "block",
        "id": "316cd412-8533-8087-a989-cf37889137c4",
        "parent": {"type": "page_id", "page_id": "414cd412-8533-8087-a989-cf37889137c5"},
        "created_time": "2025-06-20T00:00:00.000Z",
        "last_edited_time": "2025-06-20T00:00:00.000Z",
        "created_by": {"object": "user", "id": "user-id"},
        "last_edited_by": {"object": "user", "id": "user-id"},
        "has_children": false,
        "archived": false,
        "type": "equation",
        "equation": {
            "expression": "E = mc^2"
        }
    }"#;

    let http_result = ApiResponse {
        data: equation_json.to_string(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };

    let equation_block =
        parse_block_response(http_result).expect("Equation parsing should succeed");

    // Test that equation formats correctly
    let blocks = vec![equation_block];
    let output = render_blocks(&blocks, &config).expect("Formatting should succeed");

    assert!(output.contains("$$\nE = mc^2\n$$"));
    println!("✅ Equation block formatting: {}", output.trim());

    // Test embed block formatting
    let embed_json = r#"{
        "object": "block",
        "id": "416cd412-8533-8087-a989-cf37889137c5",
        "parent": {"type": "page_id", "page_id": "414cd412-8533-8087-a989-cf37889137c5"},
        "created_time": "2025-06-20T00:00:00.000Z",
        "last_edited_time": "2025-06-20T00:00:00.000Z",
        "created_by": {"object": "user", "id": "user-id"},
        "last_edited_by": {"object": "user", "id": "user-id"},
        "has_children": false,
        "archived": false,
        "type": "embed",
        "embed": {
            "url": "https://www.youtube.com/watch?v=example"
        }
    }"#;

    let http_result = ApiResponse {
        data: embed_json.to_string(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };

    let embed_block = parse_block_response(http_result).expect("Embed parsing should succeed");

    // Test that embed formats correctly
    let blocks = vec![embed_block];
    let output = render_blocks(&blocks, &config).expect("Formatting should succeed");

    assert!(output.contains("[Embed: https://www.youtube.com/watch?v=example]"));
    println!("✅ Embed block formatting: {}", output.trim());

    // Test child page block formatting
    let child_page_json = r#"{
        "object": "block",
        "id": "516cd412-8533-8087-a989-cf37889137c6",
        "parent": {"type": "page_id", "page_id": "414cd412-8533-8087-a989-cf37889137c5"},
        "created_time": "2025-06-20T00:00:00.000Z",
        "last_edited_time": "2025-06-20T00:00:00.000Z",
        "created_by": {"object": "user", "id": "user-id"},
        "last_edited_by": {"object": "user", "id": "user-id"},
        "has_children": false,
        "archived": false,
        "type": "child_page",
        "child_page": {
            "title": "Meeting Notes"
        }
    }"#;

    let http_result = ApiResponse {
        data: child_page_json.to_string(),
        status: StatusCode::OK,
        url: "test_url".to_string(),
    };

    let child_page_block =
        parse_block_response(http_result).expect("Child page parsing should succeed");

    // Test that child page formats correctly
    let blocks = vec![child_page_block];
    let output = render_blocks(&blocks, &config).expect("Formatting should succeed");

    assert!(output.contains("📄 [[Meeting Notes]]"));
    println!("✅ Child page block formatting: {}", output.trim());

    println!("\n🎉 All new block types have proper template compatibility!");
}
