// src/output/clipboard.rs
//! Platform-specific clipboard operations.
//!
//! This module handles clipboard operations across different platforms
//! using a strategy pattern for clean separation of concerns.

use crate::error::AppError;
use std::io::Write;
use std::process::{Command, Stdio};

/// Copies content to the system clipboard.
#[allow(dead_code)] // Used by bin crate
pub fn copy_to_clipboard(content: &str) -> Result<(), AppError> {
    log::debug!("Copying {} characters to clipboard", content.len());

    // Log child database content analysis for clipboard
    let child_db_count = content.matches("child_database").count();
    let table_count = content.matches("|--|").count();
    let database_count = content.matches("Database:").count();

    log::debug!("Clipboard content analysis: {} child_database mentions, {} markdown tables, {} database headers", 
        child_db_count, table_count, database_count);

    if child_db_count > 0 || table_count > 0 {
        log::info!("✓ Clipboard content contains child database content: {} child_database mentions, {} tables", 
            child_db_count, table_count);
    } else {
        log::warn!("⚠️  Clipboard content does NOT contain child database content!");
    }

    // Try arboard first (cross-platform)
    match try_arboard_clipboard(content) {
        Ok(()) => {
            log::info!("Content copied to clipboard using arboard");
            return Ok(());
        }
        Err(e) => {
            log::debug!("Arboard failed: {}, trying platform-specific methods", e);
        }
    }

    // Fall back to platform-specific methods
    let result = copy_with_platform_command(content);

    match &result {
        Ok(()) => log::info!("Content copied to clipboard using platform command"),
        Err(e) => log::error!("Failed to copy to clipboard: {}", e),
    }

    result
}

/// Tries to copy using the arboard crate.
fn try_arboard_clipboard(content: &str) -> Result<(), AppError> {
    use arboard::Clipboard;

    let mut clipboard = Clipboard::new()
        .map_err(|e| AppError::Clipboard(format!("Failed to access clipboard: {}", e)))?;

    clipboard
        .set_text(content)
        .map_err(|e| AppError::Clipboard(format!("Failed to set clipboard text: {}", e)))?;

    Ok(())
}

/// Platform-specific clipboard command execution.
#[cfg(target_os = "linux")]
fn copy_with_platform_command(content: &str) -> Result<(), AppError> {
    // Detect Wayland vs X11
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|s| s == "wayland");

    if is_wayland {
        copy_with_wl_copy(content)
    } else {
        copy_with_xclip(content)
    }
}

#[cfg(target_os = "macos")]
fn copy_with_platform_command(content: &str) -> Result<(), AppError> {
    copy_with_pbcopy(content)
}

#[cfg(target_os = "windows")]
fn copy_with_platform_command(content: &str) -> Result<(), AppError> {
    copy_with_clip_exe(content)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn copy_with_platform_command(_content: &str) -> Result<(), AppError> {
    Err(AppError::Clipboard(
        "Clipboard not supported on this platform".to_string(),
    ))
}

/// Linux/Wayland: Copy using wl-copy.
#[cfg(target_os = "linux")]
fn copy_with_wl_copy(content: &str) -> Result<(), AppError> {
    log::debug!("Attempting to copy with wl-copy");

    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Clipboard(format!("Failed to spawn wl-copy: {}", e)))?;

    // Write content to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| AppError::Clipboard(format!("Failed to write to wl-copy: {}", e)))?;
    }

    // Wait for completion
    let output = child
        .wait_with_output()
        .map_err(|e| AppError::Clipboard(format!("Failed to wait for wl-copy: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Clipboard(format!("wl-copy failed: {}", stderr)))
    }
}

/// Linux/X11: Copy using xclip.
#[cfg(target_os = "linux")]
fn copy_with_xclip(content: &str) -> Result<(), AppError> {
    log::debug!("Attempting to copy with xclip");

    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Clipboard(format!("Failed to spawn xclip: {}", e)))?;

    // Write content to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| AppError::Clipboard(format!("Failed to write to xclip: {}", e)))?;
    }

    // Wait for completion
    let output = child
        .wait_with_output()
        .map_err(|e| AppError::Clipboard(format!("Failed to wait for xclip: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Clipboard(format!("xclip failed: {}", stderr)))
    }
}

/// macOS: Copy using pbcopy.
#[cfg(target_os = "macos")]
fn copy_with_pbcopy(content: &str) -> Result<(), AppError> {
    log::debug!("Attempting to copy with pbcopy");

    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Clipboard(format!("Failed to spawn pbcopy: {}", e)))?;

    // Write content to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| AppError::Clipboard(format!("Failed to write to pbcopy: {}", e)))?;
    }

    // Wait for completion
    let output = child
        .wait_with_output()
        .map_err(|e| AppError::Clipboard(format!("Failed to wait for pbcopy: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Clipboard(format!("pbcopy failed: {}", stderr)))
    }
}

/// Windows: Copy using clip.exe.
#[cfg(target_os = "windows")]
fn copy_with_clip_exe(content: &str) -> Result<(), AppError> {
    log::debug!("Attempting to copy with clip.exe");

    let mut child = Command::new("clip")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Clipboard(format!("Failed to spawn clip.exe: {}", e)))?;

    // Write content to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| AppError::Clipboard(format!("Failed to write to clip.exe: {}", e)))?;
    }

    // Wait for completion
    let output = child
        .wait_with_output()
        .map_err(|e| AppError::Clipboard(format!("Failed to wait for clip.exe: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Clipboard(format!("clip.exe failed: {}", stderr)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires clipboard access
    fn test_clipboard_small_content() {
        let content = "Hello, clipboard!";
        let result = copy_to_clipboard(content);
        assert!(result.is_ok());
    }
}
