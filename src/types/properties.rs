use super::PageId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type-safe property system with phantom types
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Property<T> {
    pub id: String,
    pub name: String,
    pub value: T,
}

/// Property value types with strong typing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleProperty(pub Vec<RichTextItem>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberProperty(pub Option<f64>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectProperty(pub Option<SelectOption>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiSelectProperty(pub Vec<SelectOption>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateProperty(pub Option<DateValue>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaProperty(pub FormulaValue);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationProperty(pub Vec<PageId>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollupProperty(pub RollupValue);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeopleProperty(pub Vec<User>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesProperty(pub Vec<File>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxProperty(pub bool);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlProperty(pub Option<String>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailProperty(pub Option<String>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneNumberProperty(pub Option<String>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatedTimeProperty(pub chrono::DateTime<chrono::Utc>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatedByProperty(pub User);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastEditedTimeProperty(pub chrono::DateTime<chrono::Utc>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastEditedByProperty(pub User);

/// The kind of rich text content — a typed vocabulary replacing stringly-typed dispatch.
///
/// Each variant carries its specific data, making invalid states
/// unrepresentable: you can't have a "mention" type with no mention data,
/// or an "equation" type with no expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RichTextType {
    Text { content: String, link: Option<Link> },
    Mention(MentionData),
    Equation(EquationData),
}

/// Rich text item with formatting annotations.
///
/// The `text_type` field carries the content variant — text, mention, or equation —
/// and `plain_text` provides the fallback rendering for any variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichTextItem {
    pub text_type: RichTextType,
    pub annotations: Annotations,
    pub plain_text: String,
    pub href: Option<String>,
}

impl RichTextItem {
    /// Create a plain text item — the most common rich text variant.
    ///
    /// This is the vocabulary for constructing rich text in builders,
    /// tests, and adapters. Instead of 6 fields with Nones, just:
    /// ```ignore
    /// RichTextItem::plain_text("hello")
    /// ```
    #[allow(dead_code)]
    pub fn plain_text(text: &str) -> Self {
        Self {
            text_type: RichTextType::Text {
                content: text.to_string(),
                link: None,
            },
            annotations: Annotations::default(),
            plain_text: text.to_string(),
            href: None,
        }
    }
}

/// Legacy text content — kept for API deserialization compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
    pub link: Option<Link>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Annotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
    pub color: crate::types::Color,
}

/// Select option
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub id: String,
    pub name: String,
    pub color: crate::types::Color,
}

/// Date value with optional time and end date
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateValue {
    pub start: chrono::NaiveDate,
    pub end: Option<chrono::NaiveDate>,
    pub time_zone: Option<String>,
}

/// Formula value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormulaValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(DateValue),
}

/// Rollup value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RollupValue {
    Number(f64),
    Date(DateValue),
    Array(Vec<RollupArrayItem>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RollupArrayItem {
    Title(Vec<RichTextItem>),
    Number(f64),
    Date(DateValue),
    /// Catch-all for property types that don't have a dedicated variant
    /// (Select, Checkbox, Url, People, Formula results, etc.)
    Text(String),
}

/// User representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.name, &self.email) {
            (Some(name), _) => write!(f, "{}", name),
            (None, Some(email)) => write!(f, "{}", email),
            (None, None) => write!(f, "User {}", self.id),
        }
    }
}

/// Partial user representation (used in mentions)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartialUser {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

impl fmt::Display for PartialUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "User {}", self.id),
        }
    }
}

/// File representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub url: String,
    pub expiry_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Property type enumeration for runtime dispatch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    Title(TitleProperty),
    Number(NumberProperty),
    Select(SelectProperty),
    MultiSelect(MultiSelectProperty),
    Date(DateProperty),
    Formula(FormulaProperty),
    Relation(RelationProperty),
    Rollup(RollupProperty),
    People(PeopleProperty),
    Files(FilesProperty),
    Checkbox(CheckboxProperty),
    Url(UrlProperty),
    Email(EmailProperty),
    PhoneNumber(PhoneNumberProperty),
    CreatedTime(CreatedTimeProperty),
    CreatedBy(CreatedByProperty),
    LastEditedTime(LastEditedTimeProperty),
    LastEditedBy(LastEditedByProperty),
}

impl PropertyValue {
    /// Get the property type as a string
    #[allow(dead_code)]
    pub fn property_type(&self) -> &'static str {
        match self {
            PropertyValue::Title(_) => "title",
            PropertyValue::Number(_) => "number",
            PropertyValue::Select(_) => "select",
            PropertyValue::MultiSelect(_) => "multi_select",
            PropertyValue::Date(_) => "date",
            PropertyValue::Formula(_) => "formula",
            PropertyValue::Relation(_) => "relation",
            PropertyValue::Rollup(_) => "rollup",
            PropertyValue::People(_) => "people",
            PropertyValue::Files(_) => "files",
            PropertyValue::Checkbox(_) => "checkbox",
            PropertyValue::Url(_) => "url",
            PropertyValue::Email(_) => "email",
            PropertyValue::PhoneNumber(_) => "phone_number",
            PropertyValue::CreatedTime(_) => "created_time",
            PropertyValue::CreatedBy(_) => "created_by",
            PropertyValue::LastEditedTime(_) => "last_edited_time",
            PropertyValue::LastEditedBy(_) => "last_edited_by",
        }
    }
}

/// Mention data with type information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MentionData {
    #[serde(flatten)]
    pub mention_type: MentionType,
}

/// Different types of mentions in rich text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MentionType {
    User {
        user: PartialUser,
    },
    Page {
        page: PageReference,
    },
    Database {
        database: DatabaseReference,
    },
    Date {
        date: DateValue,
    },
    LinkPreview {
        link_preview: LinkPreviewReference,
    },
    #[serde(rename = "link_mention")]
    LinkMention {
        url: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageReference {
    pub id: super::NotionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseReference {
    pub id: super::NotionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkPreviewReference {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EquationData {
    pub expression: String,
}

/// File reference
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileReference {
    pub name: String,
    pub url: String,
}

/// Formula result types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormulaResult {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(DateValue),
}

/// Relation value
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationValue {
    pub id: super::NotionId,
}

/// Unique ID data
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniqueIdData {
    pub number: i64,
    pub prefix: Option<String>,
}

/// URL value
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrlValue {
    pub url: String,
}

/// Verification data
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationData {
    pub state: String,
    pub verified_by: Option<User>,
    pub date: Option<DateValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_types() {
        let title = PropertyValue::Title(TitleProperty(vec![]));
        assert_eq!(title.property_type(), "title");

        let number = PropertyValue::Number(NumberProperty(Some(42.0)));
        assert_eq!(number.property_type(), "number");
    }
}
