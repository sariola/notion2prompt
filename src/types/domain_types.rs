// src/types/domain_types.rs
//! Domain-specific newtypes for type safety and validation.

use super::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

/// API key for Notion API authentication
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Create a new API key with validation
    pub fn new(key: impl Into<String>) -> Result<Self, ValidationError> {
        let key = key.into();

        // Validate API key format
        if key.is_empty() {
            return Err(ValidationError::InvalidApiKey {
                reason: "API key cannot be empty".to_string(),
            });
        }

        if !key.starts_with("secret_") && !key.starts_with("ntn_") {
            return Err(ValidationError::InvalidApiKey {
                reason: "API key must start with 'secret_' or 'ntn_'".to_string(),
            });
        }

        if key.len() < 20 {
            return Err(ValidationError::InvalidApiKey {
                reason: "API key is too short".to_string(),
            });
        }

        Ok(Self(key))
    }

    /// Get the API key as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create an API key without validation (only for testing)
    #[cfg(test)]
    pub fn new_unchecked(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact API key in display
        write!(f, "{}...", &self.0[..10])
    }
}

/// Validated URL type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedUrl(Url);

// Manual Serialize/Deserialize implementation for Url
impl Serialize for ValidatedUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ValidatedUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ValidatedUrl::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl ValidatedUrl {
    /// Create a new validated URL
    pub fn parse(url: &str) -> Result<Self, ValidationError> {
        match Url::parse(url) {
            Ok(parsed_url) => {
                // Additional validation
                if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
                    return Err(ValidationError::InvalidUrl {
                        url: url.to_string(),
                        reason: "Only HTTP and HTTPS URLs are supported".to_string(),
                    });
                }
                Ok(Self(parsed_url))
            }
            Err(e) => Err(ValidationError::InvalidUrl {
                url: url.to_string(),
                reason: e.to_string(),
            }),
        }
    }

    /// Get the URL as a string
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the underlying URL
    #[allow(dead_code)]
    pub fn as_url(&self) -> &Url {
        &self.0
    }
}

impl fmt::Display for ValidatedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Template name with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TemplateName(String);

impl TemplateName {
    /// Valid template extensions
    const VALID_EXTENSIONS: &'static [&'static str] = &["hbs", "handlebars", "mustache"];

    /// Create a new template name with validation
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValidationError::InvalidTemplateName {
                name: name.clone(),
                reason: "Template name cannot be empty".to_string(),
            });
        }

        // Check for valid characters
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(ValidationError::InvalidTemplateName {
                name: name.clone(),
                reason: "Template name can only contain alphanumeric characters, hyphens, underscores, and dots".to_string(),
            });
        }

        // Check extension if present
        if let Some(ext) = name.split('.').next_back() {
            if name.contains('.') && !Self::VALID_EXTENSIONS.contains(&ext) {
                return Err(ValidationError::InvalidTemplateName {
                    name: name.clone(),
                    reason: format!(
                        "Invalid template extension. Valid extensions: {:?}",
                        Self::VALID_EXTENSIONS
                    ),
                });
            }
        }

        Ok(Self(name))
    }

    /// Get the template name as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the template name contains a substring
    #[allow(dead_code)]
    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }
}

impl fmt::Display for TemplateName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The name of a property on a Notion page or database.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PropertyName(String);

impl PropertyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PropertyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::borrow::Borrow<str> for PropertyName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<String> for PropertyName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PropertyName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// The final output of the render stage — a complete prompt ready for delivery.
#[derive(Debug, Clone)]
pub struct RenderedPrompt(String);

impl RenderedPrompt {
    pub fn new(content: String) -> Self {
        Self(content)
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for RenderedPrompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Warning message with structured information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    pub level: WarningLevel,
    pub message: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningLevel {
    Info,
    Warning,
    Error,
}

impl Warning {
    #[allow(dead_code)]
    pub fn new(level: WarningLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            context: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.level, self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_validation() {
        assert!(ApiKey::new("secret_abcdefghijklmnopqrs").is_ok());
        assert!(ApiKey::new("ntn_abcdefghijklmnopqrs").is_ok());
        assert!(ApiKey::new("").is_err());
        assert!(ApiKey::new("invalid_key").is_err());
        assert!(ApiKey::new("secret_short").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(ValidatedUrl::parse("https://notion.so/page").is_ok());
        assert!(ValidatedUrl::parse("http://localhost:8080").is_ok());
        assert!(ValidatedUrl::parse("ftp://example.com").is_err());
        assert!(ValidatedUrl::parse("not a url").is_err());
    }

    #[test]
    fn test_template_name_validation() {
        assert!(TemplateName::new("template").is_ok());
        assert!(TemplateName::new("my-template.hbs").is_ok());
        assert!(TemplateName::new("template_01").is_ok());
        assert!(TemplateName::new("").is_err());
        assert!(TemplateName::new("template with spaces").is_err());
        assert!(TemplateName::new("template.invalid").is_err());
    }
}
