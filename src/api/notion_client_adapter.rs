// src/api/notion_client_adapter.rs
//! Adapter layer for converting notion-client types to our domain model.
//!
//! This module preserves our architectural invariants while leveraging
//! the notion-client library's validated types and serialization.

use crate::error::{AppError, NotionClientError};
use crate::model::blocks::*;
use crate::model::common::BlockCommon;
use crate::model::{Block, Database, DatabaseTitle, NumberFormat, Page, PageTitle, Parent};
use crate::types::{BlockId, Color, DatabaseId, PageId, PropertyName, RichTextItem};
use std::collections::HashMap;

/// Convert notion-client Page to our domain Page
pub fn convert_page(notion_page: notion_client::objects::page::Page) -> Result<Page, AppError> {
    let id = PageId::parse(&notion_page.id)?;

    // Convert title from properties
    let title = extract_page_title(notion_page.properties.clone())?;

    // Convert parent
    let parent = Some(convert_parent(notion_page.parent)?);

    Ok(Page {
        id,
        title,
        url: notion_page.url,
        blocks: Vec::new(), // Will be populated during fetch stage
        properties: convert_page_properties(notion_page.properties)?,
        parent,
        archived: notion_page.archived,
    })
}

/// Convert notion-client Database to our domain Database
pub fn convert_database(
    notion_db: notion_client::objects::database::Database,
) -> Result<Database, AppError> {
    let id =
        DatabaseId::parse(
            &notion_db
                .id
                .ok_or_else(|| NotionClientError::ConversionError {
                    message: "Database missing required ID field".to_string(),
                })?,
        )?;

    let title = DatabaseTitle::new(convert_rich_text_array(notion_db.title)?);

    let parent = Some(convert_parent(notion_db.parent)?);

    Ok(Database {
        id,
        title,
        url: notion_db.url,
        pages: Vec::new(), // Will be populated during fetch stage
        properties: convert_database_properties(notion_db.properties)?,
        parent,
        archived: notion_db.archived,
    })
}

