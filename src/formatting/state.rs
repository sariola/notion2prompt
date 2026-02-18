// src/formatting/state.rs
//! Immutable formatting state with pure transitions for Markdown generation.
//!
//! This module provides a purely functional approach to managing formatting state
//! during the transformation of Notion blocks to formatted content.

use crate::constants::BLOCK_MAX_RENDER_DEPTH;
use im_rc::HashSet;

// --- Core Types ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ListKind {
    Numbered(usize), // Current number for this level
    Bulleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableContext {
    pub column_count: usize,
    pub has_column_header: bool,
    pub has_row_header: bool,
    pub alignments: Vec<ColumnAlignment>,
    pub row_count: usize,
    pub header_rendered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    Left,
    #[allow(dead_code)]
    Center,
    #[allow(dead_code)]
    Right,
}

/// Immutable formatting context that tracks state during block transformation.
/// All methods return new instances, preserving immutability.
///
/// Several fields and methods are infrastructure for the full immutable state
/// machine (cycle detection, depth tracking, exit transitions). They are not
/// yet wired into callers but form the correct API surface for future use.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FormatContext {
    /// Currently processed block IDs to prevent infinite recursion
    processed_ids: HashSet<String>,
    /// Current recursion depth
    recursion_depth: usize,
    /// Stack of list contexts for nested lists
    list_stack: Vec<ListContext>,
    /// Current table context if within a table
    table_context: Option<TableContext>,
    /// Current indentation level (for nested blocks)
    indent_level: usize,
    /// Current block nesting depth
    block_depth: usize,
    /// Whether we're inside a columns layout
    in_columns: bool,
    /// Current column index if in columns
    column_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListContext {
    kind: ListKind,
    depth: usize,
}

// --- Public API ---

#[allow(dead_code)]
impl FormatContext {
    /// Creates a new, empty formatting context.
    pub fn new() -> Self {
        Self {
            processed_ids: HashSet::new(),
            recursion_depth: 0,
            list_stack: Vec::new(),
            table_context: None,
            indent_level: 0,
            block_depth: 0,
            in_columns: false,
            column_index: None,
        }
    }

    /// Checks if a block ID has already been visited (cycle detection).
    pub fn already_visited(&self, block_id: &str) -> bool {
        self.processed_ids.contains(block_id)
    }

    /// Returns a new context with the block ID marked as visited.
    pub fn with_visited(&self, block_id: String) -> Self {
        let mut new_context = self.clone();
        new_context.processed_ids.insert(block_id);
        new_context
    }

    /// Checks if the recursion depth limit has been reached.
    pub fn depth_limit_reached(&self) -> bool {
        self.recursion_depth >= BLOCK_MAX_RENDER_DEPTH
    }

    /// Returns the current recursion depth.
    pub fn current_recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    /// Enters a new block level, incrementing recursion depth.
    pub fn enter_block(&self) -> Self {
        let mut new_context = self.clone();
        new_context.recursion_depth += 1;
        new_context.block_depth += 1;
        new_context
    }

    /// Exits a block level, decrementing recursion depth.
    pub fn exit_block(&self) -> Self {
        let mut new_context = self.clone();
        new_context.recursion_depth = new_context.recursion_depth.saturating_sub(1);
        new_context.block_depth = new_context.block_depth.saturating_sub(1);
        new_context
    }

    /// Enters children processing.
    pub fn enter_children(&self) -> Self {
        let mut new_context = self.clone();
        new_context.indent_level += 1;
        new_context
    }

    /// Checks if currently in a list.
    pub fn is_in_list(&self) -> bool {
        !self.list_stack.is_empty()
    }

    /// Returns the current list depth.
    pub fn list_depth(&self) -> usize {
        self.list_stack.len()
    }

    /// Enters a numbered list context.
    pub fn enter_numbered_list(&self) -> Self {
        let mut new_context = self.clone();
        let depth = self.list_stack.len();
        new_context.list_stack.push(ListContext {
            kind: ListKind::Numbered(1),
            depth,
        });
        new_context
    }

    /// Enters a bulleted list context.
    pub fn enter_bulleted_list(&self) -> Self {
        let mut new_context = self.clone();
        let depth = self.list_stack.len();
        new_context.list_stack.push(ListContext {
            kind: ListKind::Bulleted,
            depth,
        });
        new_context
    }

    /// Gets the current list number (for numbered lists).
    pub fn current_list_number(&self) -> usize {
        self.list_stack
            .last()
            .and_then(|ctx| match &ctx.kind {
                ListKind::Numbered(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(1)
    }

    /// Increments the current list number.
    pub fn increment_list_number(&self) -> Self {
        let mut new_context = self.clone();
        if let Some(last) = new_context.list_stack.last_mut() {
            if let ListKind::Numbered(n) = &mut last.kind {
                *n += 1;
            }
        }
        new_context
    }

    /// Exits the current list context.
    pub fn exit_list(&self) -> Self {
        let mut new_context = self.clone();
        new_context.list_stack.pop();
        new_context
    }

    /// Enters a table context.
    pub fn enter_table(&self, table_width: usize) -> Self {
        let mut new_context = self.clone();
        new_context.table_context = Some(TableContext {
            column_count: table_width,
            has_column_header: true, // Default to true
            has_row_header: false,
            alignments: vec![ColumnAlignment::Left; table_width],
            row_count: 0,
            header_rendered: false,
        });
        new_context
    }

    /// Exits the table context.
    pub fn exit_table(&self) -> Self {
        let mut new_context = self.clone();
        new_context.table_context = None;
        new_context
    }

    /// Processes a table row — advances the row counter.
    pub fn process_table_row(&self) -> Self {
        let mut new_context = self.clone();
        if let Some(ref mut table) = new_context.table_context {
            table.row_count += 1;
            if table.row_count == 1 && table.has_column_header {
                table.header_rendered = true;
            }
        }
        new_context
    }

    /// Checks if this is the first table row (for header separator).
    pub fn is_first_table_row(&self) -> bool {
        self.table_context
            .as_ref()
            .map(|t| t.row_count == 0)
            .unwrap_or(false)
    }

    /// Enters a columns layout.
    pub fn enter_columns(&self) -> Self {
        let mut new_context = self.clone();
        new_context.in_columns = true;
        new_context.column_index = Some(0);
        new_context
    }

    /// Exits a columns layout.
    pub fn exit_columns(&self) -> Self {
        let mut new_context = self.clone();
        new_context.in_columns = false;
        new_context.column_index = None;
        new_context
    }

    /// Gets the current indent level.
    pub fn indent_level(&self) -> usize {
        self.indent_level
    }

    /// Checks if spacing is needed before this block.
    pub fn needs_spacing(&self) -> bool {
        // Add spacing between top-level blocks
        self.block_depth == 0 && !self.is_in_list()
    }

    /// Enters a toggle context — semantic marker for toggle nesting.
    pub fn enter_toggle(&self) -> Self {
        // Toggles just increase indentation
        self.enter_children()
    }

    /// Enters a callout context — semantic marker for callout nesting.
    pub fn enter_callout(&self) -> Self {
        // Callouts just increase indentation
        self.enter_children()
    }
}

impl Default for FormatContext {
    fn default() -> Self {
        Self::new()
    }
}

// --- Helper Functions ---
