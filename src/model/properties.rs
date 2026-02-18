use crate::types::*;

/// Re-export property types from types module (except PropertyValue which is in property_types)
pub use crate::types::{
    CheckboxProperty, MultiSelectProperty, NumberProperty, SelectProperty, TitleProperty,
};

/// Convert from API property value to domain property value
#[allow(dead_code)]
pub fn parse_property_value(
    property_type: &str,
    value: serde_json::Value,
) -> Result<crate::types::PropertyValue, crate::error::AppError> {
    match property_type {
        "title" => {
            let items: Vec<RichTextItem> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Title(TitleProperty(items)))
        }
        "number" => {
            let number: Option<f64> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Number(NumberProperty(number)))
        }
        "select" => {
            let option: Option<SelectOption> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Select(SelectProperty(option)))
        }
        "multi_select" => {
            let options: Vec<SelectOption> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::MultiSelect(
                MultiSelectProperty(options),
            ))
        }
        "checkbox" => {
            let checked: bool = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Checkbox(CheckboxProperty(
                checked,
            )))
        }
        "rich_text" => {
            let items: Vec<RichTextItem> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Title(TitleProperty(items))) // Reuse TitleProperty for rich_text
        }
        "url" => {
            let url: Option<String> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Url(UrlProperty(url)))
        }
        "email" => {
            let email: Option<String> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Email(EmailProperty(email)))
        }
        "phone_number" => {
            let phone: Option<String> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::PhoneNumber(
                PhoneNumberProperty(phone),
            ))
        }
        "date" => {
            let date: Option<DateValue> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Date(DateProperty(date)))
        }
        "people" => {
            let people: Vec<User> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::People(PeopleProperty(people)))
        }
        "files" => {
            let files: Vec<File> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Files(FilesProperty(files)))
        }
        "relation" => {
            let relations: Vec<PageId> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Relation(RelationProperty(
                relations,
            )))
        }
        "created_time" => {
            let time_str: String = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            // Parse the time string into DateTime
            let time = chrono::DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| {
                    crate::error::AppError::MalformedResponse(format!(
                        "Invalid datetime format: {}",
                        e
                    ))
                })?
                .with_timezone(&chrono::Utc);
            Ok(crate::types::PropertyValue::CreatedTime(
                CreatedTimeProperty(time),
            ))
        }
        "created_by" => {
            let user: User = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::CreatedBy(CreatedByProperty(
                user,
            )))
        }
        "last_edited_time" => {
            let time_str: String = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            // Parse the time string into DateTime
            let time = chrono::DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| {
                    crate::error::AppError::MalformedResponse(format!(
                        "Invalid datetime format: {}",
                        e
                    ))
                })?
                .with_timezone(&chrono::Utc);
            Ok(crate::types::PropertyValue::LastEditedTime(
                LastEditedTimeProperty(time),
            ))
        }
        "last_edited_by" => {
            let user: User = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::LastEditedBy(
                LastEditedByProperty(user),
            ))
        }
        "status" => {
            // Status is often a complex object, handle it as a select option for simplicity
            let option: Option<SelectOption> = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Select(SelectProperty(option)))
        }
        "formula" => {
            let formula: FormulaValue = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Formula(FormulaProperty(
                formula,
            )))
        }
        "rollup" => {
            let rollup: RollupValue = serde_json::from_value(value)
                .map_err(|e| crate::error::AppError::MalformedResponse(e.to_string()))?;
            Ok(crate::types::PropertyValue::Rollup(RollupProperty(rollup)))
        }
        _ => {
            // Instead of failing, create a placeholder that won't break the flow
            log::warn!(
                "Unsupported property type '{}', using empty title as fallback",
                property_type
            );
            Ok(crate::types::PropertyValue::Title(TitleProperty(vec![])))
        }
    }
}
