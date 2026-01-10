//! Error types for Hook0 MCP server

use rmcp::ErrorData;
use thiserror::Error;

/// Main error type for the Hook0 MCP server
#[derive(Debug, Error)]
pub enum Hook0McpError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// HTTP client error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API error with status code and message
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Invalid parameter error
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),
}

/// Alias for MCP error type
pub type McpError = ErrorData;

impl Hook0McpError {
    /// Convert to MCP error
    pub fn into_mcp_error(self) -> McpError {
        match self {
            Hook0McpError::Config(msg) => McpError::internal_error(msg, None),
            Hook0McpError::Http(e) => McpError::internal_error(e.to_string(), None),
            Hook0McpError::Api { status, message } => {
                if status == 404 {
                    McpError::resource_not_found(message, None)
                } else if status == 401 || status == 403 {
                    McpError::internal_error(format!("Authentication failed: {}", message), None)
                } else {
                    McpError::internal_error(format!("API error ({}): {}", status, message), None)
                }
            }
            Hook0McpError::Json(e) => McpError::invalid_params(e.to_string(), None),
            Hook0McpError::Url(e) => McpError::invalid_params(e.to_string(), None),
            Hook0McpError::InvalidParam(msg) => McpError::invalid_params(msg, None),
            Hook0McpError::NotFound(msg) => McpError::resource_not_found(msg, None),
            Hook0McpError::Auth(msg) => McpError::internal_error(format!("Auth: {}", msg), None),
        }
    }
}

impl From<Hook0McpError> for McpError {
    fn from(err: Hook0McpError) -> Self {
        err.into_mcp_error()
    }
}

/// Extension trait for MCP error construction
pub trait McpErrorExt {
    /// Create a tool not found error
    fn tool_not_found(name: impl Into<String>) -> Self;

    /// Create an invalid params error
    fn invalid_params(message: impl Into<String>) -> Self;

    /// Create a resource not found error
    fn resource_not_found(uri: impl Into<String>) -> Self;

    /// Create an internal error
    fn internal_error(message: impl Into<String>) -> Self;
}

impl McpErrorExt for McpError {
    fn tool_not_found(name: impl Into<String>) -> Self {
        // method_not_found takes no args, so we use invalid_params with a descriptive message
        ErrorData::invalid_params(format!("Tool not found: {}", name.into()), None)
    }

    fn invalid_params(message: impl Into<String>) -> Self {
        ErrorData::invalid_params(message.into(), None)
    }

    fn resource_not_found(uri: impl Into<String>) -> Self {
        ErrorData::resource_not_found(format!("Resource not found: {}", uri.into()), None)
    }

    fn internal_error(message: impl Into<String>) -> Self {
        ErrorData::internal_error(message.into(), None)
    }
}
