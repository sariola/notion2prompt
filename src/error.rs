// src/error.rs
//! Application error types with structured error handling.
//!
//! Error types form the vocabulary for failure modes in the system.
//! Each error variant tells the story of what went wrong and where,
//! enabling composable recovery strategies.

use std::fmt;
use thiserror::Error;

/// Notion API error codes as a typed vocabulary.
///
/// Instead of matching against magic strings like `"rate_limited"`,
/// the domain vocabulary is encoded in the type system. Each variant
/// tells you exactly what the Notion API reported and enables
/// pattern-based recovery without stringly-typed dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants constructed via from_api_response in bin target
pub enum NotionErrorCode {
    /// API rate limit exceeded — back off and retry
    RateLimited,
    /// The requested object does not exist or is inaccessible
    ObjectNotFound,
    /// API key is invalid or expired
    Unauthorized,
    /// API key lacks permission for this resource
    RestrictedResource,
    /// Request body contains invalid JSON
    InvalidJson,
    /// Request parameters failed Notion's validation
    ValidationFailed,
    /// Conflict with current state of the resource
    Conflict,
    /// Notion internal server error
    InternalError,
    /// Notion is temporarily unavailable
    ServiceUnavailable,
    /// HTTP status code fallback when the error body is unparseable
    HttpStatus(u16),
    /// An error code this client doesn't recognize yet
    Unknown(String),
}

impl NotionErrorCode {
    /// Parse a Notion API error code string into the typed vocabulary.
    #[allow(dead_code)]
    pub fn from_api_response(code: &str) -> Self {
        match code {
            "rate_limited" => Self::RateLimited,
            "object_not_found" => Self::ObjectNotFound,
            "unauthorized" => Self::Unauthorized,
            "restricted_resource" => Self::RestrictedResource,
            "invalid_json" => Self::InvalidJson,
            "validation_error" => Self::ValidationFailed,
            "conflict_error" => Self::Conflict,
            "internal_server_error" => Self::InternalError,
            "service_unavailable" => Self::ServiceUnavailable,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Create from an HTTP status code when the error body is unparseable.
    pub fn from_http_status(status: u16) -> Self {
        Self::HttpStatus(status)
    }

    /// Whether this error is transient and worth retrying.
    #[allow(dead_code)]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::ServiceUnavailable | Self::InternalError
        )
    }

    /// Whether this error means the resource simply doesn't exist.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::ObjectNotFound)
    }
}

impl fmt::Display for NotionErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited => write!(f, "rate_limited"),
            Self::ObjectNotFound => write!(f, "object_not_found"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::RestrictedResource => write!(f, "restricted_resource"),
            Self::InvalidJson => write!(f, "invalid_json"),
            Self::ValidationFailed => write!(f, "validation_error"),
            Self::Conflict => write!(f, "conflict_error"),
            Self::InternalError => write!(f, "internal_server_error"),
            Self::ServiceUnavailable => write!(f, "service_unavailable"),
            Self::HttpStatus(code) => write!(f, "http_{}", code),
            Self::Unknown(code) => write!(f, "{}", code),
        }
    }
}

/// Main application error type.
#[derive(Error, Debug)]
#[allow(dead_code)] // Some variants only constructed in bin target
pub enum AppError {
    #[error("Missing configuration: {0}")]
    MissingConfiguration(String),

    #[error("Invalid Notion ID format: {0}")]
    InvalidId(String),