/// Convert notion-client Block to our domain Block
pub fn convert_block(
    notion_block: notion_client::objects::block::Block,
) -> Result<Block, AppError> {
    let common = convert_block_common(&notion_block)?;

    use notion_client::objects::block::BlockType;

    match notion_block.block_type {
        BlockType::Paragraph { paragraph } => Ok(Block::Paragraph(ParagraphBlock {
            common,
            content: convert_text_block_content(paragraph.rich_text, paragraph.color)?,
        })),

        BlockType::Heading1 { heading_1 } => Ok(Block::Heading1(Heading1Block {
            common,
            content: convert_text_block_content(heading_1.rich_text, heading_1.color)?,
        })),

        BlockType::Heading2 { heading_2 } => Ok(Block::Heading2(Heading2Block {
            common,
            content: convert_text_block_content(heading_2.rich_text, heading_2.color)?,
        })),

        BlockType::Heading3 { heading_3 } => Ok(Block::Heading3(Heading3Block {
            common,
            content: convert_text_block_content(heading_3.rich_text, heading_3.color)?,
        })),

        BlockType::ChildDatabase { child_database } => {
            Ok(Block::ChildDatabase(ChildDatabaseBlock {
                common,
                title: child_database.title,
                content: ChildDatabaseContent::NotFetched,
            }))
        }

        BlockType::BulletedListItem { bulleted_list_item } => {
            Ok(Block::BulletedListItem(BulletedListItemBlock {
                common,
                content: convert_text_block_content(
                    bulleted_list_item.rich_text,
                    Some(bulleted_list_item.color),
                )?,
            }))
        }

        BlockType::NumberedListItem { numbered_list_item } => {
            Ok(Block::NumberedListItem(NumberedListItemBlock {
                common,
                content: convert_text_block_content(
                    numbered_list_item.rich_text,
                    Some(numbered_list_item.color),
                )?,
            }))
        }

        BlockType::ToDo { to_do } => Ok(Block::ToDo(ToDoBlock {
            common,
            content: convert_text_block_content(to_do.rich_text, to_do.color)?,
            checked: to_do.checked.unwrap_or(false),
        })),

        BlockType::Toggle { toggle } => Ok(Block::Toggle(ToggleBlock {
            common,
            content: convert_text_block_content(toggle.rich_text, Some(toggle.color))?,
        })),

        BlockType::Quote { quote } => Ok(Block::Quote(QuoteBlock {
            common,
            content: convert_text_block_content(quote.rich_text, Some(quote.color))?,
        })),

        BlockType::Code { code } => Ok(Block::Code(CodeBlock {
            common,
            content: convert_text_block_content(code.rich_text, Default::default())?,
            language: format!("{:?}", code.language),
            caption: convert_rich_text_array(code.caption)?,
        })),

        BlockType::Divider { .. } => Ok(Block::Divider(DividerBlock { common })),

        BlockType::Breadcrumb { .. } => Ok(Block::Breadcrumb(BreadcrumbBlock { common })),

        BlockType::TableOfContents { .. } => {
            Ok(Block::TableOfContents(TableOfContentsBlock { common }))
        }

        BlockType::Bookmark { bookmark } => Ok(Block::Bookmark(BookmarkBlock {
            common,
            url: bookmark.url,
            caption: convert_rich_text_array(bookmark.caption)?,
        })),

        BlockType::Embed { embed } => Ok(Block::Embed(EmbedBlock {
            common,
            url: embed.url,
        })),

        BlockType::Equation { equation } => Ok(Block::Equation(EquationBlock {
            common,
            expression: equation.expression,
        })),

        BlockType::ChildPage { child_page } => Ok(Block::ChildPage(ChildPageBlock {
            common,
            title: child_page.title,
        })),

        BlockType::Image { image } => Ok(Block::Image(ImageBlock {
            common,
            image: convert_file_object(image.file_type)?,
            caption: Vec::new(), // ImageValue doesn't have caption in this version
        })),

        BlockType::Video { video } => Ok(Block::Video(VideoBlock {
            common,
            video: convert_file_object(video.file_type)?,
            caption: Vec::new(), // VideoValue doesn't have caption in this version
        })),

        BlockType::File { file } => Ok(Block::File(FileBlock {
            common,
            file: convert_file_object(file.file_type)?,
            caption: convert_rich_text_array(file.caption)?,
        })),

        BlockType::Pdf { pdf } => Ok(Block::Pdf(PdfBlock {
            common,
            pdf: convert_file_object(pdf.file_type)?,
            caption: convert_rich_text_array(pdf.caption)?,
        })),

        BlockType::Callout { callout } => Ok(Block::Callout(CalloutBlock {
            common,
            icon: callout.icon.map(convert_icon).transpose()?,
            content: convert_text_block_content(callout.rich_text, Some(callout.color))?,
        })),

        BlockType::Table { table } => Ok(Block::Table(TableBlock {
            common,
            table_width: table.table_width as usize,
            has_column_header: table.has_column_header,
            has_row_header: table.has_row_header,
        })),

        BlockType::TableRow { table_row } => Ok(Block::TableRow(TableRowBlock {
            common,
            cells: table_row
                .cells
                .into_iter()
                .map(convert_rich_text_array)
                .collect::<Result<Vec<_>, _>>()?,
        })),

        BlockType::ColumnList { .. } => Ok(Block::ColumnList(ColumnListBlock { common })),

        BlockType::Column { .. } => Ok(Block::Column(ColumnBlock { common })),

        BlockType::LinkToPage { link_to_page } => {
            let page_id = match link_to_page {
                notion_client::objects::parent::Parent::PageId { page_id } => {
                    PageId::parse(&page_id)?
                }
                _ => {
                    return Err(NotionClientError::ConversionError {
                        message: "LinkToPage block must have PageId parent".to_string(),
                    }
                    .into())
                }
            };
            Ok(Block::LinkToPage(LinkToPageBlock { common, page_id }))
        }

        BlockType::SyncedBlock { synced_block } => {
            let synced_from = synced_block.synced_from.map(|sf| match sf {
                notion_client::objects::block::SyncedFrom::BlockId { block_id } => SyncedFrom {
                    block_id: BlockId::parse(&block_id).unwrap_or_else(|_| {
                        log::warn!(
                            "Invalid synced_from block ID '{}', using fallback UUID",
                            block_id
                        );
                        BlockId::new_v4()
                    }),
                },
            });
            Ok(Block::Synced(SyncedBlock {
                common,
                synced_from,
            }))
        }

        BlockType::Template { template } => Ok(Block::Template(TemplateBlock {
            common,
            content: convert_text_block_content(template.rich_text, None)?,
        })),

        BlockType::LinkPreview { link_preview } => Ok(Block::LinkPreview(LinkPreviewBlock {
            common,
            url: link_preview.url,
        })),

        // For truly unsupported types, map to our Unsupported variant
        _ => Ok(Block::Unsupported(UnsupportedBlock {
            common,
            block_type: format!("{:?}", notion_block.block_type),
        })),
    }
}

/// Convert notion-client Parent to our domain Parent
fn convert_parent(
    notion_parent: notion_client::objects::parent::Parent,
) -> Result<Parent, AppError> {
    use notion_client::objects::parent::Parent as NcParent;

    match notion_parent {
        NcParent::PageId { page_id } => Ok(Parent::Page {
            page_id: PageId::parse(&page_id)?,
        }),
        NcParent::DatabaseId { database_id } => Ok(Parent::Database {
            database_id: DatabaseId::parse(&database_id)?,
        }),
        NcParent::BlockId { block_id } => Ok(Parent::Block {
            block_id: BlockId::parse(&block_id)?,
        }),
        NcParent::Workspace { .. } => Ok(Parent::Workspace),
        _ => Err(NotionClientError::ConversionError {
            message: "Unsupported parent type".to_string(),
        }
        .into()),
    }
}

