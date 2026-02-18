// tests/snapshot_tests.rs
//! Comprehensive snapshot tests for the formatting pipeline.
//!
//! Uses the `insta` crate for snapshot management. Snapshots are stored as `.snap`
//! files under `tests/snapshots/` and reviewed with `cargo insta review`.

use notion2prompt::{
    // Formatting
    compose_database_summary,
    compose_page_markdown,
    // API parsing
    parse_blocks_pagination,
    parse_database_response,
    render_blocks,
    // Domain types
    Annotations,
    ApiResponse,
    // Domain model
    Block,
    BlockCommon,
    BlockId,
    // Block types
    BookmarkBlock,
    BreadcrumbBlock,
    BulletedListItemBlock,
    CalloutBlock,
    ChildDatabaseBlock,
    ChildDatabaseContent,
    ChildPageBlock,
    CodeBlock,
    Color,
    ColumnBlock,
    ColumnListBlock,
    Database,
    DatabaseId,
    DatabaseProperty,
    DatabasePropertyType,
    DatabaseTitle,
    DateValue,
    DividerBlock,
    EmbedBlock,
    EquationBlock,
    EquationData,
    ExternalFile,
    FileBlock,
    FileObject,
    Heading1Block,
    Heading2Block,
    Heading3Block,
    Icon,
    ImageBlock,
    LinkPreviewBlock,
    LinkToPageBlock,
    NumberFormat,
    NumberedListItemBlock,
    Page,
    PageId,
    PageTitle,
    ParagraphBlock,
    Parent,
    PdfBlock,
    PropertyName,
    PropertyTypeValue,
    PropertyValue,
    QuoteBlock,
    RenderContext,
    RichTextItem,
    RichTextType,
    SelectOption,
    SyncedBlock,
    SyncedFrom,
    TableBlock,
    TableOfContentsBlock,
    TableRowBlock,
    TextBlockContent,
    ToDoBlock,
    ToggleBlock,
    UnsupportedBlock,
    VideoBlock,
};
use reqwest::StatusCode;
use std::collections::HashMap;

// =============================================================================
// Helper functions — ergonomic constructors for test data
// =============================================================================

const TEST_BLOCK_ID: &str = "00000000-0000-0000-0000-000000000001";
const TEST_PAGE_ID: &str = "00000000-0000-0000-0000-000000000002";
const TEST_DB_ID: &str = "00000000-0000-0000-0000-000000000003";

/// Create a BlockCommon with a deterministic test ID.
fn common() -> BlockCommon {
    BlockCommon::new(BlockId::parse(TEST_BLOCK_ID).unwrap())
}

/// Create a BlockCommon with specific children.
fn common_with_children(children: Vec<Block>) -> BlockCommon {
    BlockCommon::new(BlockId::parse(TEST_BLOCK_ID).unwrap()).with_children(children)
}

/// Create plain-text rich text items.
fn rich(text: &str) -> Vec<RichTextItem> {
    vec![RichTextItem::plain_text(text)]
}

/// Create a RichTextItem with specific annotations.
fn annotated(
    text: &str,
    bold: bool,
    italic: bool,
    strikethrough: bool,
    underline: bool,
    code: bool,
    color: Color,
) -> RichTextItem {
    RichTextItem {
        text_type: RichTextType::Text {
            content: text.to_string(),
            link: None,
        },
        annotations: Annotations {
            bold,
            italic,
            strikethrough,
            underline,
            code,
            color,
        },
        plain_text: text.to_string(),
        href: None,
    }
}

/// Create a TextBlockContent from plain text.
fn text_content(text: &str) -> TextBlockContent {
    TextBlockContent {
        rich_text: rich(text),
        color: Color::Default,
    }
}

/// Create a TextBlockContent from rich text items.
fn text_content_rich(items: Vec<RichTextItem>) -> TextBlockContent {
    TextBlockContent {
        rich_text: items,
        color: Color::Default,
    }
}

// --- Block constructors ---

fn paragraph(text: &str) -> Block {
    Block::Paragraph(ParagraphBlock {
        common: common(),
        content: text_content(text),
    })
}

fn paragraph_rich(items: Vec<RichTextItem>) -> Block {
    Block::Paragraph(ParagraphBlock {
        common: common(),
        content: text_content_rich(items),
    })
}

fn heading1(text: &str) -> Block {
    Block::Heading1(Heading1Block {
        common: common(),
        content: text_content(text),
    })
}

fn heading2(text: &str) -> Block {
    Block::Heading2(Heading2Block {
        common: common(),
        content: text_content(text),
    })
}

