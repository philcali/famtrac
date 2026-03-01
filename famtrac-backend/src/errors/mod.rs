#[cfg(test)]
mod tests;
mod validation;

pub use validation::{
    sanitize_string, validate_activity_timestamp, validate_activity_type, validate_date_of_birth,
    validate_dependent_name, validate_family_name,
};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error types for the famtrac-backend API
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerError {
    Validation(ValidationError),
    Store(StoreError),
    Auth(AuthError),
    NotFound(String),
    InternalError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    ConnectionError(String),
    QueryError(String),
    NotFound(String),
    ConflictError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub constraint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    Unauthorized(String),
    Forbidden(String),
    MissingIdentity,
}

/// JSON error response structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl HandlerError {
    /// Map error to HTTP status code
    pub fn status_code(&self) -> u16 {
        match self {
            HandlerError::Validation(_) => 400,
            HandlerError::Store(StoreError::NotFound(_)) => 404,
            HandlerError::Store(_) => 500,
            HandlerError::Auth(AuthError::Unauthorized(_)) => 401,
            HandlerError::Auth(AuthError::MissingIdentity) => 401,
            HandlerError::Auth(AuthError::Forbidden(_)) => 403,
            HandlerError::NotFound(_) => 404,
            HandlerError::InternalError(_) => 500,
        }
    }

    /// Convert error to JSON error response
    pub fn to_error_response(&self) -> ErrorResponse {
        match self {
            HandlerError::Validation(err) => {
                let details = if let Some(constraint) = &err.constraint {
                    Some(serde_json::json!({
                        "field": err.field,
                        "constraint": constraint,
                    }))
                } else {
                    Some(serde_json::json!({
                        "field": err.field,
                    }))
                };

                ErrorResponse {
                    error: ErrorDetail {
                        code: "VALIDATION_ERROR".to_string(),
                        message: err.message.clone(),
                        details,
                    },
                }
            }
            HandlerError::Store(StoreError::NotFound(msg)) => ErrorResponse {
                error: ErrorDetail {
                    code: "NOT_FOUND".to_string(),
                    message: msg.clone(),
                    details: None,
                },
            },
            HandlerError::Store(_) => ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: "An internal error occurred".to_string(),
                    details: None,
                },
            },
            HandlerError::Auth(AuthError::Unauthorized(msg)) => ErrorResponse {
                error: ErrorDetail {
                    code: "UNAUTHORIZED".to_string(),
                    message: msg.clone(),
                    details: None,
                },
            },
            HandlerError::Auth(AuthError::MissingIdentity) => ErrorResponse {
                error: ErrorDetail {
                    code: "UNAUTHORIZED".to_string(),
                    message: "Missing authentication credentials".to_string(),
                    details: None,
                },
            },
            HandlerError::Auth(AuthError::Forbidden(msg)) => ErrorResponse {
                error: ErrorDetail {
                    code: "FORBIDDEN".to_string(),
                    message: msg.clone(),
                    details: None,
                },
            },
            HandlerError::NotFound(msg) => ErrorResponse {
                error: ErrorDetail {
                    code: "NOT_FOUND".to_string(),
                    message: msg.clone(),
                    details: None,
                },
            },
            HandlerError::InternalError(_) => ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: "An internal error occurred".to_string(),
                    details: None,
                },
            },
        }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandlerError::Validation(err) => write!(f, "Validation error: {}", err.message),
            HandlerError::Store(err) => write!(f, "Store error: {}", err),
            HandlerError::Auth(err) => write!(f, "Auth error: {}", err),
            HandlerError::NotFound(msg) => write!(f, "Not found: {}", msg),
            HandlerError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            StoreError::QueryError(msg) => write!(f, "Query error: {}", msg),
            StoreError::NotFound(msg) => write!(f, "Not found: {}", msg),
            StoreError::ConflictError(msg) => write!(f, "Conflict error: {}", msg),
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AuthError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AuthError::MissingIdentity => write!(f, "Missing identity"),
        }
    }
}

impl std::error::Error for HandlerError {}
impl std::error::Error for StoreError {}
impl std::error::Error for AuthError {}

impl From<ValidationError> for HandlerError {
    fn from(err: ValidationError) -> Self {
        HandlerError::Validation(err)
    }
}

impl From<StoreError> for HandlerError {
    fn from(err: StoreError) -> Self {
        HandlerError::Store(err)
    }
}

impl From<AuthError> for HandlerError {
    fn from(err: AuthError) -> Self {
        HandlerError::Auth(err)
    }
}