/// Convert block common fields
fn convert_block_common(
    notion_block: &notion_client::objects::block::Block,
) -> Result<BlockCommon, AppError> {
    let id = BlockId::parse(&notion_block.id.clone().ok_or_else(|| {
        NotionClientError::ConversionError {
            message: "Block missing required ID field".to_string(),
        }
    })?)?;

    Ok(BlockCommon {
        id,
        children: Vec::new(), // Will be populated during fetch stage
        has_children: notion_block.has_children.unwrap_or(false),
        archived: notion_block.archived.unwrap_or(false),
    })
}

/// Convert text block content (rich text + color)
fn convert_text_block_content(
    rich_text: Vec<notion_client::objects::rich_text::RichText>,
    color: Option<notion_client::objects::block::TextColor>,
) -> Result<TextBlockContent, AppError> {
    Ok(TextBlockContent {
        rich_text: convert_rich_text_array(rich_text)?,
        color: convert_block_color(
            color.unwrap_or(notion_client::objects::block::TextColor::Default),
        ),
    })
}

/// Convert array of rich text items
fn convert_rich_text_array(
    rich_texts: Vec<notion_client::objects::rich_text::RichText>,
) -> Result<Vec<RichTextItem>, AppError> {
    rich_texts.into_iter().map(convert_rich_text).collect()
}

/// Convert single rich text item
fn convert_rich_text(
    rich_text: notion_client::objects::rich_text::RichText,
) -> Result<RichTextItem, AppError> {
    use notion_client::objects::rich_text::RichText as NcRichText;

    match rich_text {
        NcRichText::Text {
            text,
            annotations,
            plain_text,
            href,
        } => Ok(RichTextItem {
            text_type: crate::types::RichTextType::Text {
                content: text.content,
                link: text.link.map(|link| crate::types::Link { url: link.url }),
            },
            annotations: convert_annotations(annotations.unwrap_or_default()),
            plain_text: plain_text.unwrap_or_default(),
            href,
        }),

        NcRichText::Mention {
            mention,
            annotations,
            plain_text,
            href,
        } => {
            // For link mentions that fallback, use the href if available
            let mention_data = match convert_mention(mention) {
                Ok(mention_data) => mention_data,
                Err(_) => {
                    // If mention conversion fails, use href as fallback if available
                    let url = href
                        .clone()
                        .unwrap_or_else(|| "https://notion.so".to_string());
                    crate::types::MentionData {
                        mention_type: crate::types::MentionType::LinkMention { url },
                    }
                }
            };

            Ok(RichTextItem {
                text_type: crate::types::RichTextType::Mention(mention_data),
                annotations: convert_annotations(annotations),
                plain_text,
                href,
            })
        }

        NcRichText::Equation {
            equation,
            annotations,
            plain_text,
            href,
        } => Ok(RichTextItem {
            text_type: crate::types::RichTextType::Equation(crate::types::EquationData {
                expression: equation.expression,
            }),
            annotations: convert_annotations(annotations),
            plain_text,
            href,
        }),

        _ => {
            // For unsupported rich text types, create a plain text fallback
            Ok(RichTextItem {
                text_type: crate::types::RichTextType::Text {
                    content: "Unsupported rich text type".to_string(),
                    link: None,
                },
                annotations: Default::default(),
                plain_text: "Unsupported rich text type".to_string(),
                href: None,
            })
        }
    }
}

/// Convert annotations
fn convert_annotations(
    annotations: notion_client::objects::rich_text::Annotations,
) -> crate::types::Annotations {
    crate::types::Annotations {
        bold: annotations.bold,
        italic: annotations.italic,
        strikethrough: annotations.strikethrough,
        underline: annotations.underline,
        code: annotations.code,
        color: convert_rich_text_color(annotations.color),
    }
}

/// Convert mention with graceful fallback for unsupported types
fn convert_mention(
    mention: notion_client::objects::rich_text::Mention,
) -> Result<crate::types::MentionData, AppError> {
    use notion_client::objects::rich_text::Mention as NcMention;

    match mention {
        NcMention::User { user } => Ok(crate::types::MentionData {
            mention_type: crate::types::MentionType::User {
                user: convert_user(user)?,
            },
        }),
        NcMention::Page { page } => Ok(crate::types::MentionData {
            mention_type: crate::types::MentionType::Page {
                page: crate::types::PageReference {
                    id: crate::types::NotionId::parse(&page.id)?,
                },
            },
        }),
        NcMention::Database { database } => Ok(crate::types::MentionData {
            mention_type: crate::types::MentionType::Database {
                database: crate::types::DatabaseReference {
                    id: crate::types::NotionId::parse(&database.id)?,
                },
            },
        }),
        NcMention::Date { date } => {
            // Convert date mention to our DateValue type
            let start_date = date.start.naive_utc().date();
            let end_date = date.end.map(|end| end.naive_utc().date());

            Ok(crate::types::MentionData {
                mention_type: crate::types::MentionType::Date {
                    date: crate::types::DateValue {
                        start: start_date,
                        end: end_date,
                        time_zone: date.time_zone,
                    },
                },
            })
        }
        NcMention::LinkPreview { link_preview } => Ok(crate::types::MentionData {
            mention_type: crate::types::MentionType::LinkPreview {
                link_preview: crate::types::LinkPreviewReference {
                    url: link_preview.url,
                },
            },
        }),
        // Graceful fallback for any other mention types that might exist
        _ => {
            log::debug!("Unsupported mention type encountered, will use href from parent context");
            // Return an error so the parent context can use href
            Err(NotionClientError::ConversionError {
                message: "Unsupported mention type".to_string(),
            }
            .into())
        }
    }
}

