// tests/unit/types.rs
//! Unit tests for domain types

use notion2prompt::types::*;
use notion2prompt::error::domain::ValidationError;

#[cfg(test)]
mod api_key_tests {
    use super::*;
    
    #[test]
    fn valid_api_key_with_secret_prefix() {
        let key = ApiKey::new("secret_abcdefghijklmnopqrstuvwxyz");
        assert!(key.is_ok());
        assert_eq!(key.unwrap().as_str(), "secret_abcdefghijklmnopqrstuvwxyz");
    }
    
    #[test]
    fn valid_api_key_with_ntn_prefix() {
        let key = ApiKey::new("ntn_abcdefghijklmnopqrstuvwxyz");
        assert!(key.is_ok());
        assert_eq!(key.unwrap().as_str(), "ntn_abcdefghijklmnopqrstuvwxyz");
    }
    
    #[test]
    fn invalid_api_key_empty() {
        let key = ApiKey::new("");
        assert!(matches!(
            key,
            Err(ValidationError::InvalidApiKey { reason }) if reason.contains("empty")
        ));
    }
    
    #[test]
    fn invalid_api_key_wrong_prefix() {
        let key = ApiKey::new("invalid_abcdefghijklmnopqrstuvwxyz");
        assert!(matches!(
            key,
            Err(ValidationError::InvalidApiKey { reason }) if reason.contains("must start with")
        ));
    }
    
    #[test]
    fn invalid_api_key_too_short() {
        let key = ApiKey::new("secret_short");
        assert!(matches!(
            key,
            Err(ValidationError::InvalidApiKey { reason }) if reason.contains("too short")
        ));
    }
    
    #[test]
    fn api_key_display_redacts_value() {
        let key = ApiKey::new("secret_supersecretkey123456").unwrap();
        let display = format!("{}", key);
        assert_eq!(display, "secret_sup...");
        assert!(!display.contains("supersecretkey"));
    }
}

#[cfg(test)]
mod template_name_tests {
    use super::*;
    
    #[test]
    fn valid_template_names() {
        let valid_names = vec![
            "template",
            "my-template",
            "template_01",
            "template.hbs",
            "my-template.handlebars",
            "template123",
        ];
        
        for name in valid_names {
            let result = TemplateName::new(name);
            assert!(result.is_ok(), "Template name '{}' should be valid", name);
            assert_eq!(result.unwrap().as_str(), name);
        }
    }
    
    #[test]
    fn invalid_template_name_empty() {
        let result = TemplateName::new("");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidTemplateName { name, reason }) 
                if name.is_empty() && reason.contains("empty")
        ));
    }
    
    #[test]
    fn invalid_template_name_with_spaces() {
        let result = TemplateName::new("template with spaces");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidTemplateName { name, reason }) 
                if name == "template with spaces" && reason.contains("alphanumeric")
        ));
    }
    
    #[test]
    fn invalid_template_extension() {
        let result = TemplateName::new("template.txt");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidTemplateName { name, reason }) 
                if name == "template.txt" && reason.contains("Invalid template extension")
        ));
    }
    
    #[test]
    fn template_name_contains_method() {
        let template = TemplateName::new("my-html-template.hbs").unwrap();
        assert!(template.contains("html"));
        assert!(!template.contains("xml"));
    }
}

#[cfg(test)]
mod validated_url_tests {
    use super::*;
    
    #[test]
    fn valid_https_url() {
        let url = ValidatedUrl::parse("https://notion.so/page");
        assert!(url.is_ok());
        assert_eq!(url.unwrap().as_str(), "https://notion.so/page");
    }
    
    #[test]
    fn valid_http_url() {
        let url = ValidatedUrl::parse("http://localhost:8080/api");
        assert!(url.is_ok());
        assert_eq!(url.unwrap().as_str(), "http://localhost:8080/api");
    }
    
    #[test]
    fn invalid_url_unsupported_scheme() {
        let url = ValidatedUrl::parse("ftp://example.com");
        assert!(matches!(
            url,
            Err(ValidationError::InvalidUrl { url: u, reason }) 
                if u == "ftp://example.com" && reason.contains("HTTP and HTTPS")
        ));
    }
    
