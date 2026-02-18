use crate::types::*;
use serde::{Deserialize, Serialize};

/// Property value — wraps a typed value with its property ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyValue {
    pub id: PropertyName,
    #[serde(flatten)]
    pub type_specific_value: PropertyTypeValue,
}

impl PropertyValue {
    /// Returns the Notion API type name for this property value.
    pub fn type_name(&self) -> &'static str {
        match &self.type_specific_value {
            PropertyTypeValue::Title { .. } => "title",
            PropertyTypeValue::RichText { .. } => "rich_text",
            PropertyTypeValue::Number { .. } => "number",
            PropertyTypeValue::Select { .. } => "select",
            PropertyTypeValue::MultiSelect { .. } => "multi_select",
            PropertyTypeValue::Status { .. } => "status",
            PropertyTypeValue::Date { .. } => "date",
            PropertyTypeValue::Formula { .. } => "formula",
            PropertyTypeValue::Relation { .. } => "relation",
            PropertyTypeValue::Rollup { .. } => "rollup",
            PropertyTypeValue::People { .. } => "people",
            PropertyTypeValue::Files { .. } => "files",
            PropertyTypeValue::Checkbox { .. } => "checkbox",
            PropertyTypeValue::Url { .. } => "url",
            PropertyTypeValue::Email { .. } => "email",
            PropertyTypeValue::PhoneNumber { .. } => "phone_number",
            PropertyTypeValue::CreatedTime { .. } => "created_time",
            PropertyTypeValue::CreatedBy { .. } => "created_by",
            PropertyTypeValue::LastEditedTime { .. } => "last_edited_time",
            PropertyTypeValue::LastEditedBy { .. } => "last_edited_by",
            PropertyTypeValue::UniqueID { .. } => "unique_id",
            PropertyTypeValue::Verification { .. } => "verification",
        }
    }
}

/// The specific value types for properties - compatibility layer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyTypeValue {
    Title {
        title: Vec<RichTextItem>,
    },
    RichText {
        rich_text: Vec<RichTextItem>,
    },
    Number {
        number: Option<f64>,
    },
    Select {
        select: Option<SelectOption>,
    },
    MultiSelect {
        multi_select: Vec<SelectOption>,
    },
    Status {
        status: Option<SelectOption>,
    },
    Date {
        date: Option<DateValue>,
    },
    Formula {
        formula: FormulaResult,
    },
    Relation {
        relation: Vec<PageId>,
    },
    Rollup {
        rollup: RollupResult,
    },
    People {
        people: Vec<User>,
    },
    Files {
        files: Vec<File>,
    },
    Checkbox {
        checkbox: bool,
    },
    Url {
        url: Option<String>,
    },
    Email {
        email: Option<String>,
    },
    PhoneNumber {
        phone_number: Option<String>,
    },
    CreatedTime {
        created_time: chrono::DateTime<chrono::Utc>,
    },
    CreatedBy {
        created_by: User,
    },
    LastEditedTime {
        last_edited_time: chrono::DateTime<chrono::Utc>,
    },
    LastEditedBy {
        last_edited_by: User,
    },
    UniqueID {
        unique_id: UniqueIdData,
    },
    Verification {
        verification: Option<VerificationData>,
    },
}

/// Unique ID data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniqueIdData {
    pub number: i64,
    pub prefix: Option<String>,
}

/// Verification data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationData {
    pub state: String,
    pub verified_by: Option<User>,
    pub date: Option<chrono::DateTime<chrono::Utc>>,
}
