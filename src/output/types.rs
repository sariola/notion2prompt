// src/output/types.rs
//! Type definitions for output operations.
//!
//! This module defines immutable types for planning and executing
//! output operations following data-oriented design principles.

use std::path::PathBuf;

/// Represents a complete output plan.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Used by bin crate
pub struct OutputPlan {
    /// List of operations to perform
    pub operations: Vec<DeliveryTarget>,
}

#[allow(dead_code)] // Used by bin crate
impl OutputPlan {
    /// Creates a new empty output plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an operation to the plan.
    pub fn with_operation(mut self, operation: DeliveryTarget) -> Self {
        self.operations.push(operation);
        self
    }
}

/// Represents a single output operation.
#[derive(Debug, Clone)]
pub enum DeliveryTarget {
    /// Write content to a file
    WriteFile { path: PathBuf, content: String },
    /// Create a directory
    #[allow(dead_code)] // Used in advanced output scenarios
    CreateDirectory { path: PathBuf },
    /// Copy content to clipboard
    #[allow(dead_code)] // Used when clipboard output is enabled
    CopyToClipboard { content: String },
    /// Print to stdout
    #[allow(dead_code)] // Used when pipe output is enabled
    PrintToStdout { content: String },
}

/// Result of executing an output plan.
#[derive(Debug, Clone)]
pub struct OutputReport {
    /// Successfully completed operations
    pub completed: Vec<CompletedOperation>,
    /// Failed operations with errors
    pub failed: Vec<FailedOperation>,
    /// Execution statistics
    pub stats: ExecutionStats,
}

impl Default for OutputReport {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputReport {
    /// Creates a new empty report.
    pub fn new() -> Self {
        Self {
            completed: Vec::new(),
            failed: Vec::new(),
            stats: ExecutionStats::default(),
        }
    }

    /// Adds a completed operation to the report.
    pub fn with_completed(mut self, operation: CompletedOperation) -> Self {
        self.stats.operations_completed += 1;
        self.stats.bytes_written += operation.bytes_written;
        self.completed.push(operation);
        self
    }

    /// Adds a failed operation to the report.
    pub fn with_failed(mut self, operation: FailedOperation) -> Self {
        self.stats.operations_failed += 1;
        self.failed.push(operation);
        self
    }

    /// Checks if all operations succeeded.
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// A successfully completed operation.
#[derive(Debug, Clone)]
pub struct CompletedOperation {
    pub operation: DeliveryTarget,
    pub bytes_written: usize,
    #[allow(dead_code)] // Used in performance monitoring
    pub duration_ms: u64,
}

/// A failed operation with error information.
#[derive(Debug, Clone)]
pub struct FailedOperation {
    #[allow(dead_code)] // Used in error reporting
    pub operation: DeliveryTarget,
    pub error: String,
}

/// Execution statistics.
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub operations_completed: usize,
    pub operations_failed: usize,
    pub bytes_written: usize,
    pub total_duration_ms: u64,
}