    #[test]
    fn invalid_url_malformed() {
        let url = ValidatedUrl::parse("not a url");
        assert!(matches!(
            url,
            Err(ValidationError::InvalidUrl { .. })
        ));
    }
    
    #[test]
    fn validated_url_serialization() {
        let url = ValidatedUrl::parse("https://example.com/path").unwrap();
        let json = serde_json::to_string(&url).unwrap();
        assert_eq!(json, "\"https://example.com/path\"");
        
        let deserialized: ValidatedUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.as_str(), url.as_str());
    }
}

#[cfg(test)]
mod markdown_content_tests {
    use super::*;
    
    #[test]
    fn markdown_content_creation() {
        let content = MarkdownContent::new("# Title");
        assert_eq!(content.as_str(), "# Title");
        assert_eq!(content.len(), 7);
        assert!(!content.is_empty());
    }
    
    #[test]
    fn markdown_content_push_line() {
        let mut content = MarkdownContent::new("# Title");
        content.push_line("Some paragraph");
        content.push_line("Another paragraph");
        
        assert_eq!(content.as_str(), "# Title\nSome paragraph\nAnother paragraph");
    }
    
    #[test]
    fn markdown_content_push_line_empty_start() {
        let mut content = MarkdownContent::new("");
        content.push_line("First line");
        assert_eq!(content.as_str(), "First line");
    }
    
    #[test]
    fn markdown_content_from_string() {
        let s = String::from("# Header");
        let content = MarkdownContent::from(s);
        assert_eq!(content.as_str(), "# Header");
    }
}

#[cfg(test)]
mod validated_path_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    
    #[test]
    fn validated_path_new() {
        let path = ValidatedPath::new("/tmp/test");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().as_path().to_str().unwrap(), "/tmp/test");
    }
    
    #[test]
    fn validated_path_empty() {
        let path = ValidatedPath::new("");
        assert!(matches!(
            path,
            Err(ValidationError::InvalidFilePath { path: p, reason }) 
                if p.is_empty() && reason.contains("empty")
        ));
    }
    
    #[test]
    fn validated_path_existing() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("notion2prompt_test_file.txt");
        fs::write(&test_file, "test").unwrap();
        
        let path = ValidatedPath::existing(&test_file);
        assert!(path.is_ok());
        let validated = path.unwrap();
        assert!(validated.exists());
        assert!(validated.is_file());
        
        // Cleanup
        fs::remove_file(&test_file).unwrap();
    }
    
    #[test]
    fn validated_path_non_existing() {
        let path = ValidatedPath::existing("/tmp/definitely_does_not_exist_12345");
        assert!(matches!(
            path,
            Err(ValidationError::InvalidFilePath { reason, .. }) 
                if reason.contains("does not exist")
        ));
    }
}

#[cfg(test)]
mod warning_tests {
    use super::*;
    
    #[test]
    fn warning_creation() {
        let warning = Warning::new(WarningLevel::Warning, "Something went wrong");
        assert_eq!(warning.level, WarningLevel::Warning);
        assert_eq!(warning.message, "Something went wrong");
        assert_eq!(warning.context, None);
    }
    
    #[test]
    fn warning_with_context() {
        let warning = Warning::new(WarningLevel::Error, "Failed to process")
            .with_context("Block ID: 12345");
        assert_eq!(warning.context, Some("Block ID: 12345".to_string()));
    }
    
    #[test]
    fn warning_display() {
        let warning = Warning::new(WarningLevel::Info, "Processing complete")
            .with_context("10 items processed");
        let display = format!("{}", warning);
        assert_eq!(display, "[Info] Processing complete (10 items processed)");
    }
    
    #[test]
    fn warning_display_without_context() {
        let warning = Warning::new(WarningLevel::Warning, "Skipping invalid item");
        let display = format!("{}", warning);
        assert_eq!(display, "[Warning] Skipping invalid item");
    }
}