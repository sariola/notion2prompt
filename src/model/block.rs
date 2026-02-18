use super::blocks::*;
use super::common::BlockCommon;
use crate::types::{BlockId, PageId, RichTextItem};
use serde::{Deserialize, Serialize};

/// Macro to reduce boilerplate in Block enum methods
macro_rules! match_all_blocks {
    ($self:expr, $pattern:pat => $result:expr) => {
        match $self {
            Block::Paragraph($pattern) => $result,
            Block::Heading1($pattern) => $result,
            Block::Heading2($pattern) => $result,
            Block::Heading3($pattern) => $result,
            Block::BulletedListItem($pattern) => $result,
            Block::NumberedListItem($pattern) => $result,
            Block::ToDo($pattern) => $result,
            Block::Toggle($pattern) => $result,
            Block::Quote($pattern) => $result,
            Block::Callout($pattern) => $result,
            Block::Code($pattern) => $result,
            Block::Equation($pattern) => $result,
            Block::Divider($pattern) => $result,
            Block::Breadcrumb($pattern) => $result,
            Block::TableOfContents($pattern) => $result,
            Block::Image($pattern) => $result,
            Block::Video($pattern) => $result,
            Block::File($pattern) => $result,
            Block::Pdf($pattern) => $result,
            Block::Bookmark($pattern) => $result,
            Block::Embed($pattern) => $result,
            Block::ChildPage($pattern) => $result,
            Block::ChildDatabase($pattern) => $result,
            Block::LinkToPage($pattern) => $result,
            Block::Table($pattern) => $result,
            Block::TableRow($pattern) => $result,
            Block::ColumnList($pattern) => $result,
            Block::Column($pattern) => $result,
            Block::Synced($pattern) => $result,
            Block::Template($pattern) => $result,
            Block::LinkPreview($pattern) => $result,
            Block::Unsupported($pattern) => $result,
        }
    };
}

/// Block represents all possible Notion block types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Paragraph(ParagraphBlock),
    Heading1(Heading1Block),
    Heading2(Heading2Block),
    Heading3(Heading3Block),
    BulletedListItem(BulletedListItemBlock),
    NumberedListItem(NumberedListItemBlock),
    ToDo(ToDoBlock),
    Toggle(ToggleBlock),
    Quote(QuoteBlock),
    Callout(CalloutBlock),
    Code(CodeBlock),
    Equation(EquationBlock),
    Divider(DividerBlock),
    Breadcrumb(BreadcrumbBlock),
    TableOfContents(TableOfContentsBlock),
    Image(ImageBlock),
    Video(VideoBlock),
    File(FileBlock),
    Pdf(PdfBlock),
    Bookmark(BookmarkBlock),
    Embed(EmbedBlock),
    ChildPage(ChildPageBlock),
    ChildDatabase(ChildDatabaseBlock),
    LinkToPage(LinkToPageBlock),
    Table(TableBlock),
    TableRow(TableRowBlock),
    ColumnList(ColumnListBlock),
    Column(ColumnBlock),
    Synced(SyncedBlock),
    Template(TemplateBlock),
    LinkPreview(LinkPreviewBlock),
    Unsupported(UnsupportedBlock),
}

impl Block {
    /// Get the block's ID
    pub fn id(&self) -> &BlockId {
        match_all_blocks!(self, b => &b.common.id)
    }

    /// Get the block's children
    pub fn children(&self) -> &Vec<Block> {
        match_all_blocks!(self, b => &b.common.children)
    }

    /// Get mutable reference to children
    pub fn children_mut(&mut self) -> &mut Vec<Block> {
        match_all_blocks!(self, b => &mut b.common.children)
    }

    /// Check if block has children
    pub fn has_children(&self) -> bool {
        self.common().has_children
    }

    /// Get common block data
    pub fn common(&self) -> &BlockCommon {
        match_all_blocks!(self, b => &b.common)
    }

    /// Get mutable common block data
    pub fn common_mut(&mut self) -> &mut BlockCommon {
        match_all_blocks!(self, b => &mut b.common)
    }

    /// Set children
    pub fn set_children(&mut self, children: Vec<Block>) {
        self.common_mut().children = children;
    }

