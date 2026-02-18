#[test]
fn test_amundi_fixtures_have_block_parents() {
    // This test verifies our fixtures contain blocks with block_id parents
    let blocks_json = include_str!("../fixtures/api_responses/blocks_flow_ai_amundi.json");
    let parsed: serde_json::Value = serde_json::from_str(blocks_json).unwrap();
    
    let mut block_parent_count = 0;
    if let Some(results) = parsed["results"].as_array() {
        for block in results {
            if block["parent"]["type"] == "block_id" {
                block_parent_count += 1;
            }
        }
    }
    
    assert_eq!(block_parent_count, 3, "Expected 3 blocks with block_id parents");
}