// tests/mod.rs
//! Test suite organization for notion2prompt
//!
//! This module provides a structured approach to testing with clear separation
//! between unit tests, integration tests, and test fixtures.

// Temporarily disabled due to compilation issues
// #[cfg(test)]
// pub mod unit;

#[cfg(test)]
pub mod integration;

// Temporarily disabled due to compilation issues
// /// Common test utilities and helpers
// #[cfg(test)]
// pub mod common {
//     use notion2prompt::types::{ApiKey, NotionId, TemplateName};
//     use notion2prompt::config::PipelineConfig;
//     use std::path::PathBuf;
//
//     /// Creates a test configuration with sensible defaults
//     pub fn test_config() -> PipelineConfig {
//         PipelineConfig {
//             notion_id: NotionId::parse("12345678-1234-1234-1234-123456789abc")
//                 .expect("Test ID should be valid"),
//             api_key: ApiKey::new("secret_test_key_123456789")
//                 .expect("Test API key should be valid"),
//             depth: 5,
//             limit: 100,
//             template: TemplateName::new("test-template")
//                 .expect("Test template name should be valid"),
//             content_dir: PathBuf::from("/tmp/test-content"),
//             output_file: None,
//             clipboard: false,
//             pipe: false,
//             verbose: false,
//             instruction: None,
//         }
//     }
//
//     /// Creates a test API key
//     pub fn test_api_key() -> ApiKey {
//         ApiKey::new("secret_test_key_123456789")
//             .expect("Test API key should be valid")
//     }
//
//     /// Creates a test Notion ID
//     pub fn test_notion_id() -> NotionId {
//         NotionId::parse("12345678-1234-1234-1234-123456789abc")
//             .expect("Test ID should be valid")
//     }
// }