fn heading3(text: &str) -> Block {
    Block::Heading3(Heading3Block {
        common: common(),
        content: text_content(text),
    })
}

fn bulleted(text: &str) -> Block {
    Block::BulletedListItem(BulletedListItemBlock {
        common: common(),
        content: text_content(text),
    })
}

fn bulleted_with_children(text: &str, children: Vec<Block>) -> Block {
    Block::BulletedListItem(BulletedListItemBlock {
        common: common_with_children(children),
        content: text_content(text),
    })
}

fn numbered(text: &str) -> Block {
    Block::NumberedListItem(NumberedListItemBlock {
        common: common(),
        content: text_content(text),
    })
}

fn numbered_with_children(text: &str, children: Vec<Block>) -> Block {
    Block::NumberedListItem(NumberedListItemBlock {
        common: common_with_children(children),
        content: text_content(text),
    })
}

fn todo(text: &str, checked: bool) -> Block {
    Block::ToDo(ToDoBlock {
        common: common(),
        content: text_content(text),
        checked,
    })
}

fn toggle(text: &str, children: Vec<Block>) -> Block {
    Block::Toggle(ToggleBlock {
        common: common_with_children(children),
        content: text_content(text),
    })
}

fn quote(text: &str) -> Block {
    Block::Quote(QuoteBlock {
        common: common(),
        content: text_content(text),
    })
}

fn quote_with_children(text: &str, children: Vec<Block>) -> Block {
    Block::Quote(QuoteBlock {
        common: common_with_children(children),
        content: text_content(text),
    })
}

fn callout(text: &str, emoji: &str) -> Block {
    Block::Callout(CalloutBlock {
        common: common(),
        icon: Some(Icon::Emoji {
            emoji: emoji.to_string(),
        }),
        content: text_content(text),
    })
}

fn callout_with_children(text: &str, emoji: &str, children: Vec<Block>) -> Block {
    Block::Callout(CalloutBlock {
        common: common_with_children(children),
        icon: Some(Icon::Emoji {
            emoji: emoji.to_string(),
        }),
        content: text_content(text),
    })
}

fn code(text: &str, language: &str) -> Block {
    Block::Code(CodeBlock {
        common: common(),
        language: language.to_string(),
        caption: vec![],
        content: text_content(text),
    })
}

fn divider() -> Block {
    Block::Divider(DividerBlock { common: common() })
}

fn table_of_contents() -> Block {
    Block::TableOfContents(TableOfContentsBlock { common: common() })
}

fn breadcrumb() -> Block {
    Block::Breadcrumb(BreadcrumbBlock { common: common() })
}

fn bookmark(url: &str) -> Block {
    Block::Bookmark(BookmarkBlock {
        common: common(),
        url: url.to_string(),
        caption: vec![],
    })
}

fn bookmark_with_caption(url: &str, caption: &str) -> Block {
    Block::Bookmark(BookmarkBlock {
        common: common(),
        url: url.to_string(),
        caption: rich(caption),
    })
}

fn image_external(url: &str) -> Block {
    Block::Image(ImageBlock {
        common: common(),
        image: FileObject::External {
            external: ExternalFile {
                url: url.to_string(),
            },
        },
        caption: vec![],
    })
}

fn image_with_caption(url: &str, caption: &str) -> Block {
    Block::Image(ImageBlock {
        common: common(),
        image: FileObject::External {
            external: ExternalFile {
                url: url.to_string(),
            },
        },
        caption: rich(caption),
    })
}

fn video(url: &str) -> Block {
    Block::Video(VideoBlock {
        common: common(),
        video: FileObject::External {
            external: ExternalFile {
                url: url.to_string(),
            },
        },
        caption: vec![],
    })
}

fn file_block(url: &str) -> Block {
    Block::File(FileBlock {
        common: common(),
        file: FileObject::External {
            external: ExternalFile {
                url: url.to_string(),
            },
        },
        caption: vec![],
    })
}

fn pdf(url: &str) -> Block {
    Block::Pdf(PdfBlock {
        common: common(),
        pdf: FileObject::External {
            external: ExternalFile {
                url: url.to_string(),
            },
        },
        caption: vec![],
    })
}

fn embed(url: &str) -> Block {
    Block::Embed(EmbedBlock {
        common: common(),
        url: url.to_string(),
    })
}

fn link_preview(url: &str) -> Block {
    Block::LinkPreview(LinkPreviewBlock {
        common: common(),
        url: url.to_string(),
    })
}

fn child_page(title: &str) -> Block {
    Block::ChildPage(ChildPageBlock {
        common: common(),
        title: title.to_string(),
    })
}