    #[error("Network failure: {0}")]
    NetworkFailure(#[from] reqwest::Error),

    #[error("Notion API returned an error ({code}): {message}")]
    NotionService {
        code: NotionErrorCode,
        message: String,
        status: reqwest::StatusCode,
    },

    #[error("Malformed response: {0}")]
    MalformedResponse(String),

    #[error("Filesystem IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error interacting with clipboard: {0}")]
    Clipboard(String),

    #[error("Template file not found at {path}: {source}")]
    TemplateNotFound {
        path: String,
        source: std::io::Error,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Path error: {0}")]
    PathError(String),

    #[error("Failed to assemble object tree for root '{root_id}': {cause}")]
    AssemblyFailed { root_id: String, cause: String },

    #[error("Output delivery failed: {}", failures.join(", "))]
    DeliveryFailed { failures: Vec<String> },

    #[error("Internal error: {message}")]
    InternalError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Template render error for template {name}: {message}")]
    TemplateRenderError { name: String, message: String },

    #[error("JSON parse error for {path}: {source}")]
    JsonParseError {
        path: std::path::PathBuf,
        source: serde_json::Error,
    },

    #[error("Maximum recursion depth ({0}) exceeded")]
    RecursionLimitExceeded(usize),

    #[error(transparent)]
    ValidationError(#[from] crate::types::ValidationError),

    #[error(transparent)]
    NotionClient(#[from] NotionClientError),
}

// Allow converting from anyhow::Error, preserving error chain
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError {
            message: err.to_string(),
            source: None,
        }
    }
}

impl From<arboard::Error> for AppError {
    fn from(err: arboard::Error) -> Self {
        AppError::Clipboard(format!("Clipboard error: {}", err))
    }
}

impl From<std::fmt::Error> for AppError {
    fn from(err: std::fmt::Error) -> Self {
        AppError::InternalError {
            message: "Formatting error".to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::MalformedResponse(err.to_string())
    }
}

/// Notion client error mapping
#[derive(Error, Debug)]
#[allow(dead_code)] // Variants constructed via From<notion_client::NotionClientError>
pub enum NotionClientError {
    #[error("Failed to serialize request: {source}")]
    Serialization {
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to deserialize response: {source}\nBody: {body}")]
    Deserialization {
        #[source]
        source: serde_json::Error,
        body: String,
    },

    #[error("HTTP transport error: {message}")]
    Transport { message: String },

    #[error("Invalid authentication header: {message}")]
    InvalidHeader { message: String },

    #[error("Notion API error ({status}): {code} - {message}")]
    NotionApi {
        status: u32,
        code: String,
        message: String,
        request_id: Option<String>,
    },

    #[error("Type conversion error: {message}")]
    ConversionError { message: String },
}

// Convert notion_client errors to our error hierarchy
impl From<notion_client::NotionClientError> for NotionClientError {
    fn from(err: notion_client::NotionClientError) -> Self {
        use notion_client::NotionClientError as NcError;

        match err {
            NcError::FailedToSerialize { source } => Self::Serialization { source },
            NcError::FailedToDeserialize { source, body } => Self::Deserialization { source, body },
            NcError::FailedToRequest { source }
            | NcError::FailedToText { source }
            | NcError::FailedToBuildRequest { source } => Self::Transport {
                message: source.to_string(),
            },
            NcError::InvalidHeader { source } => Self::InvalidHeader {
                message: source.to_string(),
            },
            NcError::InvalidStatusCode { error } => Self::NotionApi {
                status: error.status,
                code: error.code,
                message: error.message,
                request_id: error.request_id,
            },
        }
    }
}

/// Domain vocabulary for why a database fetch failed.
///
/// This is not an error type — it's a classification of the failure reason,
/// enabling domain-specific handling (e.g., showing a clear message for
/// linked databases vs. a generic fallback for permission errors).
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseFetchFailure {
    /// The database is a linked database (Notion API limitation).
    LinkedDatabase,
    /// The integration lacks permission to access this database.
    PermissionDenied { reason: String },
    /// The database was not found.
    NotFound,
    /// Some other failure occurred.
    Other { cause: String },
}

impl std::fmt::Display for DatabaseFetchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LinkedDatabase => write!(
                f,
                "linked database (Notion API does not support retrieving linked databases)"
            ),
            Self::PermissionDenied { reason } => write!(f, "permission denied: {}", reason),
            Self::NotFound => write!(f, "database not found"),
            Self::Other { cause } => write!(f, "{}", cause),
        }
    }
}

/// Classifies a database fetch error into a domain-specific failure reason.
///
/// This is a pure function that examines the error structure to determine
/// whether the failure is due to a linked database (Notion API limitation),
/// a permission issue, or something else.
pub fn classify_database_fetch_failure(error: &AppError) -> DatabaseFetchFailure {
    match error {
        AppError::NotionClient(NotionClientError::NotionApi { message, code, .. }) => {
            classify_from_code_and_message(code, message)
        }
        AppError::NotionService { code, message, .. } => {
            if message.contains("linked database") {
                DatabaseFetchFailure::LinkedDatabase
            } else if code.is_not_found() {
                DatabaseFetchFailure::NotFound
            } else if matches!(
                code,
                NotionErrorCode::RestrictedResource | NotionErrorCode::Unauthorized
            ) {
                DatabaseFetchFailure::PermissionDenied {
                    reason: message.clone(),
                }
            } else {
                DatabaseFetchFailure::Other {
                    cause: error.to_string(),
                }
            }
        }
        _ => DatabaseFetchFailure::Other {
            cause: error.to_string(),
        },
    }
}

/// Classifies based on Notion API error code and message strings.
fn classify_from_code_and_message(code: &str, message: &str) -> DatabaseFetchFailure {
    if message.contains("linked database") {
        DatabaseFetchFailure::LinkedDatabase
    } else if code == "object_not_found" {
        DatabaseFetchFailure::NotFound
    } else if code == "restricted_resource" || code == "unauthorized" {
        DatabaseFetchFailure::PermissionDenied {
            reason: message.to_string(),
        }
    } else {
        DatabaseFetchFailure::Other {
            cause: format!("{}: {}", code, message),
        }
    }
}

/// Result type alias for convenience
#[allow(dead_code)]
pub type Result<T, E = AppError> = std::result::Result<T, E>;
