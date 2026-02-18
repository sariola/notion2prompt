// src/output/mod.rs
//! Output handling with clear separation of planning and execution.
//!
//! This module provides a data-oriented approach to output operations,
//! separating the planning phase (pure functions) from the execution
//! phase (I/O operations).

mod clipboard;
mod paths;
mod types;
mod writer;

// Re-export the public interface
#[allow(unused_imports)] // Used by bin crate
pub use clipboard::copy_to_clipboard;
pub use paths::{create_clean_filename, get_relative_path};
#[allow(unused_imports)] // Used by bin crate
pub use types::{DeliveryTarget, OutputPlan, OutputReport};
#[allow(unused_imports)] // Used by bin crate
pub use writer::deliver;
