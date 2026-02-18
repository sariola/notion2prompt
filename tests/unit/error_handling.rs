// tests/unit/error_handling.rs
//! Unit tests for error handling improvements

#[allow(unused_imports)]
use notion2prompt::error::{
    AppError, 
    ValidationError, 
    ApiError, 
    FormatError, 
    FileError, 
    BuilderError, 
    ConfigError
};

#[cfg(test)]
mod validation_error_tests {
    use super::*;
    
    #[test]
    fn validation_error_messages() {
        let err = ValidationError::InvalidApiKey { 
            reason: "missing prefix".to_string() 
        };
        assert_eq!(err.to_string(), "Invalid API key format: missing prefix");
        
        let err = ValidationError::InvalidNotionId { 
            input: "bad-id".to_string() 
        };
        assert_eq!(err.to_string(), "Invalid Notion ID format: bad-id");
        
        let err = ValidationError::InvalidUrl { 
            url: "ftp://example.com".to_string(),
            reason: "unsupported protocol".to_string() 
        };
        assert_eq!(err.to_string(), "Invalid URL format: ftp://example.com - unsupported protocol");
    }
}

#[cfg(test)]
mod api_error_tests {
    use super::*;
    
    #[test]
    fn api_error_request_failed() {
        let err = ApiError::RequestFailed {
            endpoint: "/v1/pages".to_string(),
            message: "connection timeout".to_string(),
            status_code: None,
        };
        assert_eq!(err.to_string(), "HTTP request failed for /v1/pages: connection timeout");
    }
    
    #[test]
    fn api_error_rate_limit() {
        let err = ApiError::RateLimit { retry_after: 60 };
        assert_eq!(err.to_string(), "Rate limit exceeded, retry after 60 seconds");
    }
    
    #[test]
    fn api_error_not_found() {
        let err = ApiError::NotFound {
            resource_type: "page".to_string(),
            id: "12345".to_string(),
        };
        assert_eq!(err.to_string(), "Resource not found: page with id 12345");
    }
}

#[cfg(test)]
mod format_error_tests {
    use super::*;
    
    #[test]
    fn format_error_block() {
        let err = FormatError::BlockFormatError {
            block_type: "code".to_string(),
            block_id: "abc123".to_string(),
            reason: "missing language".to_string(),
        };
        assert_eq!(
            err.to_string(), 
            "Block formatting failed for code (id: abc123): missing language"
        );
    }
    
    #[test]
    fn format_error_circular_reference() {
        let err = FormatError::CircularReference {
            path: "page1 -> page2 -> page1".to_string(),
        };
        assert_eq!(err.to_string(), "Circular reference detected: page1 -> page2 -> page1");
    }
    
    #[test]
    fn format_error_depth_limit() {
        let err = FormatError::DepthLimitExceeded {
            max_depth: 10,
            path: "root/child1/child2/...".to_string(),
        };
        assert_eq!(
            err.to_string(), 
            "Maximum depth 10 exceeded at: root/child1/child2/..."
        );
    }
}

#[cfg(test)]
mod file_error_tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn file_error_read() {
        let err = FileError::ReadError {
            path: PathBuf::from("/tmp/test.txt"),
            reason: "permission denied".to_string(),
        };
        assert_eq!(
            err.to_string(), 
            "Failed to read file /tmp/test.txt: permission denied"
        );
    }
    
    #[test]
    fn file_error_not_found() {
        let err = FileError::NotFound {
            path: PathBuf::from("/tmp/missing.txt"),
        };
        assert_eq!(err.to_string(), "File not found: /tmp/missing.txt");
    }
}

#[cfg(test)]
mod builder_error_tests {
    use super::*;
    
    #[test]
    fn builder_error_missing_field() {
        let err = BuilderError::MissingField {
            field: "title".to_string(),
        };
        assert_eq!(err.to_string(), "Required field missing: title");
    }
    
    #[test]
    fn builder_error_invalid_field() {
        let err = BuilderError::InvalidField {
            field: "age".to_string(),
            reason: "must be positive".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid field value for age: must be positive");
    }
}

#[cfg(test)]
mod error_context_tests {
    use super::*;
    
    #[test]
    fn error_context_builder() {
        let context = ErrorContext::new("database_fetch")
            .with_context("database_id", "db123")
            .with_context("page", "5")
            .with_context("filter", "status=published")
            .build();
        
        assert_eq!(
            context, 
            "Operation: database_fetch, database_id: db123, page: 5, filter: status=published"
        );
    }
    
    #[test]
    fn error_context_single() {
        let context = ErrorContext::new("block_parse")
            .with_context("block_id", "block456")
            .build();
        
        assert_eq!(context, "Operation: block_parse, block_id: block456");
    }
}