    /// Get block type name
    pub fn block_type(&self) -> &'static str {
        match self {
            Block::Paragraph(_) => "paragraph",
            Block::Heading1(_) => "heading_1",
            Block::Heading2(_) => "heading_2",
            Block::Heading3(_) => "heading_3",
            Block::BulletedListItem(_) => "bulleted_list_item",
            Block::NumberedListItem(_) => "numbered_list_item",
            Block::ToDo(_) => "to_do",
            Block::Toggle(_) => "toggle",
            Block::Quote(_) => "quote",
            Block::Callout(_) => "callout",
            Block::Code(_) => "code",
            Block::Equation(_) => "equation",
            Block::Divider(_) => "divider",
            Block::Breadcrumb(_) => "breadcrumb",
            Block::TableOfContents(_) => "table_of_contents",
            Block::Image(_) => "image",
            Block::Video(_) => "video",
            Block::File(_) => "file",
            Block::Pdf(_) => "pdf",
            Block::Bookmark(_) => "bookmark",
            Block::Embed(_) => "embed",
            Block::ChildPage(_) => "child_page",
            Block::ChildDatabase(_) => "child_database",
            Block::LinkToPage(_) => "link_to_page",
            Block::Table(_) => "table",
            Block::TableRow(_) => "table_row",
            Block::ColumnList(_) => "column_list",
            Block::Column(_) => "column",
            Block::Synced(_) => "synced_block",
            Block::Template(_) => "template",
            Block::LinkPreview(_) => "link_preview",
            Block::Unsupported(_) => "unsupported",
        }
    }

    /// Accept a visitor
    pub fn accept<V: BlockVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Block::Paragraph(b) => visitor.visit_paragraph(&b.common.id, &b.content),
            Block::Heading1(b) => visitor.visit_heading1(&b.common.id, &b.content),
            Block::Heading2(b) => visitor.visit_heading2(&b.common.id, &b.content),
            Block::Heading3(b) => visitor.visit_heading3(&b.common.id, &b.content),
            Block::BulletedListItem(b) => {
                visitor.visit_bulleted_list_item(&b.common.id, &b.content)
            }
            Block::NumberedListItem(b) => {
                visitor.visit_numbered_list_item(&b.common.id, &b.content)
            }
            Block::ToDo(b) => visitor.visit_todo(&b.common.id, b),
            Block::Toggle(b) => visitor.visit_toggle(&b.common.id, &b.content),
            Block::Quote(b) => visitor.visit_quote(&b.common.id, &b.content),
            Block::Callout(b) => visitor.visit_callout(&b.common.id, b),
            Block::Code(b) => visitor.visit_code(&b.common.id, b),
            Block::Equation(b) => visitor.visit_equation(&b.common.id, &b.expression),
            Block::Divider(b) => visitor.visit_divider(&b.common.id),
            Block::Breadcrumb(b) => visitor.visit_breadcrumb(&b.common.id),
            Block::TableOfContents(b) => visitor.visit_table_of_contents(&b.common.id),
            Block::Image(b) => visitor.visit_image(&b.common.id, b),
            Block::Video(b) => visitor.visit_video(&b.common.id, b),
            Block::File(b) => visitor.visit_file(&b.common.id, b),
            Block::Pdf(b) => visitor.visit_pdf(&b.common.id, b),
            Block::Bookmark(b) => visitor.visit_bookmark(&b.common.id, b),
            Block::Embed(b) => visitor.visit_embed(&b.common.id, b),
            Block::ChildPage(b) => visitor.visit_child_page(&b.common.id, b),
            Block::ChildDatabase(b) => visitor.visit_child_database(&b.common.id, b),
            Block::LinkToPage(b) => visitor.visit_link_to_page(&b.common.id, &b.page_id),
            Block::Table(b) => visitor.visit_table(&b.common.id, b),
            Block::TableRow(b) => visitor.visit_table_row(&b.common.id, &b.cells),
            Block::ColumnList(b) => visitor.visit_column_list(&b.common.id),
            Block::Column(b) => visitor.visit_column(&b.common.id),
            Block::Synced(b) => visitor.visit_synced_block(&b.common.id, b),
            Block::Template(b) => visitor.visit_template(&b.common.id, b),
            Block::LinkPreview(b) => visitor.visit_link_preview(&b.common.id, &b.url),
            Block::Unsupported(b) => visitor.visit_unsupported(&b.common.id, &b.block_type),
        }
    }
}

/// Visitor trait for traversing block structures.
///
/// All methods have default implementations that return `Default::default()`,
/// so implementors only need to override the methods they care about.
pub trait BlockVisitor {
    type Output: Default;