fn child_database(title: &str) -> Block {
    Block::ChildDatabase(ChildDatabaseBlock {
        common: common(),
        title: title.to_string(),
        content: ChildDatabaseContent::NotFetched,
    })
}

fn equation(expr: &str) -> Block {
    Block::Equation(EquationBlock {
        common: common(),
        expression: expr.to_string(),
    })
}

fn link_to_page(page_id: &str) -> Block {
    Block::LinkToPage(LinkToPageBlock {
        common: common(),
        page_id: PageId::parse(page_id).unwrap(),
    })
}

fn table_with_rows(has_header: bool, width: usize, rows: Vec<Block>) -> Block {
    Block::Table(TableBlock {
        common: common_with_children(rows),
        table_width: width,
        has_column_header: has_header,
        has_row_header: false,
    })
}

fn table_row(cells: Vec<&str>) -> Block {
    Block::TableRow(TableRowBlock {
        common: common(),
        cells: cells.into_iter().map(rich).collect(),
    })
}

fn column_list(columns: Vec<Vec<Block>>) -> Block {
    let column_blocks: Vec<Block> = columns
        .into_iter()
        .map(|col_children| {
            Block::Column(ColumnBlock {
                common: common_with_children(col_children),
            })
        })
        .collect();
    Block::ColumnList(ColumnListBlock {
        common: common_with_children(column_blocks),
    })
}

fn synced(children: Vec<Block>) -> Block {
    Block::Synced(SyncedBlock {
        common: common_with_children(children),
        synced_from: None,
    })
}

fn synced_reference(source_id: &str, children: Vec<Block>) -> Block {
    Block::Synced(SyncedBlock {
        common: common_with_children(children),
        synced_from: Some(SyncedFrom {
            block_id: BlockId::parse(source_id).unwrap(),
        }),
    })
}

fn unsupported(block_type: &str) -> Block {
    Block::Unsupported(UnsupportedBlock {
        common: common(),
        block_type: block_type.to_string(),
    })
}

// --- Page/Database constructors ---

fn simple_page(title: &str, blocks: Vec<Block>) -> Page {
    Page {
        id: PageId::parse(TEST_PAGE_ID).unwrap(),
        title: PageTitle::new(title),
        url: format!("https://www.notion.so/{}", TEST_PAGE_ID),
        blocks,
        properties: HashMap::new(),
        parent: Some(Parent::Workspace),
        archived: false,
    }
}

fn page_with_properties(
    title: &str,
    blocks: Vec<Block>,
    props: Vec<(&str, PropertyTypeValue)>,
) -> Page {
    let mut properties = HashMap::new();
    for (name, value) in props {
        properties.insert(
            PropertyName::new(name),
            PropertyValue {
                id: PropertyName::new(name),
                type_specific_value: value,
            },
        );
    }
    Page {
        id: PageId::parse(TEST_PAGE_ID).unwrap(),
        title: PageTitle::new(title),
        url: format!("https://www.notion.so/{}", TEST_PAGE_ID),
        blocks,
        properties,
        parent: Some(Parent::Workspace),
        archived: false,
    }
}

fn simple_database(title: &str, pages: Vec<Page>) -> Database {
    Database {
        id: DatabaseId::parse(TEST_DB_ID).unwrap(),
        title: DatabaseTitle::new(rich(title)),
        url: format!("https://www.notion.so/{}", TEST_DB_ID),
        pages,
        properties: HashMap::new(),
        parent: Some(Parent::Workspace),
        archived: false,
    }
}

fn database_with_schema(
    title: &str,
    schema: Vec<(&str, DatabasePropertyType)>,
    pages: Vec<Page>,
) -> Database {
    let mut properties = HashMap::new();
    for (name, prop_type) in schema {
        properties.insert(
            PropertyName::new(name),
            DatabaseProperty {
                id: PropertyName::new(name),
                name: PropertyName::new(name),
                property_type: prop_type,
            },
        );
    }
    Database {
        id: DatabaseId::parse(TEST_DB_ID).unwrap(),
        title: DatabaseTitle::new(rich(title)),
        url: format!("https://www.notion.so/{}", TEST_DB_ID),
        pages,
        properties,
        parent: Some(Parent::Workspace),
        archived: false,
    }
}

