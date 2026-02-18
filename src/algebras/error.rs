//! Error types for algebra traits.
//!
//! These errors are used by the algebra layer and are intentionally
//! domain-specific rather than generic. Each error tells a story about
//! what went wrong in the algebra's operation.

use std::fmt;

/// Error that can occur during content retrieval operations.
///
/// This is the error type for [`NotionContent`] algebra operations.
/// It represents failure modes that any content retrieval implementation
/// might encounter.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchError {
    /// The requested object was not found (404).
    NotFound { id: String },

    /// Authentication failed — invalid or missing API key.
    Unauthorized { reason: String },

    /// The client lacks permission to access this resource.
    RestrictedResource { reason: String },

    /// The API rate limit was exceeded.
    RateLimited { retry_after_seconds: Option<u64> },

    /// The request was malformed or invalid.
    InvalidRequest { reason: String },

    /// Notion API returned an error.
    ApiError {
        code: String,
        message: String,
        status: u16,
    },

    /// Network or transport error.
    Transport { message: String },

    /// The response could not be parsed.
    MalformedResponse { reason: String },

    /// An operation timed out.
    Timeout { operation: String },

    /// Some other error occurred.
    Other { message: String },
}

impl FetchError {
    /// Returns `true` if this error is transient and worth retrying.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. }
                | Self::Timeout { .. }
                | Self::ApiError {
                    status: 408 | 429 | 500..=599,
                    ..
                }
        )
    }

    /// Returns `true` if this error means the resource doesn't exist.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { id } => write!(f, "Object not found: {}", id),
            Self::Unauthorized { reason } => write!(f, "Unauthorized: {}", reason),
            Self::RestrictedResource { reason } => write!(f, "Restricted resource: {}", reason),
            Self::RateLimited {
                retry_after_seconds,
            } => {
                write!(f, "Rate limited")?;
                if let Some(seconds) = retry_after_seconds {
                    write!(f, " (retry after {}s)", seconds)?;
                }
                Ok(())
            }
            Self::InvalidRequest { reason } => write!(f, "Invalid request: {}", reason),
            Self::ApiError { code, message, .. } => {
                write!(f, "API error [{}]: {}", code, message)
            }
            Self::Transport { message } => write!(f, "Transport error: {}", message),
            Self::MalformedResponse { reason } => write!(f, "Malformed response: {}", reason),
            Self::Timeout { operation } => write!(f, "Timeout during: {}", operation),
            Self::Other { message } => write!(f, "Error: {}", message),
        }
    }
}

impl std::error::Error for FetchError {}

/// Error that can occur during visit tracking operations.
///
/// This is the error type for [`VisitTracker`] algebra operations.
/// Visit tracking is a simple operation, so errors are minimal.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackError {
    /// The tracker is full and cannot accept more visits.
    CapacityExceeded { max: usize },

    /// An operation failed for some other reason.
    Other { message: String },
}

impl fmt::Display for TrackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded { max } => {
                write!(f, "Visit tracker capacity exceeded (max: {})", max)
            }
            Self::Other { message } => write!(f, "Track error: {}", message),
        }
    }
}

impl std::error::Error for TrackError {}

// ==============================================================================
// Conversion from existing errors
// ==============================================================================

impl From<crate::error::AppError> for FetchError {
    fn from(err: crate::error::AppError) -> Self {
        use crate::error::{AppError, NotionClientError, NotionErrorCode};

        match err {
            // Note: `ref` bindings are required here so `err` remains available
            // for `err.to_string()` in the wildcard catch-all sub-arm.
            AppError::NotionService {
                ref code,
                ref message,
                ..
            } => match code {
                NotionErrorCode::ObjectNotFound => Self::NotFound {
                    id: message.clone(),
                },
                NotionErrorCode::Unauthorized | NotionErrorCode::RestrictedResource => {
                    Self::RestrictedResource {
                        reason: message.clone(),
                    }
                }
                NotionErrorCode::RateLimited => Self::RateLimited {
                    retry_after_seconds: None,
                },
                NotionErrorCode::InvalidJson | NotionErrorCode::ValidationFailed => {
                    Self::InvalidRequest {
                        reason: message.clone(),
                    }
                }
                NotionErrorCode::InternalError | NotionErrorCode::ServiceUnavailable => {
                    Self::ApiError {
                        code: code.to_string(),
                        message: message.clone(),
                        status: 500,
                    }
                }
                _ => Self::Other {
                    message: err.to_string(),
                },
            },
            // This arm fully destructures by move — all sub-arms consume the
            // fields directly, so `ref` is not needed.
            AppError::NotionClient(NotionClientError::NotionApi {
                code,
                message,
                status,
                ..
            }) => match code.as_str() {
                "object_not_found" => Self::NotFound { id: message },
                "unauthorized" | "restricted_resource" => {
                    Self::RestrictedResource { reason: message }
                }
                "rate_limited" => Self::RateLimited {
                    retry_after_seconds: None,
                },
                "validation_error" => Self::InvalidRequest { reason: message },
                _ => Self::ApiError {
                    code,
                    message,
                    status: status as u16,
                },
            },
            AppError::NetworkFailure(_) => Self::Transport {
                message: err.to_string(),
            },
            AppError::MalformedResponse(_) => Self::MalformedResponse {
                reason: err.to_string(),
            },
            _ => Self::Other {
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_error_is_retryable() {
        assert!(FetchError::RateLimited {
            retry_after_seconds: None
        }
        .is_retryable());
        assert!(FetchError::Timeout {
            operation: "fetch".to_string()
        }
        .is_retryable());
        assert!(!FetchError::NotFound {
            id: "test".to_string()
        }
        .is_retryable());
        assert!(!FetchError::Unauthorized {
            reason: "bad key".to_string()
        }
        .is_retryable());
    }

    #[test]
    fn test_fetch_error_is_not_found() {
        assert!(FetchError::NotFound {
            id: "test".to_string()
        }
        .is_not_found());
        assert!(!FetchError::Unauthorized {
            reason: "bad key".to_string()
        }
        .is_not_found());
    }

    #[test]
    fn test_fetch_error_display() {
        let err = FetchError::NotFound {
            id: "abc123".to_string(),
        };
        assert_eq!(err.to_string(), "Object not found: abc123");

        let err = FetchError::RateLimited {
            retry_after_seconds: Some(60),
        };
        assert_eq!(err.to_string(), "Rate limited (retry after 60s)");
    }
}
