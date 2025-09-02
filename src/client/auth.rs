//! Authentication functionality for McMaster-Carr API

use anyhow::Result;
use reqwest::Client;
use std::path::PathBuf;
use std::fs;
use tokio::fs as async_fs;

use crate::config::paths::{get_config_dir, get_token_path, find_certificate_path, expand_path};
use crate::models::auth::{Credentials, LoginRequest, LoginResponse, ErrorResponse};
use crate::utils::output::OutputFormat;

/// Authentication-related methods for McmasterClient
impl super::api::McmasterClient {
    /// Authenticate with username and password
    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        let login_request = LoginRequest {
            user_name: username,
            password,
        };

        let response = self.client.post("https://api.mcmaster.com/v1/login")
            .json(&login_request)
            .send()
            .await?;

        if response.status().is_success() {
            let login_response: LoginResponse = response.json().await?;
            self.token = Some(login_response.token.clone());

            // Save token to file for future use
            if let Err(e) = self.save_token().await {
                if !self.quiet_mode {
                    eprintln!("⚠️  Warning: Could not save token: {}", e);
                }
            }

            if !self.quiet_mode {
                println!("✅ Login successful");
            }
        } else {
            // Try to parse as error response
            let error_text = response.text().await?;
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "Login failed: {}",
                    error_response.error_message.unwrap_or("Unknown error".to_string())
                ));
            } else {
                return Err(anyhow::anyhow!("Login failed: {}", error_text));
            }
        }

        Ok(())
    }

    /// Logout and invalidate current token
    pub async fn logout(&mut self) -> Result<()> {
        if let Some(token) = &self.token {
            let response = self.client.delete("https://api.mcmaster.com/v1/logout")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            if response.status().is_success() {
                self.token = None;
                // Remove token file
                let token_path = get_token_path();
                if token_path.exists() {
                    let _ = async_fs::remove_file(token_path).await;
                }
                if !self.quiet_mode {
                    println!("✅ Logout successful");
                }
            } else {
                if !self.quiet_mode {
                    eprintln!("⚠️  Warning: Logout request failed, but clearing local token");
                }
                self.token = None;
                let token_path = get_token_path();
                if token_path.exists() {
                    let _ = async_fs::remove_file(token_path).await;
                }
            }
        } else {
            if !self.quiet_mode {
                println!("ℹ️  No active session to logout from");
            }
        }

        Ok(())
    }

    /// Load stored token from file
    pub async fn load_token(&mut self) -> Result<()> {
        let token_path = get_token_path();
        
        if token_path.exists() {
            let token = async_fs::read_to_string(token_path).await?;
            self.token = Some(token.trim().to_string());
            if !self.quiet_mode {
                println!("🔑 Loaded existing authentication token");
            }
        } else {
            if !self.quiet_mode {
                println!("ℹ️  No existing token found");
            }
        }

        Ok(())
    }

    /// Save current token to file
    async fn save_token(&self) -> Result<()> {
        if let Some(token) = &self.token {
            let config_dir = get_config_dir();
            
            // Create config directory if it doesn't exist
            if !config_dir.exists() {
                async_fs::create_dir_all(&config_dir).await?;
            }

            let token_path = get_token_path();
            async_fs::write(token_path, token).await?;
        }
        
        Ok(())
    }

    /// Login with stored credentials if available
    pub async fn login_with_stored_credentials(&mut self) -> Result<()> {
        if let Some(ref credentials) = self.credentials.clone() {
            self.login(credentials.username.clone(), credentials.password.clone()).await
        } else {
            Err(anyhow::anyhow!("No credentials available"))
        }
    }

    /// Find certificate in default locations (quiet version)
    pub fn find_default_certificate_quiet(quiet: bool) -> Option<PathBuf> {
        let cert_path = find_certificate_path();
        
        if let Some(ref path) = cert_path {
            if !quiet {
                println!("Found certificate at: {}", path.display());
            }
        }
        
        cert_path
    }

    /// Find certificate in default locations (with output)
    pub fn find_default_certificate() -> Option<PathBuf> {
        Self::find_default_certificate_quiet(false)
    }

    /// Set quiet mode (suppress non-essential output)
    pub fn set_quiet_mode(&mut self, quiet: bool) {
        self.quiet_mode = quiet;
    }

    /// Save credentials template to file
    pub async fn save_credentials_template(&self, path: &str) -> Result<()> {
        let credentials_path = expand_path(path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = credentials_path.parent() {
            if !parent.exists() {
                async_fs::create_dir_all(parent).await?;
            }
        }

        let template = if path.ends_with(".json") {
            // JSON template
            serde_json::to_string_pretty(&serde_json::json!({
                "username": "your@email.com",
                "password": "your_password",
                "certificate_path": "~/.config/mmc/certificate.pfx",
                "certificate_password": "certificate_password"
            }))?
        } else {
            // TOML template
            r#"username = "your@email.com"
password = "your_password"

# Certificate settings (optional - will auto-discover if not specified)
# Default locations checked:
#   ~/.config/mmc/certificate.pfx
#   ~/.config/mmc/certificate.p12  
#   ~/.mmcli/certificate.pfx (legacy)
#   ~/.mmcli/certificate.p12 (legacy)
certificate_path = "~/.config/mmc/certificate.pfx"
certificate_password = "certificate_password"
"#.to_string()
        };

        async_fs::write(&credentials_path, template).await?;
        println!("✅ Credentials template saved to: {}", credentials_path.display());
        
        Ok(())
    }
}