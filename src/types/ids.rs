use super::ValidationError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;
use uuid::Uuid;

/// Strong typing for IDs with phantom types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id<T> {
    value: String,
    _phantom: PhantomData<T>,
}

/// Marker types for different ID kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatabaseMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceMarker;

/// Type aliases for specific ID types
pub type PageId = Id<PageMarker>;
pub type BlockId = Id<BlockMarker>;
pub type DatabaseId = Id<DatabaseMarker>;
#[allow(dead_code)]
pub type UserId = Id<UserMarker>;
#[allow(dead_code)]
pub type WorkspaceId = Id<WorkspaceMarker>;

impl<T> Id<T> {
    /// Parse various Notion ID formats into a normalized ID
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let normalized = normalize_notion_id(input)?;
        Ok(Self {
            value: normalized,
            _phantom: PhantomData,
        })
    }

    /// Create an ID from an already normalized string (internal use)
    pub(crate) fn from_normalized(value: String) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    /// Create a new random v4 UUID ID
    pub fn new_v4() -> Self {
        let uuid = Uuid::new_v4();
        Self {
            value: uuid.as_simple().to_string(),
            _phantom: PhantomData,
        }
    }

    /// Get the ID as a string reference
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Get the ID with dashes for API calls
    #[allow(dead_code)]
    pub fn to_dashed(&self) -> String {
        if self.value.len() == 32 && !self.value.contains('-') {
            format!(
                "{}-{}-{}-{}-{}",
                &self.value[0..8],
                &self.value[8..12],
                &self.value[12..16],
                &self.value[16..20],
                &self.value[20..32]
            )
        } else {
            self.value.clone()
        }
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_normalized(value))
    }
}

/// Normalize various Notion ID formats into a consistent format
fn normalize_notion_id(input: &str) -> Result<String, ValidationError> {
    let input = input.trim();

    // Handle URLs
    if input.starts_with("http://") || input.starts_with("https://") {
        if let Some(id) = extract_id_from_url(input) {
            return normalize_notion_id(id);
        }
        return Err(ValidationError::InvalidId(format!(
            "Could not extract ID from URL: {}",
            input
        )));
    }

    // Remove any dashes and validate
    let normalized = input.replace('-', "");

    // Validate length (Notion IDs are 32 hex characters)
    if normalized.len() != 32 {
        return Err(ValidationError::InvalidId(format!(
            "Invalid ID length: expected 32 characters, got {}",
            normalized.len()
        )));
    }

    // Validate hex characters
    if !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidId(
            "ID must contain only hexadecimal characters".to_string(),
        ));
    }

    Ok(normalized.to_lowercase())
}

