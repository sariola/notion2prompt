// tests/unit/mod.rs
//! Unit tests for notion2prompt components
//!
//! Unit tests focus on testing individual components in isolation,
//! without external dependencies or I/O operations.

#[cfg(test)]
mod types;

#[cfg(test)]
mod error_handling;

// #[cfg(test)]
// mod formatting;

#[cfg(test)]
mod model;

// #[cfg(test)]
// mod performance;

#[cfg(test)]
mod api_parsing;

#[cfg(test)]
mod child_database_embedding;

#[cfg(test)]
mod test_complex_page;

#[cfg(test)]
mod test_block_parent;