/// Normalize a markdown table section by sorting data rows (preserving header + separator).
/// This makes database summary snapshots deterministic despite HashMap ordering.
fn normalize_table_rows(input: &str) -> String {
    let mut result = Vec::new();
    let mut table_rows: Vec<String> = Vec::new();
    let mut in_table = false;
    let mut header_count = 0;

    for line in input.lines() {
        if line.starts_with('|') {
            if !in_table {
                in_table = true;
                header_count = 0;
            }
            header_count += 1;
            if header_count <= 2 {
                // Header row + separator — keep in order
                result.push(line.to_string());
            } else {
                table_rows.push(line.to_string());
            }
        } else {
            if in_table && !table_rows.is_empty() {
                table_rows.sort();
                result.append(&mut table_rows);
            }
            in_table = false;
            header_count = 0;
            result.push(line.to_string());
        }
    }
    // Flush remaining table rows
    if !table_rows.is_empty() {
        table_rows.sort();
        result.extend(table_rows);
    }
    result.join("\n")
}

/// Normalize page markdown by sorting property bullet lines within the "## Properties" section.
/// This makes page property snapshots deterministic despite HashMap ordering.
fn normalize_property_lines(input: &str) -> String {
    let mut result = Vec::new();
    let mut prop_lines: Vec<String> = Vec::new();
    let mut in_properties = false;

    for line in input.lines() {
        if line == "## Properties" {
            in_properties = true;
            result.push(line.to_string());
            continue;
        }
        if in_properties {
            if line.starts_with("- **") {
                prop_lines.push(line.to_string());
                continue;
            }
            if line.trim().is_empty() && prop_lines.is_empty() {
                // Blank line between heading and first property — pass through
                result.push(line.to_string());
                continue;
            }
            // End of properties section — flush sorted
            if !prop_lines.is_empty() {
                prop_lines.sort();
                result.append(&mut prop_lines);
            }
            in_properties = false;
            result.push(line.to_string());
        } else {
            result.push(line.to_string());
        }
    }
    // Flush remaining
    if !prop_lines.is_empty() {
        prop_lines.sort();
        result.extend(prop_lines);
    }
    result.join("\n")
}

/// Render blocks with default context, unwrapping the result.
fn render(blocks: &[Block]) -> String {
    render_blocks(blocks, &RenderContext::default()).unwrap()
}

/// Render a single block.
fn render_one(block: &Block) -> String {
    render(&[block.clone()])
}

// =============================================================================
// Test modules
// =============================================================================

mod blocks {
    use super::*;

    #[test]
    fn paragraph_block() {
        insta::assert_snapshot!(render_one(&paragraph("Hello, world!")));
    }

    #[test]
    fn paragraph_empty() {
        insta::assert_snapshot!(render_one(&paragraph("")));
    }

    #[test]
    fn heading1_block() {
        insta::assert_snapshot!(render_one(&heading1("Main Title")));
    }

    #[test]
    fn heading2_block() {
        insta::assert_snapshot!(render_one(&heading2("Section Title")));
    }

    #[test]
    fn heading3_block() {
        insta::assert_snapshot!(render_one(&heading3("Subsection Title")));
    }

    #[test]
    fn bulleted_list_item() {
        insta::assert_snapshot!(render_one(&bulleted("List item")));
    }

    #[test]
    fn numbered_list_item() {
        insta::assert_snapshot!(render_one(&numbered("First item")));
    }

    #[test]
    fn todo_checked() {
        insta::assert_snapshot!(render_one(&todo("Completed task", true)));
    }

    #[test]
    fn todo_unchecked() {
        insta::assert_snapshot!(render_one(&todo("Pending task", false)));
    }

    #[test]
    fn toggle_block() {
        insta::assert_snapshot!(render_one(&toggle(
            "Toggle header",
            vec![paragraph("Hidden content")]
        )));
    }

    #[test]
    fn quote_block() {
        insta::assert_snapshot!(render_one(&quote("A wise saying")));
    }

    #[test]
    fn callout_block() {
        insta::assert_snapshot!(render_one(&callout("Important notice", "💡")));
    }

    #[test]
    fn code_rust() {
        insta::assert_snapshot!(render_one(&code(
            "fn main() {\n    println!(\"hello\");\n}",
            "rust"
        )));
    }

    #[test]
    fn code_python() {
        insta::assert_snapshot!(render_one(&code(
            "def hello():\n    print(\"hello\")",
            "python"
        )));
    }

    #[test]
    fn divider_block() {
        insta::assert_snapshot!(render_one(&divider()));
    }

    #[test]
    fn breadcrumb_block() {
        insta::assert_snapshot!(render_one(&breadcrumb()));
    }

    #[test]
    fn bookmark_block() {
        insta::assert_snapshot!(render_one(&bookmark("https://example.com")));
    }

