#[cfg(test)]
mod tests {
    use notion2prompt::api::parse_blocks_pagination;
    
    #[test]
    fn test_parse_blocks_pagination() {
        let json = include_str!("../fixtures/api_responses/blocks_flow_ai_jetbrains.json");
        let result = parse_blocks_pagination(json, "test_url").unwrap();
        
        assert_eq!(result.object, "list");
        assert_eq!(result.results.len(), 1);
        assert!(!result.has_more);
        assert!(result.next_cursor.is_none());
        
        // Check the block content
        let block = &result.results[0];
        match block {
            notion2prompt::Block::Paragraph(p) => {
                assert_eq!(p.content.rich_text.len(), 1);
                assert_eq!(p.content.rich_text[0].plain_text, "This is a collaboration between Flow AI and JetBrains.");
            }
            _ => panic!("Expected paragraph block"),
        }
    }
}