/// Convert user
fn convert_user(
    user: notion_client::objects::user::User,
) -> Result<crate::types::PartialUser, AppError> {
    Ok(crate::types::PartialUser {
        id: user.id,
        name: user.name,
        avatar_url: user.avator_url, // Note: typo in notion-client
    })
}

/// Convert file object from notion-client to our domain type
fn convert_file_object(file: notion_client::objects::file::File) -> Result<FileObject, AppError> {
    use notion_client::objects::file::File as NcFile;

    match file {
        NcFile::External { external } => Ok(FileObject::External {
            external: ExternalFile { url: external.url },
        }),
        NcFile::File { file } => Ok(FileObject::File {
            file: NotionFile {
                url: file.url,
                expiry_time: Some(file.expiry_time),
            },
        }),
        // No unreachable pattern needed - File enum only has External and File variants
    }
}

/// Convert icon from notion-client to our domain type  
fn convert_icon(icon: notion_client::objects::block::Icon) -> Result<Icon, AppError> {
    use notion_client::objects::block::Icon as NcIcon;

    match icon {
        NcIcon::Emoji(emoji) => {
            // Extract emoji string from the Emoji enum
            match emoji {
                notion_client::objects::emoji::Emoji::Emoji { emoji } => Ok(Icon::Emoji { emoji }),
            }
        }
        NcIcon::File(file) => {
            // Convert notion-client File to our FileObject, then extract for Icon
            let file_obj = convert_file_object(file)?;
            match file_obj {
                FileObject::File { file } => Ok(Icon::File { file }),
                FileObject::External { external } => Ok(Icon::External { external }),
            }
        }
    }
}

/// Convert block-level color (for paragraphs, headings, etc.)
/// Generates a color conversion function from a Notion color enum to our Color type.
/// The `with_backgrounds` variant also maps *Background variants to their base colors.
macro_rules! impl_color_conversion {
    ($fn_name:ident, $source:ty) => {
        fn $fn_name(color: $source) -> Color {
            match color {
                <$source>::Default => Color::Default,
                <$source>::Gray => Color::Gray,
                <$source>::Brown => Color::Brown,
                <$source>::Orange => Color::Orange,
                <$source>::Yellow => Color::Yellow,
                <$source>::Green => Color::Green,
                <$source>::Blue => Color::Blue,
                <$source>::Purple => Color::Purple,
                <$source>::Pink => Color::Pink,
                <$source>::Red => Color::Red,
            }
        }
    };
    ($fn_name:ident, $source:ty, with_backgrounds) => {
        fn $fn_name(color: $source) -> Color {
            match color {
                <$source>::Default => Color::Default,
                <$source>::Gray | <$source>::GrayBackground => Color::Gray,
                <$source>::Brown | <$source>::BrownBackground => Color::Brown,
                <$source>::Orange | <$source>::OrangeBackground => Color::Orange,
                <$source>::Yellow | <$source>::YellowBackground => Color::Yellow,
                <$source>::Green | <$source>::GreenBackground => Color::Green,
                <$source>::Blue | <$source>::BlueBackground => Color::Blue,
                <$source>::Purple | <$source>::PurpleBackground => Color::Purple,
                <$source>::Pink | <$source>::PinkBackground => Color::Pink,
                <$source>::Red | <$source>::RedBackground => Color::Red,
            }
        }
    };
}

impl_color_conversion!(
    convert_block_color,
    notion_client::objects::block::TextColor,
    with_backgrounds
);
impl_color_conversion!(
    convert_rich_text_color,
    notion_client::objects::rich_text::TextColor,
    with_backgrounds
);
impl_color_conversion!(
    convert_database_property_color,
    notion_client::objects::database::Color
);
impl_color_conversion!(
    convert_page_property_color,
    notion_client::objects::page::Color
);

/// Convert PartialUser to User
fn convert_partial_user_to_user(partial_user: crate::types::PartialUser) -> crate::types::User {
    crate::types::User {
        id: partial_user.id,
        name: partial_user.name,
        avatar_url: partial_user.avatar_url,
        email: None, // PartialUser doesn't have email
    }
}