    #[test]
    fn bookmark_block_with_caption() {
        insta::assert_snapshot!(render_one(&bookmark_with_caption(
            "https://example.com",
            "Example Site"
        )));
    }

    #[test]
    fn image_external_block() {
        insta::assert_snapshot!(render_one(&image_external("https://example.com/image.png")));
    }

    #[test]
    fn image_with_caption_block() {
        insta::assert_snapshot!(render_one(&image_with_caption(
            "https://example.com/photo.jpg",
            "A beautiful photo"
        )));
    }

    #[test]
    fn video_block() {
        insta::assert_snapshot!(render_one(&video("https://example.com/video.mp4")));
    }

    #[test]
    fn file_block_test() {
        insta::assert_snapshot!(render_one(&file_block("https://example.com/doc.pdf")));
    }

    #[test]
    fn pdf_block() {
        insta::assert_snapshot!(render_one(&pdf("https://example.com/paper.pdf")));
    }

    #[test]
    fn embed_block() {
        insta::assert_snapshot!(render_one(&embed("https://twitter.com/status/123")));
    }

    #[test]
    fn link_preview_block() {
        insta::assert_snapshot!(render_one(&link_preview("https://github.com/repo")));
    }

    #[test]
    fn child_page_block() {
        insta::assert_snapshot!(render_one(&child_page("My Sub-Page")));
    }

    #[test]
    fn child_database_block() {
        insta::assert_snapshot!(render_one(&child_database("My Database")));
    }

    #[test]
    fn equation_block() {
        insta::assert_snapshot!(render_one(&equation("E = mc^2")));
    }

    #[test]
    fn link_to_page_block() {
        insta::assert_snapshot!(render_one(&link_to_page(
            "aabbccdd-aabb-ccdd-aabb-ccddaabbccdd"
        )));
    }

    #[test]
    fn table_of_contents_with_headings() {
        let blocks = vec![
            table_of_contents(),
            heading1("Introduction"),
            heading2("Overview"),
            heading1("Conclusion"),
        ];
        insta::assert_snapshot!(render(&blocks));
    }

    #[test]
    fn column_list_block() {
        insta::assert_snapshot!(render_one(&column_list(vec![
            vec![paragraph("Column 1 content")],
            vec![paragraph("Column 2 content")],
        ])));
    }

    #[test]
    fn synced_block() {
        insta::assert_snapshot!(render_one(&synced(vec![paragraph("Synced content")])));
    }

    #[test]
    fn synced_reference_block() {
        insta::assert_snapshot!(render_one(&synced_reference(
            "aabbccdd-aabb-ccdd-aabb-ccddaabbccdd",
            vec![paragraph("Referenced content")]
        )));
    }

    #[test]
    fn unsupported_block() {
        insta::assert_snapshot!(render_one(&unsupported("new_block_type")));
    }
}

mod rich_text {
    use super::*;

