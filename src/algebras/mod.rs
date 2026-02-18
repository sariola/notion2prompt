//! Capability algebras for notion2prompt.
//!
//! This module defines algebraic traits that represent **capabilities**,
//! not implementations. Each trait is:
//!
//! - **Object-safe** — can be used as `dyn Trait`
//! - **Documented with laws** — properties that all implementations must satisfy
//! - **Async via `async_trait`** — Rust native async in traits
//!
//! # Architecture
//!
//! The algebra layer sits between domain logic and interpreters:
//!
//! ```text
//! Application Layer (main.rs)
//!         ↓
//! Domain Layer (algebras/)
//!         ↓
//! Interpreter Layer (interpreters/)
//! ```
//!
//! # Capability Traits
//!
//! - [`NotionContent`] — Content retrieval from Notion API
//! - [`VisitTracker`] — Visit tracking for cycle detection
//! - [`DepthLimiter`] — Depth limiting for recursion control
//!
//! # Extension Traits
//!
//! Base traits use concrete domain types (`Page`, `Database`, `Block`) for
//! object safety. Extension traits provide higher-level convenience methods
//! built from the base trait operations.
//!
//! # Laws
//!
//! Each trait documents algebraic laws that all implementations must satisfy.
//! These are verified via law tests in each module's test suite.

pub mod content;
pub mod error;
pub mod state;

// Re-exports for convenience
pub use content::{NotionContent, NotionContentExt};
pub use error::{FetchError, TrackError};
pub use state::{DepthLimiter, VisitTracker};
