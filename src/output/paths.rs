// src/output/paths.rs
//! Pure functions for path calculations and filename generation.
//!
//! This module handles all path-related operations without
//! performing any I/O operations.

#![allow(dead_code)]

use crate::error::AppError;
use std::path::{Path, PathBuf};

/// Creates a clean, safe filename from a title and ID.
pub fn create_clean_filename(title: &str, id: &str, use_short_id: bool) -> String {
    let safe_title = sanitize_filename(title);
    let id_part = if use_short_id {
        id.split('-').next().unwrap_or(id)
    } else {
        id
    };

    if safe_title.is_empty() || safe_title == "unnamed" {
        format!("Untitled Page_{}.md", id_part)
    } else {
        format!("{}_{}.md", safe_title, id_part)
    }
}

/// Sanitizes a string to be safe for use as a filename.
pub fn sanitize_filename(name: &str) -> String {
    let mut safe_name = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>();

    // Trim whitespace and dots
    safe_name = safe_name.trim().trim_matches('.').to_string();

    // Limit length
    if safe_name.len() > 100 {
        safe_name.truncate(100);
    }

    // Default if empty
    if safe_name.is_empty() {
        safe_name = "unnamed".to_string();
    }

    safe_name
}

/// Calculates a relative path from one file to another.
pub fn get_relative_path(from: &Path, to: &Path) -> Result<String, AppError> {
    let from_dir = from.parent().unwrap_or_else(|| Path::new("."));

    let relative = pathdiff::diff_paths(to, from_dir).ok_or_else(|| {
        AppError::PathError(format!(
            "Could not calculate relative path from {} to {}",
            from.display(),
            to.display()
        ))
    })?;

    // Ensure forward slashes for Markdown compatibility
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

/// Calculates the output path for a Notion object.
#[allow(dead_code)] // Public API - may be used by library consumers
pub fn calculate_output_path(
    base_path: &Path,
    _object_type: &str,
    object_id: &str,
    object_title: &str,
) -> PathBuf {
    let filename = create_clean_filename(object_title, object_id, true);
    base_path.join(filename)
}

/// Checks if a path would be safe to write to.
#[allow(dead_code)] // Public API - may be used by library consumers
pub fn is_safe_path(path: &Path, base_dir: &Path) -> bool {
    // Normalize paths
    if let (Ok(canonical_path), Ok(canonical_base)) = (path.canonicalize(), base_dir.canonicalize())
    {
        // Check if path is within base directory
        canonical_path.starts_with(&canonical_base)
    } else {
        // If we can't canonicalize (file doesn't exist yet), check components
        let normalized = normalize_path(path);
        let normalized_base = normalize_path(base_dir);
        normalized.starts_with(&normalized_base)
    }
}

/// Normalizes a path by resolving .. and . components.
#[allow(dead_code)] // Internal utility - may be used by library consumers
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {
                // Skip
            }
            c => {
                components.push(c);
            }
        }
    }

    components.into_iter().collect()
}

/// Generates a unique filename if the original already exists.
#[allow(dead_code)] // Public API - may be used by library consumers
pub fn make_unique_filename(base_path: &Path, original: &str) -> String {
    let path = base_path.join(original);

    // If it doesn't exist, use original
    if !path.exists() {
        return original.to_string();
    }

    // Extract name and extension
    let stem = Path::new(original)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = Path::new(original)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("md");

    // Try with numbers
    for i in 1..1000 {
        let new_name = format!("{}_{}.{}", stem, i, ext);
        if !base_path.join(&new_name).exists() {
            return new_name;
        }
    }

    // Fallback with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}_{}.{}", stem, timestamp, ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello/World"), "Hello_World");
        assert_eq!(sanitize_filename("Test:File*Name"), "Test_File_Name");
        assert_eq!(sanitize_filename("   spaces   "), "spaces");
        assert_eq!(sanitize_filename("...dots..."), "dots");
        assert_eq!(sanitize_filename(""), "unnamed");
    }

    #[test]
    fn test_create_clean_filename() {
        let filename = create_clean_filename("Test Page", "12345678", true);
        assert_eq!(filename, "Test Page_12345678.md");

        let filename = create_clean_filename("", "12345678", true);
        assert_eq!(filename, "Untitled Page_12345678.md");

        let filename = create_clean_filename("Test/Page", "abc-def-ghi", true);
        assert_eq!(filename, "Test_Page_abc.md");
    }

    #[test]
    fn test_normalize_path() {
        let path = Path::new("/home/user/../user/./file.txt");
        let normalized = normalize_path(path);
        assert_eq!(normalized, Path::new("/home/user/file.txt"));
    }
}