    #[test]
    fn bold_text() {
        let item = annotated(
            "Bold text",
            true,
            false,
            false,
            false,
            false,
            Color::Default,
        );
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn italic_text() {
        let item = annotated(
            "Italic text",
            false,
            true,
            false,
            false,
            false,
            Color::Default,
        );
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn strikethrough_text() {
        let item = annotated(
            "Struck through",
            false,
            false,
            true,
            false,
            false,
            Color::Default,
        );
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn underline_text() {
        let item = annotated(
            "Underlined text",
            false,
            false,
            false,
            true,
            false,
            Color::Default,
        );
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn inline_code() {
        let item = annotated(
            "let x = 42",
            false,
            false,
            false,
            false,
            true,
            Color::Default,
        );
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn combined_bold_italic() {
        let item = annotated(
            "Bold and italic",
            true,
            true,
            false,
            false,
            false,
            Color::Default,
        );
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn text_with_link() {
        let item = RichTextItem {
            text_type: RichTextType::Text {
                content: "Click here".to_string(),
                link: None,
            },
            annotations: Annotations::default(),
            plain_text: "Click here".to_string(),
            href: Some("https://example.com".to_string()),
        };
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn mixed_rich_text() {
        let items = vec![
            RichTextItem::plain_text("Normal "),
            annotated("bold", true, false, false, false, false, Color::Default),
            RichTextItem::plain_text(" and "),
            annotated("italic", false, true, false, false, false, Color::Default),
            RichTextItem::plain_text(" text."),
        ];
        insta::assert_snapshot!(render_one(&paragraph_rich(items)));
    }

    #[test]
    fn all_annotations_combined() {
        let item = annotated("Everything", true, true, true, true, false, Color::Default);
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }

    #[test]
    fn inline_equation() {
        let item = RichTextItem {
            text_type: RichTextType::Equation(EquationData {
                expression: "x^2 + y^2 = r^2".to_string(),
            }),
            annotations: Annotations::default(),
            plain_text: "x^2 + y^2 = r^2".to_string(),
            href: None,
        };
        insta::assert_snapshot!(render_one(&paragraph_rich(vec![item])));
    }
}

mod lists {
    use super::*;

    #[test]
    fn single_bulleted_item() {
        insta::assert_snapshot!(render(&[bulleted("Single bullet")]));
    }

    #[test]
    fn multiple_bulleted_items() {
        insta::assert_snapshot!(render(&[
            bulleted("First"),
            bulleted("Second"),
            bulleted("Third"),
        ]));
    }

    #[test]
    fn single_numbered_item() {
        insta::assert_snapshot!(render(&[numbered("Only item")]));
    }

    #[test]
    fn multiple_numbered_items() {
        insta::assert_snapshot!(render(&[
            numbered("First"),
            numbered("Second"),
            numbered("Third"),
        ]));
    }

    #[test]
    fn nested_bulleted_list() {
        insta::assert_snapshot!(render(&[
            bulleted_with_children("Parent", vec![bulleted("Child A"), bulleted("Child B")]),
            bulleted("Sibling"),
        ]));
    }

    #[test]
    fn nested_numbered_list() {
        insta::assert_snapshot!(render(&[
            numbered_with_children("Parent", vec![numbered("Sub 1"), numbered("Sub 2")]),
            numbered("Next parent"),
        ]));
    }

    #[test]
    fn mixed_list_types() {
        insta::assert_snapshot!(render(&[
            bulleted("Bullet A"),
            bulleted("Bullet B"),
            numbered("Number 1"),
            numbered("Number 2"),
        ]));
    }

    #[test]
    fn todo_list_mixed() {
        insta::assert_snapshot!(render(&[
            todo("Buy groceries", true),
            todo("Clean house", false),
            todo("Read book", true),
            todo("Exercise", false),
        ]));
    }
}

mod nesting {
    use super::*;

    #[test]
    fn toggle_with_paragraph_children() {
        insta::assert_snapshot!(render_one(&toggle(
            "Click to expand",
            vec![
                paragraph("First paragraph inside toggle"),
                paragraph("Second paragraph inside toggle"),
            ]
        )));
    }

    #[test]
    fn toggle_with_nested_toggles() {
        insta::assert_snapshot!(render_one(&toggle(
            "Outer toggle",
            vec![toggle(
                "Inner toggle",
                vec![paragraph("Deeply nested content")]
            )]
        )));
    }

    #[test]
    fn quote_block_with_children() {
        insta::assert_snapshot!(render_one(&quote_with_children(
            "Main quote",
            vec![paragraph("Attribution or follow-up")]
        )));
    }

    #[test]
    fn callout_block_with_children() {
        insta::assert_snapshot!(render_one(&callout_with_children(
            "Warning message",
            "⚠️",
            vec![
                paragraph("Details about the warning"),
                bulleted("Step 1"),
                bulleted("Step 2"),
            ]
        )));
    }

    #[test]
    fn bulleted_with_nested_bullets_3_levels() {
        insta::assert_snapshot!(render(&[bulleted_with_children(
            "Level 1",
            vec![bulleted_with_children("Level 2", vec![bulleted("Level 3")])]
        )]));
    }

    #[test]
    fn numbered_with_nested_numbered() {
        insta::assert_snapshot!(render(&[
            numbered_with_children(
                "Chapter 1",
                vec![numbered("Section 1.1"), numbered("Section 1.2")]
            ),
            numbered_with_children("Chapter 2", vec![numbered("Section 2.1")]),
        ]));
    }

    #[test]
    fn heading_then_paragraphs_then_heading() {
        insta::assert_snapshot!(render(&[
            heading1("Introduction"),
            paragraph("First paragraph under intro."),
            paragraph("Second paragraph under intro."),
            heading2("Details"),
            paragraph("Content under details."),
        ]));
    }

    #[test]
    fn deep_nesting_5_levels() {
        insta::assert_snapshot!(render_one(&toggle(
            "Level 1",
            vec![toggle(
                "Level 2",
                vec![toggle(
                    "Level 3",
                    vec![toggle(
                        "Level 4",
                        vec![toggle("Level 5", vec![paragraph("Bottom")])]
                    )]
                )]
            )]
        )));
    }
}

mod tables {
    use super::*;

    #[test]
    fn simple_table_with_header() {
        let t = table_with_rows(
            true,
            3,
            vec![
                table_row(vec!["Name", "Age", "City"]),
                table_row(vec!["Alice", "30", "NYC"]),
                table_row(vec!["Bob", "25", "SF"]),
            ],
        );
        insta::assert_snapshot!(render_one(&t));
    }

    #[test]
    fn simple_table_without_header() {
        let t = table_with_rows(
            false,
            2,
            vec![table_row(vec!["A1", "B1"]), table_row(vec!["A2", "B2"])],
        );
        insta::assert_snapshot!(render_one(&t));
    }

    #[test]
    fn table_with_empty_cells() {
        let t = table_with_rows(
            true,
            3,
            vec![
                table_row(vec!["Header 1", "Header 2", "Header 3"]),
                table_row(vec!["Data", "", "More data"]),
                table_row(vec!["", "Only middle", ""]),
            ],
        );
        insta::assert_snapshot!(render_one(&t));
    }

    #[test]
    fn table_single_column() {
        let t = table_with_rows(
            true,
            1,
            vec![
                table_row(vec!["Items"]),
                table_row(vec!["Apple"]),
                table_row(vec!["Banana"]),
            ],
        );
        insta::assert_snapshot!(render_one(&t));
    }

    #[test]
    fn table_many_columns() {
        let t = table_with_rows(
            true,
            5,
            vec![
                table_row(vec!["A", "B", "C", "D", "E"]),
                table_row(vec!["1", "2", "3", "4", "5"]),
            ],
        );
        insta::assert_snapshot!(render_one(&t));
    }
}

mod databases {
    use super::*;

    #[test]
    fn database_summary_simple() {
        let db = database_with_schema(
            "Project Tracker",
            vec![
                ("Name", DatabasePropertyType::Title),
                ("Status", DatabasePropertyType::Status { options: vec![] }),
                ("Priority", DatabasePropertyType::Select { options: vec![] }),
            ],
            vec![simple_page("Task 1", vec![]), simple_page("Task 2", vec![])],
        );
        insta::assert_snapshot!(normalize_table_rows(
            &compose_database_summary(&db).unwrap()
        ));
    }

    #[test]
    fn database_summary_empty() {
        let db = database_with_schema(
            "Empty Database",
            vec![("Title", DatabasePropertyType::Title)],
            vec![],
        );
        insta::assert_snapshot!(normalize_table_rows(
            &compose_database_summary(&db).unwrap()
        ));
    }

    #[test]
    fn database_summary_many_property_types() {
        let db = database_with_schema(
            "Comprehensive DB",
            vec![
                ("Name", DatabasePropertyType::Title),
                (
                    "Amount",
                    DatabasePropertyType::Number {
                        format: NumberFormat::Dollar,
                    },
                ),
                ("Due Date", DatabasePropertyType::Date),
                ("Assignee", DatabasePropertyType::People),
                ("Done", DatabasePropertyType::Checkbox),
                ("Website", DatabasePropertyType::Url),
                ("Email", DatabasePropertyType::Email),
                ("Created", DatabasePropertyType::CreatedTime),
            ],
            vec![simple_page("Row 1", vec![])],
        );
        insta::assert_snapshot!(normalize_table_rows(
            &compose_database_summary(&db).unwrap()
        ));
    }

    #[test]
    fn child_database_not_fetched() {
        insta::assert_snapshot!(render_one(&child_database("Key Highlights")));
    }

    #[test]
    fn child_database_embedded() {
        let db = simple_database("Metrics", vec![]);
        let block = Block::ChildDatabase(ChildDatabaseBlock {
            common: common(),
            title: "Metrics".to_string(),
            content: ChildDatabaseContent::Fetched(Box::new(db)),
        });
        insta::assert_snapshot!(render_one(&block));
    }
}

mod pages {
    use super::*;

    #[test]
    fn minimal_page() {
        let page = simple_page("My Page", vec![]);
        insta::assert_snapshot!(compose_page_markdown(&page, &RenderContext::default()).unwrap());
    }

    #[test]
    fn page_with_content() {
        let page = simple_page(
            "Development Guide",
            vec![
                heading1("Getting Started"),
                paragraph("Welcome to the development guide."),
                heading2("Installation"),
                code("cargo install notion2prompt", "bash"),
                heading2("Usage"),
                paragraph("Run the CLI with your Notion page URL."),
            ],
        );
        insta::assert_snapshot!(compose_page_markdown(&page, &RenderContext::default()).unwrap());
    }

    #[test]
    fn page_with_props() {
        let page = page_with_properties(
            "Feature Spec",
            vec![paragraph("Description of the feature.")],
            vec![
                (
                    "Status",
                    PropertyTypeValue::Select {
                        select: Some(SelectOption {
                            id: "1".to_string(),
                            name: "In Progress".to_string(),
                            color: Color::Blue,
                        }),
                    },
                ),
                (
                    "Priority",
                    PropertyTypeValue::Select {
                        select: Some(SelectOption {
                            id: "2".to_string(),
                            name: "High".to_string(),
                            color: Color::Red,
                        }),
                    },
                ),
                ("Done", PropertyTypeValue::Checkbox { checkbox: false }),
            ],
        );
        insta::assert_snapshot!(normalize_property_lines(
            &compose_page_markdown(&page, &RenderContext::default()).unwrap()
        ));
    }

    #[test]
    fn full_page() {
        let page = page_with_properties(
            "Weekly Standup Notes",
            vec![
                heading1("Summary"),
                paragraph("This week's key accomplishments."),
                bulleted("Shipped feature X"),
                bulleted("Fixed critical bug Y"),
                heading2("Action Items"),
                todo("Follow up with design team", false),
                todo("Deploy to production", true),
                divider(),
                heading2("Resources"),
                bookmark("https://docs.example.com"),
                code("SELECT * FROM users;", "sql"),
            ],
            vec![(
                "Date",
                PropertyTypeValue::Date {
                    date: Some(DateValue {
                        start: chrono::NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
                        end: None,
                        time_zone: None,
                    }),
                },
            )],
        );
        insta::assert_snapshot!(compose_page_markdown(&page, &RenderContext::default()).unwrap());
    }

    #[test]
    fn page_with_varied_blocks() {
        let page = simple_page(
            "Block Showcase",
            vec![
                heading1("Text Blocks"),
                paragraph("A simple paragraph."),
                quote("A thoughtful quote."),
                callout("Take note!", "📝"),
                heading2("Lists"),
                bulleted("Bullet A"),
                bulleted("Bullet B"),
                numbered("Step 1"),
                numbered("Step 2"),
                heading2("Media"),
                image_external("https://example.com/img.png"),
                divider(),
                heading2("Code"),
                code("console.log('hello')", "javascript"),
                equation("\\int_0^1 x^2 dx = \\frac{1}{3}"),
            ],
        );
        insta::assert_snapshot!(compose_page_markdown(&page, &RenderContext::default()).unwrap());
    }
}

mod integration {
    use super::*;

    #[test]
    fn fixture_jetbrains_blocks() {
        let blocks_json = include_str!("fixtures/api_responses/blocks_flow_ai_jetbrains.json");
        let blocks = parse_blocks_pagination(ApiResponse {
            data: blocks_json.to_string(),
            status: StatusCode::OK,
            url: "https://api.notion.com/v1/blocks/test/children".to_string(),
        })
        .unwrap();

        let output = render_blocks(&blocks.results, &RenderContext::default()).unwrap();
        insta::assert_snapshot!("jetbrains_blocks", output);
    }

    #[test]
    fn fixture_aie_agents_blocks() {
        let blocks_json = include_str!("fixtures/api_responses/blocks_aie_agents_nyc.json");
        let blocks = parse_blocks_pagination(ApiResponse {
            data: blocks_json.to_string(),
            status: StatusCode::OK,
            url: "https://api.notion.com/v1/blocks/test/children".to_string(),
        })
        .unwrap();

        let output = render_blocks(&blocks.results, &RenderContext::default()).unwrap();
        insta::assert_snapshot!("aie_agents_blocks", output);
    }

    #[test]
    fn fixture_amundi_blocks() {
        let blocks_json = include_str!("fixtures/api_responses/blocks_flow_ai_amundi.json");
        let blocks = parse_blocks_pagination(ApiResponse {
            data: blocks_json.to_string(),
            status: StatusCode::OK,
            url: "https://api.notion.com/v1/blocks/test/children".to_string(),
        })
        .unwrap();

        let output = render_blocks(&blocks.results, &RenderContext::default()).unwrap();
        insta::assert_snapshot!("amundi_blocks", output);
    }

    #[test]
    fn fixture_key_highlights_database() {
        let db_json = include_str!("fixtures/api_responses/database_key_highlights.json");
        let db = parse_database_response(ApiResponse {
            data: db_json.to_string(),
            status: StatusCode::OK,
            url: "https://api.notion.com/v1/databases/test".to_string(),
        })
        .unwrap();

        let output = compose_database_summary(&db).unwrap();
        insta::assert_snapshot!("key_highlights_db_summary", normalize_table_rows(&output));
    }
}