/// Extract page title from properties
fn extract_page_title(
    properties: HashMap<String, notion_client::objects::page::PageProperty>,
) -> Result<PageTitle, AppError> {
    // Look for title property
    for (_, property) in properties {
        if let notion_client::objects::page::PageProperty::Title { title, .. } = property {
            let plain_text = title
                .into_iter()
                .map(convert_rich_text)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|rt| rt.plain_text)
                .collect::<Vec<_>>()
                .join("");
            return Ok(PageTitle::new(plain_text));
        }
    }

    // Fallback to empty title
    Ok(PageTitle::new("Untitled"))
}

/// Convert page properties with graceful fallbacks
fn convert_page_properties(
    properties: HashMap<String, notion_client::objects::page::PageProperty>,
) -> Result<HashMap<PropertyName, crate::model::PropertyValue>, AppError> {
    let mut converted = HashMap::new();

    for (name, property) in properties {
        match convert_page_property(&name, property) {
            Ok(prop_value) => {
                converted.insert(PropertyName::new(name), prop_value);
            }
            Err(e) => {
                // Graceful fallback: log the error but continue processing
                log::warn!(
                    "Failed to convert page property '{}': {}. Skipping.",
                    name,
                    e
                );
                continue;
            }
        }
    }

    Ok(converted)
}

/// Convert database properties with graceful fallbacks
fn convert_database_properties(
    properties: HashMap<String, notion_client::objects::database::DatabaseProperty>,
) -> Result<HashMap<PropertyName, crate::model::DatabaseProperty>, AppError> {
    let mut converted = HashMap::new();

    for (name, property) in properties {
        match convert_database_property(&name, property) {
            Ok(db_prop) => {
                converted.insert(PropertyName::new(name), db_prop);
            }
            Err(e) => {
                // Graceful fallback: log the error but continue processing
                log::warn!(
                    "Failed to convert database property '{}': {}. Skipping.",
                    name,
                    e
                );
                continue;
            }
        }
    }

    Ok(converted)
}

// --- Shared property conversion helpers ---

/// Converts a Notion `DateOrDateTime` to a `NaiveDate`.
fn resolve_date(dod: notion_client::objects::page::DateOrDateTime) -> chrono::NaiveDate {
    match dod {
        notion_client::objects::page::DateOrDateTime::Date(d) => d,
        notion_client::objects::page::DateOrDateTime::DateTime(dt) => dt.date_naive(),
    }
}

/// Converts an optional Notion `DateOrDateTime` start + optional end into our `DateValue`.
fn convert_notion_date(
    d: notion_client::objects::page::DatePropertyValue,
) -> crate::types::DateValue {
    crate::types::DateValue {
        start: d
            .start
            .map(resolve_date)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
        end: d.end.map(resolve_date),
        time_zone: d.time_zone,
    }
}

/// Converts a Notion select option to our domain `SelectOption`.
fn convert_select_option(
    s: notion_client::objects::page::SelectPropertyValue,
) -> crate::types::SelectOption {
    crate::types::SelectOption {
        id: s.id.unwrap_or_default(),
        name: s.name.unwrap_or_default(),
        color: convert_page_property_color(
            s.color
                .unwrap_or(notion_client::objects::page::Color::Default),
        ),
    }
}

/// Converts a Notion file reference to our domain `File`.
fn convert_file_ref(f: notion_client::objects::page::FilePropertyValue) -> crate::types::File {
    crate::types::File {
        name: f.name,
        url: match &f.file {
            notion_client::objects::file::File::External { external } => external.url.clone(),
            notion_client::objects::file::File::File { file } => file.url.clone(),
        },
        expiry_time: match &f.file {
            notion_client::objects::file::File::File { file } => Some(file.expiry_time),
            _ => None,
        },
    }
}

/// Builds a `PropertyValue` with the given name and value.
fn make_property(
    name: &str,
    value: crate::model::PropertyTypeValue,
) -> crate::model::PropertyValue {
    crate::model::PropertyValue {
        id: PropertyName::new(name),
        type_specific_value: value,
    }
}

// --- Per-type converters for complex property types ---

/// Converts a Notion Formula property value to our domain `FormulaResult`.
fn convert_formula_value(
    formula: Option<notion_client::objects::page::FormulaPropertyValue>,
) -> crate::types::FormulaResult {
    use notion_client::objects::page::FormulaPropertyValue;

    formula
        .map(|f| match f {
            FormulaPropertyValue::String { string } => {
                crate::types::FormulaResult::String(string.unwrap_or_default())
            }
            FormulaPropertyValue::Number { number } => crate::types::FormulaResult::Number(
                number.map(|n| n.as_f64().unwrap_or(0.0)).unwrap_or(0.0),
            ),
            FormulaPropertyValue::Boolean { boolean } => {
                crate::types::FormulaResult::Boolean(boolean)
            }
            FormulaPropertyValue::Date { date } => match date {
                Some(d) => crate::types::FormulaResult::Date(convert_notion_date(d)),
                None => crate::types::FormulaResult::String(String::new()),
            },
        })
        .unwrap_or(crate::types::FormulaResult::String(String::new()))
}

