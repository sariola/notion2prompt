// src/pipeline.rs
//! Pipeline capability traits — abstract the three stages of the Notion-to-prompt pipeline.
//!
//! Each trait describes a single capability, enabling testing each stage in isolation.

use crate::error::AppError;
use crate::model::NotionObject;
use crate::output::OutputReport;
use crate::types::{NotionId, RenderedPrompt};

/// Retrieves Notion content by ID.
#[async_trait::async_trait]
pub trait ContentSource {
    async fn fetch(&self, id: &NotionId) -> Result<NotionObject, AppError>;
}

/// Transforms a NotionObject into a RenderedPrompt.
pub trait PromptComposer {
    fn compose(&self, content: &NotionObject) -> Result<RenderedPrompt, AppError>;
}

/// Delivers a rendered prompt to its destinations.
pub trait PromptDelivery {
    fn deliver(&self, prompt: RenderedPrompt) -> Result<OutputReport, AppError>;
}
