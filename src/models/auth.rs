//! Authentication models

use serde::{Deserialize, Serialize};

/// Login request payload
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    #[serde(rename = "UserName")]
    pub user_name: String,
    #[serde(rename = "Password")]
    pub password: String,
}

/// Login response from the API
#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "AuthToken")]
    pub token: String,
    #[serde(rename = "ExpirationTS")]
    #[allow(dead_code)]
    pub expiration: Option<String>,
}

/// Error response from the API
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "ErrorCode")]
    #[allow(dead_code)]
    pub error_code: Option<String>,
    #[serde(rename = "ErrorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "ErrorDescription")]
    pub error_description: Option<String>,
}

/// User credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub certificate_path: Option<String>,
    pub certificate_password: Option<String>,
    pub subscriptions_file: Option<String>,
}