/// Converts a Notion Rollup property value to our domain `RollupResult`.
fn convert_rollup_value(
    rollup: Option<notion_client::objects::page::RollupPropertyValue>,
) -> crate::types::RollupResult {
    use notion_client::objects::page::RollupPropertyValue;

    rollup
        .map(|r| match r {
            RollupPropertyValue::Number { number, .. } => crate::types::RollupResult::Number {
                number: number.map(|n| n.as_f64().unwrap_or(0.0)),
            },
            RollupPropertyValue::Date { date, .. } => crate::types::RollupResult::Date {
                date: date.map(|d| crate::types::DateValue {
                    start: d.naive_utc().date(),
                    end: None,
                    time_zone: None,
                }),
            },
            RollupPropertyValue::Array { array, .. } => crate::types::RollupResult::Array {
                array: array.into_iter().map(convert_rollup_array_item).collect(),
            },
            RollupPropertyValue::Incomplete { .. } => crate::types::RollupResult::String {
                string: Some("incomplete".to_string()),
            },
            RollupPropertyValue::Unsupported { .. } => crate::types::RollupResult::String {
                string: Some("unsupported".to_string()),
            },
        })
        .unwrap_or(crate::types::RollupResult::String { string: None })
}

/// Converts a single rollup array item (`PageProperty`) into a `RollupArrayItem`.
///
/// Rollup arrays contain full `PageProperty` values. We extract the meaningful
/// content from each variant, using `Text(String)` as a catch-all for types that
/// don't have a dedicated `RollupArrayItem` variant.
fn convert_rollup_array_item(
    item: notion_client::objects::page::PageProperty,
) -> crate::types::RollupArrayItem {
    use crate::types::RollupArrayItem;
    use notion_client::objects::page::PageProperty;

    match item {
        PageProperty::Title { title, .. }
        | PageProperty::RichText {
            rich_text: title, ..
        } => match convert_rich_text_array(title) {
            Ok(items) => RollupArrayItem::Title(items),
            Err(_) => RollupArrayItem::Text(String::new()),
        },
        PageProperty::Number { number, .. } => {
            RollupArrayItem::Number(number.and_then(|n| n.as_f64()).unwrap_or(0.0))
        }
        PageProperty::Date { date: Some(d), .. } => RollupArrayItem::Date(convert_notion_date(d)),
        PageProperty::Date { date: None, .. } => RollupArrayItem::Text(String::new()),
        PageProperty::Select { select, .. } => {
            RollupArrayItem::Text(select.and_then(|s| s.name).unwrap_or_default())
        }
        PageProperty::MultiSelect { multi_select, .. } => {
            let names: Vec<String> = multi_select.into_iter().filter_map(|s| s.name).collect();
            RollupArrayItem::Text(names.join(", "))
        }
        PageProperty::Status { status, .. } => {
            RollupArrayItem::Text(status.and_then(|s| s.name).unwrap_or_default())
        }
        PageProperty::Checkbox { checkbox, .. } => {
            RollupArrayItem::Text(if checkbox { "Yes" } else { "No" }.to_string())
        }
        PageProperty::Url { url, .. } => RollupArrayItem::Text(url.unwrap_or_default()),
        PageProperty::Email { email, .. } => RollupArrayItem::Text(email.unwrap_or_default()),
        PageProperty::PhoneNumber { phone_number, .. } => {
            RollupArrayItem::Text(phone_number.unwrap_or_default())
        }
        PageProperty::People { people, .. } => {
            let names: Vec<String> = people.into_iter().filter_map(|u| u.name).collect();
            RollupArrayItem::Text(names.join(", "))
        }
        PageProperty::Formula { formula, .. } => {
            let result = convert_formula_value(formula);
            RollupArrayItem::Text(match result {
                crate::types::FormulaResult::String(s) => s,
                crate::types::FormulaResult::Number(n) => n.to_string(),
                crate::types::FormulaResult::Boolean(b) => b.to_string(),
                crate::types::FormulaResult::Date(d) => d.start.to_string(),
            })
        }
        PageProperty::CreatedTime { created_time, .. } => {
            RollupArrayItem::Text(created_time.format("%Y-%m-%d %H:%M").to_string())
        }
        PageProperty::LastEditedTime {
            last_edited_time, ..
        } => RollupArrayItem::Text(
            last_edited_time
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default(),
        ),
        PageProperty::CreatedBy { created_by, .. } => {
            RollupArrayItem::Text(created_by.name.unwrap_or_default())
        }
        PageProperty::LastEditedBy { last_edited_by, .. } => {
            RollupArrayItem::Text(last_edited_by.name.unwrap_or_default())
        }
        PageProperty::Files { files, .. } => {
            let names: Vec<String> = files.into_iter().map(|f| f.name).collect();
            RollupArrayItem::Text(names.join(", "))
        }
        PageProperty::Relation { relation, .. } => {
            let ids: Vec<String> = relation.into_iter().map(|r| r.id).collect();
            RollupArrayItem::Text(ids.join(", "))
        }
        PageProperty::Rollup { rollup, .. } => {
            let result = convert_rollup_value(rollup);
            RollupArrayItem::Text(match result {
                crate::types::RollupResult::String { string } => string.unwrap_or_default(),
                crate::types::RollupResult::Number { number } => {
                    number.map(|n| n.to_string()).unwrap_or_default()
                }
                crate::types::RollupResult::Date { date } => {
                    date.map(|d| d.start.to_string()).unwrap_or_default()
                }
                crate::types::RollupResult::Array { array } => format!("[{} items]", array.len()),
                crate::types::RollupResult::Boolean { boolean } => {
                    boolean.map(|b| b.to_string()).unwrap_or_default()
                }
                crate::types::RollupResult::Unsupported { .. }
                | crate::types::RollupResult::Incomplete { .. } => String::new(),
            })
        }
        _ => RollupArrayItem::Text(String::new()),
    }
}

