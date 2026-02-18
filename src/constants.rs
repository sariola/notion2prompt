// src/constants.rs
//! Domain constants that define the operational boundaries of the system.
//!
//! Each constant is named for the domain concept it constrains, not its
//! technical role. Reading these constants should tell you the story
//! of how the system operates: how deep it recurses, how much it fetches,
//! how it allocates memory.

// ---------------------------------------------------------------------------
// Notion API boundaries
// ---------------------------------------------------------------------------

/// How many objects the Notion API returns per page of results.
///
/// The Notion API maximum is 100. We use the maximum to minimize
/// round-trips during recursive fetching.
pub const NOTION_API_PAGE_SIZE: usize = 100;

/// Maximum nesting depth when recursively fetching from the Notion API.
///
/// Notion pages can nest arbitrarily deep (pages within databases within
/// pages). This limit prevents stack overflow and runaway fetches.
/// 50 levels is far deeper than any real Notion workspace.
pub const NOTION_MAX_FETCH_DEPTH: u8 = 50;

// ---------------------------------------------------------------------------
// Formatting boundaries
// ---------------------------------------------------------------------------

/// Maximum nesting depth when recursively formatting blocks to markdown.
///
/// This is the formatting-layer equivalent of `NOTION_MAX_FETCH_DEPTH`.
/// Prevents infinite recursion in the block visitor when rendering deeply
/// nested toggle blocks, lists, or synced content.
#[allow(dead_code)]
pub const BLOCK_MAX_RENDER_DEPTH: usize = 100;

/// Maximum nesting depth for bulleted/numbered/todo lists.
///
/// Markdown renderers and human readability both degrade beyond ~10 levels
/// of list indentation. This caps the indent prefix generation.
#[allow(dead_code)]
pub const LIST_MAX_NESTING: usize = 10;

/// Number of spaces per indentation level in formatted output.
#[allow(dead_code)]
pub const INDENT_SPACES: usize = 2;

/// Maximum column width for rendered markdown tables.
#[allow(dead_code)]
pub const TABLE_MAX_COLUMNS: u32 = 100;

// ---------------------------------------------------------------------------
// String capacity hints (performance, not correctness)
// ---------------------------------------------------------------------------

/// Estimated characters per block, used to pre-allocate output strings.
///
/// This is a performance hint, not a constraint. Over-estimating wastes
/// a little memory; under-estimating causes reallocation.
pub const CHARS_PER_BLOCK_ESTIMATE: usize = 256;

/// Estimated characters per child block for nested content.
#[allow(dead_code)]
pub const CHARS_PER_CHILD_ESTIMATE: usize = 200;

/// Default initial capacity for output string builders.
#[allow(dead_code)]
pub const OUTPUT_STRING_INITIAL_CAPACITY: usize = 512;

// ---------------------------------------------------------------------------
// Error display
// ---------------------------------------------------------------------------

/// Maximum characters shown when previewing error response bodies.
#[allow(dead_code)]
pub const ERROR_BODY_PREVIEW_LENGTH: usize = 200;
