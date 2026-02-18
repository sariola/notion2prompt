# API Response Test Fixtures

This directory contains real API responses from Notion that are used for testing the parser.

## File Naming Convention

- `page_*.json` - Page API responses
- `database_*.json` - Database API responses  
- `block_*.json` - Block API responses
- `blocks_children_*.json` - Block children API responses
- `database_query_*.json` - Database query API responses

## Adding New Fixtures

1. Capture the raw API response using verbose mode
2. Remove any sensitive data (tokens, private content)
3. Save with descriptive filename following the convention
4. Add corresponding test case in `tests/unit/api_parsing.rs`

## Example Usage

```rust
#[test]
fn parse_page_flow_ai_jetbrains() {
    let json = include_str!("../../fixtures/api_responses/page_flow_ai_jetbrains.json");
    let response: PageResponse = serde_json::from_str(json).unwrap();
    let page = response.to_domain().unwrap();
    
    assert_eq!(page.title().as_str(), "Flow AI x JetBrains");
    assert_eq!(page.id.as_str(), "216cd412-8533-8087-a989-cf37889137c3");
}
```