// --- Main property dispatcher ---

/// Convert individual page property with graceful fallback.
fn convert_page_property(
    name: &str,
    property: notion_client::objects::page::PageProperty,
) -> Result<crate::model::PropertyValue, AppError> {
    use crate::model::PropertyTypeValue;
    use notion_client::objects::page::PageProperty;

    let value = match property {
        PageProperty::Title { title, .. } => PropertyTypeValue::Title {
            title: convert_rich_text_array(title)?,
        },
        PageProperty::RichText { rich_text, .. } => PropertyTypeValue::RichText {
            rich_text: convert_rich_text_array(rich_text)?,
        },
        PageProperty::Number { number, .. } => PropertyTypeValue::Number {
            number: number.and_then(|n| n.as_f64()),
        },
        PageProperty::Checkbox { checkbox, .. } => PropertyTypeValue::Checkbox { checkbox },
        PageProperty::Url { url, .. } => PropertyTypeValue::Url { url },
        PageProperty::Email { email, .. } => PropertyTypeValue::Email { email },
        PageProperty::PhoneNumber { phone_number, .. } => {
            PropertyTypeValue::PhoneNumber { phone_number }
        }
        PageProperty::Select { select, .. } => PropertyTypeValue::Select {
            select: select.map(convert_select_option),
        },
        PageProperty::MultiSelect { multi_select, .. } => PropertyTypeValue::MultiSelect {
            multi_select: multi_select
                .into_iter()
                .map(convert_select_option)
                .collect(),
        },
        PageProperty::Status { status, .. } => PropertyTypeValue::Status {
            status: status.map(convert_select_option),
        },
        PageProperty::Date { date, .. } => PropertyTypeValue::Date {
            date: date.map(convert_notion_date),
        },
        PageProperty::People { people, .. } => {
            let users = people
                .into_iter()
                .map(convert_user)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(convert_partial_user_to_user)
                .collect();
            PropertyTypeValue::People { people: users }
        }
        PageProperty::Files { files, .. } => PropertyTypeValue::Files {
            files: files.into_iter().map(convert_file_ref).collect(),
        },
        PageProperty::CreatedTime { created_time, .. } => {
            PropertyTypeValue::CreatedTime { created_time }
        }
        PageProperty::CreatedBy { created_by, .. } => PropertyTypeValue::CreatedBy {
            created_by: convert_partial_user_to_user(convert_user(created_by)?),
        },
        PageProperty::LastEditedTime {
            last_edited_time, ..
        } => PropertyTypeValue::LastEditedTime {
            last_edited_time: last_edited_time.unwrap_or_default(),
        },
        PageProperty::LastEditedBy { last_edited_by, .. } => PropertyTypeValue::LastEditedBy {
            last_edited_by: convert_partial_user_to_user(convert_user(last_edited_by)?),
        },
        PageProperty::Relation { relation, .. } => {
            let relations = relation
                .into_iter()
                .filter_map(|r| {
                    PageId::parse(&r.id)
                        .map_err(|e| {
                            log::warn!("Skipping relation with invalid ID '{}': {}", r.id, e)
                        })
                        .ok()
                })
                .collect();
            PropertyTypeValue::Relation {
                relation: relations,
            }
        }
        PageProperty::Formula { formula, .. } => PropertyTypeValue::Formula {
            formula: convert_formula_value(formula),
        },
        PageProperty::Rollup { rollup, .. } => PropertyTypeValue::Rollup {
            rollup: convert_rollup_value(rollup),
        },
        PageProperty::UniqueID { unique_id, .. } => match unique_id {
            Some(uid) => PropertyTypeValue::UniqueID {
                unique_id: crate::model::UniqueIdData {
                    number: uid.number.and_then(|n| n.as_i64()).unwrap_or(0),
                    prefix: uid.prefix,
                },
            },
            None => PropertyTypeValue::RichText { rich_text: vec![] },
        },
        PageProperty::Verification { verification, .. } => PropertyTypeValue::Verification {
            verification: verification.map(|v| crate::model::VerificationData {
                state: format!("{:?}", v.state),
                verified_by: v
                    .verified_by
                    .and_then(|u| convert_user(u).ok().map(convert_partial_user_to_user)),
                date: v.date.map(|d| {
                    convert_notion_date(d)
                        .start
                        .and_hms_opt(0, 0, 0)
                        .unwrap_or_default()
                        .and_utc()
                }),
            }),
        },
        PageProperty::Button { .. } => {
            // Button properties are interactive-only; no data to convert
            PropertyTypeValue::RichText { rich_text: vec![] }
        }
    };

    Ok(make_property(name, value))
}

