// src/formatting/properties/render.rs
//! Rendering implementations for formatted properties.
//!
//! This module implements the rendering logic for property types,
//! keeping rendering separate from data representation.

use super::types::*;

impl Renderable for FormattedProperty {
    fn render_text(&self) -> String {
        match self {
            FormattedProperty::Text(s) => s.clone(),
            FormattedProperty::Number(n) => n.render_text(),
            FormattedProperty::Boolean(b) => if *b { "Yes" } else { "No" }.to_string(),
            FormattedProperty::Date(d) => d.to_string(),
            FormattedProperty::Select(s) => s.clone(),
            FormattedProperty::MultiSelect(items) => items.join(", "),
            FormattedProperty::Status(s) => s.clone(),
            FormattedProperty::People(names) => names.join(", "),
            FormattedProperty::Files(files) => files
                .iter()
                .map(|f| f.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            FormattedProperty::Url(link) => link.text.as_ref().unwrap_or(&link.url).clone(),
            FormattedProperty::Email(e) => e.clone(),
            FormattedProperty::Phone(p) => p.clone(),
            FormattedProperty::Formula(f) => f.render_text(),
            FormattedProperty::Relation(r) => r.render_text(),
            FormattedProperty::Rollup(r) => r.render_text(),
            FormattedProperty::CreatedTime(dt) => format_datetime(dt),
            FormattedProperty::LastEditedTime(dt) => format_datetime(dt),
            FormattedProperty::CreatedBy(name) => name.clone(),
            FormattedProperty::LastEditedBy(name) => name.clone(),
            FormattedProperty::UniqueId(id) => id.clone(),
            FormattedProperty::Verification(v) => v.render_text(),
            FormattedProperty::List(items) => items
                .iter()
                .map(|item| item.render_text())
                .collect::<Vec<_>>()
                .join(", "),
            FormattedProperty::Empty => String::new(),
        }
    }

    fn render_markdown(&self) -> String {
        match self {
            FormattedProperty::Boolean(b) => if *b { "✅" } else { "⬜" }.to_string(),
            FormattedProperty::Files(files) => files
                .iter()
                .map(|f| f.render_markdown())
                .collect::<Vec<_>>()
                .join(", "),
            FormattedProperty::Url(link) => link.render_markdown(),
            _ => self.render_text(),
        }
    }
}

impl Renderable for NumberValue {
    fn render_text(&self) -> String {
        match self.format {
            NumberFormat::Auto => format_number_auto(self.value),
            NumberFormat::Integer => format!("{:.0}", self.value),
            NumberFormat::Decimal(places) => {
                format!("{:.prec$}", self.value, prec = places as usize)
            }
            NumberFormat::Percentage => format!("{:.1}%", self.value * 100.0),
            NumberFormat::Currency(fmt) => match fmt.position {
                CurrencyPosition::Prefix => format!("{}{:.2}", fmt.symbol, self.value),
                CurrencyPosition::Suffix => format!("{:.2}{}", self.value, fmt.symbol),
            },
        }
    }
}

impl Renderable for FileLink {
    fn render_text(&self) -> String {
        self.name.clone()
    }

    fn render_markdown(&self) -> String {
        format!("[{}]({})", self.name, self.url)
    }
}

impl Renderable for UrlLink {
    fn render_text(&self) -> String {
        self.text.as_ref().unwrap_or(&self.url).clone()
    }

    fn render_markdown(&self) -> String {
        if self.url.is_empty() {
            self.render_text()
        } else {
            let display = self.text.as_ref().unwrap_or(&self.url);
            format!("[{}]({})", display, self.url)
        }
    }
}

impl Renderable for FormulaValue {
    fn render_text(&self) -> String {
        match self {
            FormulaValue::String(s) => s.clone(),
            FormulaValue::Number(n) => n.render_text(),
            FormulaValue::Boolean(b) => if *b { "Yes" } else { "No" }.to_string(),
            FormulaValue::Date(d) => d.clone(),
        }
    }
}

impl Renderable for FormattedRelation {
    fn render_text(&self) -> String {
        let base = self.ids.join(", ");
        if self.has_more {
            format!("{}...", base)
        } else {
            base
        }
    }
}

impl Renderable for RollupValue {
    fn render_text(&self) -> String {
        match self {
            RollupValue::Number(n) => n.render_text(),
            RollupValue::Date(d) => d.clone(),
            RollupValue::Array(items) => items.join(", "),
            RollupValue::String(s) => s.clone(),
            RollupValue::Boolean(b) => if *b { "Yes" } else { "No" }.to_string(),
            RollupValue::Unsupported => "[Unsupported Rollup]".to_string(),
            RollupValue::Incomplete => "[Incomplete Rollup]".to_string(),
        }
    }
}

impl Renderable for VerificationValue {
    fn render_text(&self) -> String {
        match &self.verified_by {
            Some(user) => format!("{} ({})", self.state, user),
            None => self.state.clone(),
        }
    }
}

// --- Helper Functions ---

/// Formats a number with automatic precision.
fn format_number_auto(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else {
        format!("{:.2}", n)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

/// Formats a DateTime in a human-readable format.
fn format_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// Escapes text for use in a Markdown table cell.
#[allow(dead_code)]
pub fn escape_for_table_cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', "<br>")
}
