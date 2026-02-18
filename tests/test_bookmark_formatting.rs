use notion2prompt::{
    render_blocks, Block, BlockCommon, BlockId, BookmarkBlock, RenderContext, RichTextItem,
};

fn create_bookmark_block(url: &str, caption: &str) -> Block {
    let caption_items = if caption.is_empty() {
        vec![]
    } else {
        vec![RichTextItem::plain_text(caption)]
    };

    Block::Bookmark(BookmarkBlock {
        common: BlockCommon {
            id: BlockId::parse("12345678-1234-1234-1234-123456789abc").unwrap(),
            has_children: false,
            archived: false,
            children: vec![],
        },
        url: url.to_string(),
        caption: caption_items,
    })
}

#[test]
fn test_bookmark_formatting_with_caption() {
    let block = create_bookmark_block("https://example.com", "Example website");
    let blocks = vec![block];

    let config = RenderContext::default();
    let result = render_blocks(&blocks, &config).unwrap();

    assert_eq!(result, "[🔖 https://example.com - Example website]\n");
}

#[test]
fn test_bookmark_formatting_without_caption() {
    let block = create_bookmark_block("https://notion.so", "");
    let blocks = vec![block];

    let config = RenderContext::default();
    let result = render_blocks(&blocks, &config).unwrap();

    assert_eq!(result, "[🔖 https://notion.so]\n");
}

#[test]
fn test_multiple_bookmarks() {
    let blocks = vec![
        create_bookmark_block("https://github.com", "GitHub"),
        create_bookmark_block("https://rust-lang.org", "Rust Programming Language"),
        create_bookmark_block("https://notion.so", ""),
    ];

    let config = RenderContext::default();
    let result = render_blocks(&blocks, &config).unwrap();

    let expected = "\
[🔖 https://github.com - GitHub]
[🔖 https://rust-lang.org - Rust Programming Language]
[🔖 https://notion.so]
";

    assert_eq!(result, expected);
}