/// Extract ID from Notion URL
fn extract_id_from_url(url: &str) -> Option<&str> {
    // Handle various Notion URL formats
    let url = url.trim_end_matches('/');

    // Format: https://www.notion.so/[workspace]/[title]-[id]
    if let Some(pos) = url.rfind('-') {
        let potential_id = &url[pos + 1..];
        if potential_id.len() == 32 {
            return Some(potential_id);
        }
    }

    // Format: https://www.notion.so/[id]
    if let Some(pos) = url.rfind('/') {
        let potential_id = &url[pos + 1..];
        if potential_id.len() == 32 || (potential_id.len() == 36 && potential_id.contains('-')) {
            return Some(potential_id);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_parsing() {
        // Test direct ID
        let id = PageId::parse("550e8400e29b41d4a716446655440000").unwrap();
        assert_eq!(id.as_str(), "550e8400e29b41d4a716446655440000");

        // Test dashed ID
        let id = PageId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(id.as_str(), "550e8400e29b41d4a716446655440000");

        // Test URL
        let id = PageId::parse("https://www.notion.so/Test-Page-550e8400e29b41d4a716446655440000")
            .unwrap();
        assert_eq!(id.as_str(), "550e8400e29b41d4a716446655440000");
    }

    #[test]
    fn test_invalid_ids() {
        assert!(PageId::parse("too-short").is_err());
        assert!(PageId::parse("not-hex-chars-00000000000000000").is_err());
        assert!(PageId::parse("").is_err());
    }

    #[test]
    fn test_to_dashed() {
        let id = PageId::parse("550e8400e29b41d4a716446655440000").unwrap();
        assert_eq!(id.to_dashed(), "550e8400-e29b-41d4-a716-446655440000");
    }
}

/// NotionId - A general-purpose Notion ID that can represent any type of object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotionId(String); // Store the non-hyphenated version internally

impl NotionId {
    /// Returns the canonical non-hyphenated ID.
    #[allow(dead_code)]
    pub fn value(&self) -> &str {
        &self.0
    }

    /// Returns the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the hyphenated UUID format for API compatibility.
    pub fn to_hyphenated(&self) -> String {
        if self.0.len() == 32 && !self.0.contains('-') {
            format!(
                "{}-{}-{}-{}-{}",
                &self.0[0..8],
                &self.0[8..12],
                &self.0[12..16],
                &self.0[16..20],
                &self.0[20..32]
            )
        } else {
            self.0.clone()
        }
    }

    /// Creates a NotionId from a validated hex string.
    fn from_hex(hex: &str) -> Result<Self, ValidationError> {
        if hex.len() == 32 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(NotionId(hex.to_lowercase()))
        } else {
            Err(ValidationError::InvalidId(format!(
                "Invalid Notion ID format: {}",
                hex
            )))
        }
    }

    /// Creates a new NotionId with validation.
    #[allow(dead_code)]
    pub fn new(id: String) -> Result<Self, ValidationError> {
        Self::parse(&id)
    }

    /// Parses various Notion ID formats.
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let cleaned = input.trim().trim_end_matches('/');

        // 1. UUID format with dashes
        if let Ok(uuid) = Uuid::parse_str(cleaned) {
            return Ok(NotionId(uuid.as_simple().to_string()));
        }

        // 2. Direct 32-char hex ID
        if cleaned.len() == 32 && cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
            return Self::from_hex(cleaned);
        }

        // 3. Extract from URLs
        if cleaned.contains("notion") {
            return Self::extract_from_url(cleaned);
        }

        Err(ValidationError::InvalidId(format!(
            "Could not parse Notion ID from: {}",
            input
        )))
    }

    /// Extracts ID from Notion URLs.
    fn extract_from_url(url: &str) -> Result<Self, ValidationError> {
        lazy_static::lazy_static! {
            static ref ID_REGEX: Regex = Regex::new(
                r"(?:[/-])([a-fA-F0-9]{32}|[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12})(?:[/?#]|$)"
            ).expect("Failed to compile Notion ID regex - this is a bug in the code");
        }

        if let Some(captures) = ID_REGEX.captures(url) {
            if let Some(id_match) = captures.get(1) {
                let id = id_match.as_str().replace('-', "");
                return Self::from_hex(&id);
            }
        }

        Err(ValidationError::InvalidId(format!(
            "No valid ID found in URL: {}",
            url
        )))
    }

    // Backwards compatibility
    pub fn value_hyphenated(&self) -> String {
        self.to_hyphenated()
    }
}

impl fmt::Display for NotionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for NotionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NotionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NotionId::parse(&s).map_err(serde::de::Error::custom)
    }
}

// Conversions from specific ID types to NotionId
impl From<PageId> for NotionId {
    fn from(id: PageId) -> Self {
        NotionId(id.as_str().to_string())
    }
}

impl From<BlockId> for NotionId {
    fn from(id: BlockId) -> Self {
        NotionId(id.as_str().to_string())
    }
}

impl From<DatabaseId> for NotionId {
    fn from(id: DatabaseId) -> Self {
        NotionId(id.as_str().to_string())
    }
}

impl From<&PageId> for NotionId {
    fn from(id: &PageId) -> Self {
        NotionId(id.as_str().to_string())
    }
}

impl From<&BlockId> for NotionId {
    fn from(id: &BlockId) -> Self {
        NotionId(id.as_str().to_string())
    }
}

impl From<&DatabaseId> for NotionId {
    fn from(id: &DatabaseId) -> Self {
        NotionId(id.as_str().to_string())
    }
}
