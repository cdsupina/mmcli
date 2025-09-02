//! Error handling utilities

use std::error::Error;
use std::fmt;

/// Custom error type for McMaster-Carr CLI operations
#[derive(Debug)]
pub enum ClientError {
    /// HTTP request failed
    Request(reqwest::Error),
    /// JSON serialization/deserialization failed
    Json(serde_json::Error),
    /// File I/O error
    Io(std::io::Error),
    /// Authentication error
    Auth(String),
    /// API error with message
    Api(String),
    /// Configuration error
    Config(String),
    /// Generic error with message
    Generic(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Request(e) => write!(f, "Request error: {}", e),
            ClientError::Json(e) => write!(f, "JSON error: {}", e),
            ClientError::Io(e) => write!(f, "I/O error: {}", e),
            ClientError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            ClientError::Api(msg) => write!(f, "API error: {}", msg),
            ClientError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ClientError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for ClientError {}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        ClientError::Request(err)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> Self {
        ClientError::Json(err)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::Io(err)
    }
}