// src/output/writer.rs
//! Executes output operations by performing actual I/O.
//!
//! This module is the only place where file I/O operations occur,
//! keeping the rest of the codebase pure and testable.

use super::clipboard::copy_to_clipboard;
use super::types::*;
use crate::error::AppError;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

/// Delivers the output plan, performing all I/O operations.
#[allow(dead_code)] // Used by bin crate
pub fn deliver(plan: OutputPlan) -> Result<OutputReport, AppError> {
    let mut report = OutputReport::new();
    let start_time = Instant::now();

    log::info!(
        "Executing output plan with {} operations",
        plan.operations.len()
    );

    for operation in plan.operations {
        let op_start = Instant::now();
        match execute_operation(&operation) {
            Ok(bytes_written) => {
                let duration_ms = op_start.elapsed().as_millis() as u64;
                report = report.with_completed(CompletedOperation {
                    operation,
                    bytes_written,
                    duration_ms,
                });
            }
            Err(e) => {
                log::error!("Operation failed: {}", e);
                report = report.with_failed(FailedOperation {
                    operation,
                    error: e.to_string(),
                });
            }
        }
    }

    report.stats.total_duration_ms = start_time.elapsed().as_millis() as u64;

    log::info!(
        "Output plan execution complete: {} succeeded, {} failed in {}ms",
        report.stats.operations_completed,
        report.stats.operations_failed,
        report.stats.total_duration_ms
    );

    Ok(report)
}

/// Executes a single output operation.
fn execute_operation(operation: &DeliveryTarget) -> Result<usize, AppError> {
    match operation {
        DeliveryTarget::WriteFile { path, content } => write_file(path, content),
        DeliveryTarget::CreateDirectory { path } => {
            create_directory(path)?;
            Ok(0)
        }
        DeliveryTarget::CopyToClipboard { content } => {
            copy_to_clipboard(content)?;
            Ok(content.len())
        }
        DeliveryTarget::PrintToStdout { content } => {
            print_to_stdout(content)?;
            Ok(content.len())
        }
    }
}

/// Writes content to a file.
fn write_file(path: &Path, content: &str) -> Result<usize, AppError> {
    log::debug!("Writing {} bytes to {}", content.len(), path.display());

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the file
    fs::write(path, content)?;

    log::info!("Wrote file: {}", path.display());
    Ok(content.len())
}

/// Creates a directory.
fn create_directory(path: &Path) -> Result<(), AppError> {
    log::debug!("Creating directory: {}", path.display());

    if path.exists() {
        if path.is_dir() {
            log::debug!("Directory already exists: {}", path.display());
            return Ok(());
        } else {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Path exists but is not a directory: {}", path.display()),
            )));
        }
    }

    fs::create_dir_all(path)?;
    log::info!("Created directory: {}", path.display());
    Ok(())
}

/// Prints content to stdout.
fn print_to_stdout(content: &str) -> Result<(), AppError> {
    print!("{}", content);
    std::io::stdout().flush()?;
    Ok(())
}