    fn visit_paragraph(&mut self, _id: &BlockId, _content: &TextBlockContent) -> Self::Output {
        Default::default()
    }
    fn visit_heading1(&mut self, _id: &BlockId, _content: &TextBlockContent) -> Self::Output {
        Default::default()
    }
    fn visit_heading2(&mut self, _id: &BlockId, _content: &TextBlockContent) -> Self::Output {
        Default::default()
    }
    fn visit_heading3(&mut self, _id: &BlockId, _content: &TextBlockContent) -> Self::Output {
        Default::default()
    }
    fn visit_bulleted_list_item(
        &mut self,
        _id: &BlockId,
        _content: &TextBlockContent,
    ) -> Self::Output {
        Default::default()
    }
    fn visit_numbered_list_item(
        &mut self,
        _id: &BlockId,
        _content: &TextBlockContent,
    ) -> Self::Output {
        Default::default()
    }
    fn visit_todo(&mut self, _id: &BlockId, _todo: &ToDoBlock) -> Self::Output {
        Default::default()
    }
    fn visit_toggle(&mut self, _id: &BlockId, _content: &TextBlockContent) -> Self::Output {
        Default::default()
    }
    fn visit_quote(&mut self, _id: &BlockId, _content: &TextBlockContent) -> Self::Output {
        Default::default()
    }
    fn visit_callout(&mut self, _id: &BlockId, _callout: &CalloutBlock) -> Self::Output {
        Default::default()
    }
    fn visit_code(&mut self, _id: &BlockId, _code: &CodeBlock) -> Self::Output {
        Default::default()
    }
    fn visit_equation(&mut self, _id: &BlockId, _expression: &str) -> Self::Output {
        Default::default()
    }
    fn visit_divider(&mut self, _id: &BlockId) -> Self::Output {
        Default::default()
    }
    fn visit_breadcrumb(&mut self, _id: &BlockId) -> Self::Output {
        Default::default()
    }
    fn visit_table_of_contents(&mut self, _id: &BlockId) -> Self::Output {
        Default::default()
    }
    fn visit_image(&mut self, _id: &BlockId, _image: &ImageBlock) -> Self::Output {
        Default::default()
    }
    fn visit_video(&mut self, _id: &BlockId, _video: &VideoBlock) -> Self::Output {
        Default::default()
    }
    fn visit_file(&mut self, _id: &BlockId, _file: &FileBlock) -> Self::Output {
        Default::default()
    }
    fn visit_pdf(&mut self, _id: &BlockId, _pdf: &PdfBlock) -> Self::Output {
        Default::default()
    }
    fn visit_bookmark(&mut self, _id: &BlockId, _bookmark: &BookmarkBlock) -> Self::Output {
        Default::default()
    }
    fn visit_embed(&mut self, _id: &BlockId, _embed: &EmbedBlock) -> Self::Output {
        Default::default()
    }
    fn visit_child_page(&mut self, _id: &BlockId, _page: &ChildPageBlock) -> Self::Output {
        Default::default()
    }
    fn visit_child_database(
        &mut self,
        _id: &BlockId,
        _database: &ChildDatabaseBlock,
    ) -> Self::Output {
        Default::default()
    }
    fn visit_link_to_page(&mut self, _id: &BlockId, _page_id: &PageId) -> Self::Output {
        Default::default()
    }
    fn visit_table(&mut self, _id: &BlockId, _table: &TableBlock) -> Self::Output {
        Default::default()
    }
    fn visit_table_row(&mut self, _id: &BlockId, _cells: &[Vec<RichTextItem>]) -> Self::Output {
        Default::default()
    }
    fn visit_column_list(&mut self, _id: &BlockId) -> Self::Output {
        Default::default()
    }
    fn visit_column(&mut self, _id: &BlockId) -> Self::Output {
        Default::default()
    }
    fn visit_synced_block(&mut self, _id: &BlockId, _synced: &SyncedBlock) -> Self::Output {
        Default::default()
    }
    fn visit_template(&mut self, _id: &BlockId, _template: &TemplateBlock) -> Self::Output {
        Default::default()
    }
    fn visit_link_preview(&mut self, _id: &BlockId, _url: &str) -> Self::Output {
        Default::default()
    }
    fn visit_unsupported(&mut self, _id: &BlockId, _block_type: &str) -> Self::Output {
        Default::default()
    }
}
