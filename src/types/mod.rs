use thiserror::Error;

mod collections;
mod colors;
mod compat;
mod domain_types;
mod ids;
mod properties;

pub use colors::*;
pub use compat::*;
pub use domain_types::*;
pub use ids::*;
pub use properties::*;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid Notion ID format: {0}")]
    InvalidId(String),

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Collection bounds violation: {actual} items, expected {min}..={max}")]
    BoundsViolation {
        actual: usize,
        min: usize,
        max: usize,
    },

    #[error("Invalid URL: {url} - {reason}")]
    InvalidUrl { url: String, reason: String },

    #[error("Empty required field: {0}")]
    EmptyField(&'static str),

    #[error("Value out of bounds: {value}, expected {min}..={max}")]
    OutOfBounds { value: u32, min: u32, max: u32 },

    #[error("Invalid API key format: {reason}")]
    InvalidApiKey { reason: String },

    #[error("Invalid template name: {name} - {reason}")]
    InvalidTemplateName { name: String, reason: String },

    #[error("Invalid file path: {path} - {reason}")]
    InvalidFilePath { path: String, reason: String },
}
