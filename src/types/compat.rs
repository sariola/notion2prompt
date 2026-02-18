use super::*;
use serde::{Deserialize, Serialize};

/// Rollup result types - used by legacy PropertyValue system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RollupResult {
    Number { number: Option<f64> },
    Date { date: Option<DateValue> },
    Array { array: Vec<RollupArrayItem> },
    String { string: Option<String> },
    Boolean { boolean: Option<bool> },
    Unsupported { unsupported: serde_json::Value },
    Incomplete { incomplete: serde_json::Value },
}

/// File data representation for property values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileData {
    pub name: String,
    pub file_type: String,
    pub external_url: Option<String>,
}
