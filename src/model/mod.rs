mod block;
pub mod blocks;
pub mod common;
pub mod properties;
mod property_value;

pub use block::{Block, BlockVisitor};
pub use blocks::*;
pub use common::*;
pub use property_value::{PropertyTypeValue, PropertyValue, UniqueIdData, VerificationData};

use crate::types::{BlockId, DatabaseId, NotionId, PageId, PropertyName};
use serde::{Deserialize, Serialize};

/// The root object that can be fetched from Notion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotionObject {
    Page(Page),
    Database(Database),
    Block(Block),
}

impl NotionObject {
    pub fn id(&self) -> NotionId {
        match self {
            NotionObject::Page(page) => NotionId::from(&page.id),
            NotionObject::Database(database) => NotionId::from(&database.id),
            NotionObject::Block(block) => NotionId::from(block.id()),
        }
    }

    pub fn object_type_name(&self) -> &str {
        match self {
            NotionObject::Page(_) => "page",
            NotionObject::Database(_) => "database",
            NotionObject::Block(_) => "block",
        }
    }

    /// Returns a human-readable display title for this object.
    pub fn display_title(&self) -> String {
        match self {
            NotionObject::Page(page) => page.title().as_str().to_string(),
            NotionObject::Database(db) => {
                let text = db.title().as_plain_text();
                if text.is_empty() {
                    "Untitled Database".to_string()
                } else {
                    text
                }
            }
            NotionObject::Block(block) => format!("Block {}", block.id().as_str()),
        }
    }
}

/// A Notion page
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub title: PageTitle,
    pub url: String,
    pub blocks: Vec<Block>,
    pub properties: std::collections::HashMap<PropertyName, PropertyValue>,
    pub parent: Option<Parent>,
    pub archived: bool,
}

impl Page {
    /// Get the page title
    pub fn title(&self) -> &PageTitle {
        &self.title
    }
}

/// A Notion database
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Database {
    pub id: DatabaseId,
    pub title: DatabaseTitle,
    pub url: String,
    pub pages: Vec<Page>,
    pub properties: std::collections::HashMap<PropertyName, DatabaseProperty>,
    pub parent: Option<Parent>,
    pub archived: bool,
}

impl Database {
    /// Get the database title
    pub fn title(&self) -> &DatabaseTitle {
        &self.title
    }
}

/// Parent reference with typed IDs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Parent {
    #[serde(rename = "page_id")]
    Page { page_id: PageId },
    #[serde(rename = "database_id")]
    Database { database_id: DatabaseId },
    #[serde(rename = "block_id")]
    Block { block_id: BlockId },
    #[serde(rename = "workspace")]
    Workspace,
}

/// Page title
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageTitle(String);

impl PageTitle {
    pub fn new(title: impl Into<String>) -> Self {
        Self(title.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PageTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Database title
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseTitle(Vec<crate::types::RichTextItem>);

impl DatabaseTitle {
    pub fn new(items: Vec<crate::types::RichTextItem>) -> Self {
        Self(items)
    }

    pub fn as_plain_text(&self) -> String {
        self.0
            .iter()
            .map(|item| item.plain_text.as_str())
            .collect::<Vec<_>>()
            .join("")
    }

    #[allow(dead_code)]
    pub fn items(&self) -> &[crate::types::RichTextItem] {
        &self.0
    }
}

impl std::fmt::Display for DatabaseTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_plain_text())
    }
}

/// Database property definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseProperty {
    pub id: PropertyName,
    pub name: PropertyName,
    pub property_type: DatabasePropertyType,
}

/// Database property types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DatabasePropertyType {
    Title,
    RichText,
    Number {
        format: NumberFormat,
    },
    Select {
        options: Vec<crate::types::SelectOption>,
    },
    MultiSelect {
        options: Vec<crate::types::SelectOption>,
    },
    Date,
    Formula {
        expression: String,
    },
    Relation {
        database_id: String,
        synced_property_name: Option<String>,
        synced_property_id: Option<String>,
    },
    Rollup {
        relation_property_name: String,
        relation_property_id: String,
        rollup_property_name: String,
        rollup_property_id: String,
        function: String,
    },
    People,
    Files,
    Checkbox,
    Url,
    Email,
    PhoneNumber,
    CreatedTime,
    CreatedBy,
    LastEditedTime,
    LastEditedBy,
    Status {
        options: Vec<crate::types::SelectOption>,
    },
}

impl std::fmt::Display for DatabasePropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabasePropertyType::Title => write!(f, "title"),
            DatabasePropertyType::RichText => write!(f, "rich_text"),
            DatabasePropertyType::Number { .. } => write!(f, "number"),
            DatabasePropertyType::Select { .. } => write!(f, "select"),
            DatabasePropertyType::MultiSelect { .. } => write!(f, "multi_select"),
            DatabasePropertyType::Date => write!(f, "date"),
            DatabasePropertyType::Formula { .. } => write!(f, "formula"),
            DatabasePropertyType::Relation { .. } => write!(f, "relation"),
            DatabasePropertyType::Rollup { .. } => write!(f, "rollup"),
            DatabasePropertyType::People => write!(f, "people"),
            DatabasePropertyType::Files => write!(f, "files"),
            DatabasePropertyType::Checkbox => write!(f, "checkbox"),
            DatabasePropertyType::Url => write!(f, "url"),
            DatabasePropertyType::Email => write!(f, "email"),
            DatabasePropertyType::PhoneNumber => write!(f, "phone_number"),
            DatabasePropertyType::CreatedTime => write!(f, "created_time"),
            DatabasePropertyType::CreatedBy => write!(f, "created_by"),
            DatabasePropertyType::LastEditedTime => write!(f, "last_edited_time"),
            DatabasePropertyType::LastEditedBy => write!(f, "last_edited_by"),
            DatabasePropertyType::Status { .. } => write!(f, "status"),
        }
    }
}

/// Number format options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberFormat {
    Number,
    NumberWithCommas,
    Percent,
    Dollar,
    CanadianDollar,
    Euro,
    Pound,
    Yen,
    Ruble,
    Rupee,
    Won,
    Yuan,
    Real,
    Lira,
    Rupiah,
    Franc,
    HongKongDollar,
    NewZealandDollar,
    Krona,
    NorwegianKrone,
    MexicanPeso,
    Rand,
    NewTaiwanDollar,
    DanishKrone,
    Zloty,
    Baht,
    Forint,
    Koruna,
    Shekel,
    ChileanPeso,
    PhilippinePeso,
    Dirham,
    ColombianPeso,
    Riyal,
    Ringgit,
    Leu,
    ArgentinePeso,
    UruguayanPeso,
}

/// Represents either a page ID or database ID
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PageOrDbId {
    Page { page_id: PageId },
    Database { database_id: DatabaseId },
}

impl PageOrDbId {
    /// Get the ID value regardless of type
    #[allow(dead_code)]
    pub fn id(&self) -> &str {
        match self {
            PageOrDbId::Page { page_id } => page_id.as_str(),
            PageOrDbId::Database { database_id } => database_id.as_str(),
        }
    }

    /// Get the object type
    #[allow(dead_code)]
    pub fn object_type(&self) -> &str {
        match self {
            PageOrDbId::Page { .. } => "page",
            PageOrDbId::Database { .. } => "database",
        }
    }
}
