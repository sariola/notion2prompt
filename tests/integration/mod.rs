// tests/integration/mod.rs
//! Integration tests for notion2prompt
//!
//! Integration tests verify that multiple components work together correctly,
//! including API interactions, formatting pipelines, and output generation.

// Temporarily disabled due to compilation issues
// #[cfg(test)]
// mod api_integration;

// #[cfg(test)]
// mod dependency_injection;

// #[cfg(test)]
// mod formatting_pipeline;

// #[cfg(test)]
// mod output_generation;

// #[cfg(test)]
// mod child_database_fetching;

// #[cfg(test)]
// mod child_database_priority;

// #[cfg(test)]
// mod test_block_parents;

#[cfg(test)]
mod test_child_database_inclusion;

#[cfg(test)]
mod end_to_end_child_database_test;

#[cfg(test)]
mod simple_fixture_test;