/// Convert individual database property with graceful fallback
fn convert_database_property(
    name: &str,
    property: notion_client::objects::database::DatabaseProperty,
) -> Result<crate::model::DatabaseProperty, AppError> {
    use crate::model::DatabasePropertyType;
    use notion_client::objects::database::DatabaseProperty;

    let property_type = match property {
        DatabaseProperty::Title { .. } => DatabasePropertyType::Title,
        DatabaseProperty::RichText { .. } => DatabasePropertyType::RichText,
        DatabaseProperty::Number { .. } => DatabasePropertyType::Number {
            format: NumberFormat::Number,
        },
        DatabaseProperty::Select { select, .. } => DatabasePropertyType::Select {
            options: select
                .options
                .into_iter()
                .map(|opt| crate::types::SelectOption {
                    id: opt.id.unwrap_or_default(),
                    name: opt.name,
                    color: convert_database_property_color(opt.color.unwrap_or_default()),
                })
                .collect(),
        },
        DatabaseProperty::MultiSelect { multi_select, .. } => DatabasePropertyType::MultiSelect {
            options: multi_select
                .options
                .into_iter()
                .map(|opt| crate::types::SelectOption {
                    id: opt.id.unwrap_or_default(),
                    name: opt.name,
                    color: convert_database_property_color(opt.color.unwrap_or_default()),
                })
                .collect(),
        },
        DatabaseProperty::Date { .. } => DatabasePropertyType::Date,
        DatabaseProperty::Checkbox { .. } => DatabasePropertyType::Checkbox,
        DatabaseProperty::Url { .. } => DatabasePropertyType::Url,
        DatabaseProperty::Email { .. } => DatabasePropertyType::Email,
        DatabaseProperty::PhoneNumber { .. } => DatabasePropertyType::PhoneNumber,
        DatabaseProperty::Files { .. } => DatabasePropertyType::Files,
        DatabaseProperty::Relation { relation, .. } => DatabasePropertyType::Relation {
            database_id: relation.database_id.unwrap_or_default(),
            synced_property_name: relation.synced_property_name,
            synced_property_id: relation.synced_property_id,
        },
        DatabaseProperty::Formula { formula, .. } => DatabasePropertyType::Formula {
            expression: formula.expression,
        },
        DatabaseProperty::Rollup { rollup, .. } => DatabasePropertyType::Rollup {
            relation_property_name: rollup.relation_property_name,
            relation_property_id: rollup.relation_property_id.unwrap_or_default(),
            rollup_property_name: rollup.rollup_property_name,
            rollup_property_id: rollup.rollup_property_id.unwrap_or_default(),
            function: format!("{:?}", rollup.function),
        },
        DatabaseProperty::CreatedTime { .. } => DatabasePropertyType::CreatedTime,
        DatabaseProperty::CreatedBy { .. } => DatabasePropertyType::CreatedBy,
        DatabaseProperty::LastEditedTime { .. } => DatabasePropertyType::LastEditedTime,
        DatabaseProperty::LastEditedBy { .. } => DatabasePropertyType::LastEditedBy,
        DatabaseProperty::Status { status, .. } => DatabasePropertyType::Status {
            options: status
                .options
                .into_iter()
                .map(|opt| crate::types::SelectOption {
                    id: opt.id.unwrap_or_default(),
                    name: opt.name,
                    color: convert_database_property_color(opt.color.unwrap_or_default()),
                })
                .collect(),
        },
        _ => {
            // Graceful fallback for unsupported database property types
            log::debug!(
                "Unsupported database property type for '{}', using Title fallback",
                name
            );
            DatabasePropertyType::Title
        }
    };

    Ok(crate::model::DatabaseProperty {
        id: PropertyName::new(name),
        name: PropertyName::new(name),
        property_type,
    })
}
