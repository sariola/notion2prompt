// src/formatting/properties/formatters.rs
//! Type-specific formatting implementations for Notion properties.
//!
//! This module contains the logic for converting Notion property values
//! into formatted representations, organized by property type.

use super::types::*;
use crate::error::AppError;
use crate::formatting::rich_text::rich_text_to_markdown;
use crate::model::{PropertyTypeValue, PropertyValue, UniqueIdData, VerificationData};
use crate::types::{
    DateValue, File, FormulaResult, PageId, RollupArrayItem, RollupResult, SelectOption, UrlValue,
    User,
};

// --- Main Formatting Entry Point ---

/// Formats a property value into a structured representation.
pub fn format_property_value(value: &PropertyValue) -> Result<FormattedProperty, AppError> {
    use PropertyTypeValue::*;

    log::trace!(
        "Formatting property ID: {}, Type: {}",
        value.id,
        value.type_name()
    );

    match &value.type_specific_value {
        Title { title } => format_title(title),
        RichText { rich_text } => format_rich_text(rich_text),
        Number { number } => Ok(format_number(*number)),
        Select { select } => Ok(format_select(select.as_ref())),
        MultiSelect { multi_select } => Ok(format_multi_select(multi_select)),
        Status { status } => Ok(format_status(status.as_ref())),
        Date { date } => Ok(format_date(date.as_ref())),
        People { people } => Ok(format_people(people)),
        Files { files } => Ok(format_files(files)),
        Checkbox { checkbox } => Ok(FormattedProperty::Boolean(*checkbox)),
        Url { url } => Ok(format_url_string(url.as_deref())),
        Email { email } => Ok(format_email(email.as_deref())),
        PhoneNumber { phone_number } => Ok(format_phone(phone_number.as_deref())),
        Formula { formula } => format_formula(formula),
        Relation { relation } => Ok(format_relation(relation, None)),
        Rollup { rollup } => format_rollup(rollup),
        CreatedTime { created_time } => Ok(FormattedProperty::CreatedTime(*created_time)),
        CreatedBy { created_by } => Ok(FormattedProperty::CreatedBy(created_by.to_string())),
        LastEditedTime { last_edited_time } => {
            Ok(FormattedProperty::LastEditedTime(*last_edited_time))
        }
        LastEditedBy { last_edited_by } => {
            Ok(FormattedProperty::LastEditedBy(last_edited_by.to_string()))
        }
        UniqueID { unique_id } => Ok(format_unique_id(unique_id)),
        Verification { verification } => match verification {
            Some(v) => Ok(format_verification(v)),
            None => Ok(FormattedProperty::Empty),
        },
    }
}

// --- Text Formatters ---

fn format_text_property(
    items: &[crate::types::RichTextItem],
) -> Result<FormattedProperty, AppError> {
    let text = rich_text_to_markdown(items)?;
    Ok(if text.is_empty() {
        FormattedProperty::Empty
    } else {
        FormattedProperty::Text(text)
    })
}

fn format_title(title: &[crate::types::RichTextItem]) -> Result<FormattedProperty, AppError> {
    format_text_property(title)
}

fn format_rich_text(
    rich_text: &[crate::types::RichTextItem],
) -> Result<FormattedProperty, AppError> {
    format_text_property(rich_text)
}

// --- Number Formatter ---

fn format_number(number: Option<f64>) -> FormattedProperty {
    match number {
        Some(n) => FormattedProperty::Number(NumberValue::new(n)),
        None => FormattedProperty::Empty,
    }
}

// --- Select Formatters ---

fn format_select(select: Option<&SelectOption>) -> FormattedProperty {
    match select {
        Some(opt) => FormattedProperty::Select(opt.name.clone()),
        None => FormattedProperty::Empty,
    }
}

fn format_multi_select(multi_select: &[SelectOption]) -> FormattedProperty {
    if multi_select.is_empty() {
        FormattedProperty::Empty
    } else {
        let names: Vec<String> = multi_select.iter().map(|opt| opt.name.clone()).collect();
        FormattedProperty::MultiSelect(names)
    }
}

fn format_status(status: Option<&SelectOption>) -> FormattedProperty {
    match status {
        Some(opt) => FormattedProperty::Status(opt.name.clone()),
        None => FormattedProperty::Empty,
    }
}

// --- Date Formatter ---

fn format_date(date: Option<&DateValue>) -> FormattedProperty {
    match date {
        Some(d) => FormattedProperty::Date(DateRange {
            start: d.start.to_string(),
            end: d.end.map(|e| e.to_string()),
        }),
        None => FormattedProperty::Empty,
    }
}

// --- People Formatter ---

fn format_people(people: &[User]) -> FormattedProperty {
    if people.is_empty() {
        FormattedProperty::Empty
    } else {
        let names: Vec<String> = people.iter().map(|p| p.to_string()).collect();
        FormattedProperty::People(names)
    }
}

// --- Files Formatter ---

fn format_files(files: &[File]) -> FormattedProperty {
    if files.is_empty() {
        FormattedProperty::Empty
    } else {
        let file_links: Vec<FileLink> = files
            .iter()
            .map(|f| FileLink {
                name: f.name.clone(),
                url: f.url.clone(),
            })
            .collect();
        if file_links.is_empty() {
            FormattedProperty::Empty
        } else {
            FormattedProperty::Files(file_links)
        }
    }
}

// --- URL Formatter ---

#[allow(dead_code)]
fn format_url(url: Option<&UrlValue>) -> FormattedProperty {
    match url {
        Some(u) => FormattedProperty::Url(UrlLink {
            url: u.url.clone(),
            text: None,
        }),
        None => FormattedProperty::Empty,
    }
}

fn format_url_string(url: Option<&str>) -> FormattedProperty {
    match url {
        Some(u) => FormattedProperty::Url(UrlLink {
            url: u.to_string(),
            text: None,
        }),
        None => FormattedProperty::Empty,
    }
}

// --- Contact Formatters ---

fn format_email(email: Option<&str>) -> FormattedProperty {
    match email {
        Some(e) => FormattedProperty::Email(e.to_string()),
        None => FormattedProperty::Empty,
    }
}

fn format_phone(phone: Option<&str>) -> FormattedProperty {
    match phone {
        Some(p) => FormattedProperty::Phone(p.to_string()),
        None => FormattedProperty::Empty,
    }
}

// --- Formula Formatter ---

fn format_formula(formula: &FormulaResult) -> Result<FormattedProperty, AppError> {
    let formatted = match formula {
        FormulaResult::String(s) => FormulaValue::String(s.clone()),
        FormulaResult::Number(n) => FormulaValue::Number(NumberValue::new(*n)),
        FormulaResult::Boolean(b) => FormulaValue::Boolean(*b),
        FormulaResult::Date(d) => FormulaValue::Date(d.start.to_string()),
    };
    Ok(FormattedProperty::Formula(formatted))
}

// --- Relation Formatter ---

fn format_relation(relation: &[PageId], has_more: Option<bool>) -> FormattedProperty {
    let ids: Vec<String> = relation.iter().map(|r| r.as_str().to_string()).collect();
    if ids.is_empty() {
        FormattedProperty::Empty
    } else {
        FormattedProperty::Relation(FormattedRelation {
            ids,
            has_more: has_more.unwrap_or(false),
        })
    }
}

// --- Rollup Formatter ---

fn format_rollup(rollup: &RollupResult) -> Result<FormattedProperty, AppError> {
    let formatted = match rollup {
        RollupResult::Number { number } => match number {
            Some(n) => RollupValue::Number(NumberValue::new(*n)),
            None => return Ok(FormattedProperty::Empty),
        },
        RollupResult::Date { date } => match date {
            Some(d) => RollupValue::Date(d.start.to_string()),
            None => return Ok(FormattedProperty::Empty),
        },
        RollupResult::Array { array } => {
            let items: Vec<String> = array.iter().map(format_rollup_array_item).collect();
            RollupValue::Array(items)
        }
        RollupResult::String { string } => match string {
            Some(s) => RollupValue::String(s.clone()),
            None => return Ok(FormattedProperty::Empty),
        },
        RollupResult::Boolean { boolean } => match boolean {
            Some(b) => RollupValue::Boolean(*b),
            None => return Ok(FormattedProperty::Empty),
        },
        RollupResult::Unsupported { .. } => RollupValue::Unsupported,
        RollupResult::Incomplete { .. } => RollupValue::Incomplete,
    };
    Ok(FormattedProperty::Rollup(formatted))
}

/// Formats a rollup array item to string
fn format_rollup_array_item(item: &RollupArrayItem) -> String {
    match item {
        RollupArrayItem::Title(title) => rich_text_to_markdown(title).unwrap_or_default(),
        RollupArrayItem::Number(n) => n.to_string(),
        RollupArrayItem::Date(d) => d.start.to_string(),
        RollupArrayItem::Text(s) => s.clone(),
    }
}

/// Formats a property type value directly without creating a temporary PropertyValue.
#[allow(dead_code)]
fn format_property_type_value(value: &PropertyTypeValue) -> Result<String, AppError> {
    use PropertyTypeValue::*;

    match value {
        Title { title } | RichText { rich_text: title } => rich_text_to_markdown(title),
        Number { number } => Ok(number.map_or_else(String::new, |n| n.to_string())),
        Select { select } => Ok(select.as_ref().map_or_else(String::new, |s| s.name.clone())),
        MultiSelect { multi_select } => Ok(multi_select
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")),
        Date { date } => Ok(date
            .as_ref()
            .map_or_else(String::new, |d| d.start.to_string())),
        People { people } => Ok(people
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")),
        Status { status } => Ok(status.as_ref().map_or_else(String::new, |s| s.name.clone())),
        Email { email } => Ok(email.clone().unwrap_or_default()),
        PhoneNumber { phone_number } => Ok(phone_number.clone().unwrap_or_default()),
        Checkbox { checkbox } => Ok(checkbox.to_string()),
        Url { url } => Ok(url.clone().unwrap_or_default()),
        _ => Ok(String::new()),
    }
}

// --- ID Formatters ---

fn format_unique_id(unique_id: &UniqueIdData) -> FormattedProperty {
    let id = format!(
        "{}{}",
        unique_id.prefix.as_deref().unwrap_or(""),
        unique_id.number
    );
    FormattedProperty::UniqueId(id)
}

// --- Verification Formatter ---

fn format_verification(verification: &VerificationData) -> FormattedProperty {
    let state = verification.state.clone();
    let verified_by = verification.verified_by.as_ref().map(|u| u.to_string());
    FormattedProperty::Verification(VerificationValue { state, verified_by })
}
