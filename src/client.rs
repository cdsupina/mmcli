use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs::{home_dir, config_dir};
use tokio::fs;
use native_tls::{Identity, TlsConnector};
use std::fs as std_fs;
use std::io::{self, Write};

const BASE_URL: &str = "https://api.mcmaster.com";

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    #[serde(rename = "UserName")]
    pub user_name: String,
    #[serde(rename = "Password")]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "AuthToken")]
    pub token: String,
    #[serde(rename = "ExpirationTS")]
    #[allow(dead_code)]
    pub expiration: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub certificate_path: Option<String>,
    pub certificate_password: Option<String>,
}

pub struct McmasterClient {
    client: Client,
    token: Option<String>,
    credentials: Option<Credentials>,
}

impl McmasterClient {
    pub fn new_with_credentials(credentials: Option<Credentials>) -> Result<Self> {
        let mut client_builder = Client::builder();

        // Try to find and load certificate
        if let Some(ref creds) = credentials {
            let cert_path = if let Some(ref explicit_path) = creds.certificate_path {
                // Use explicitly specified path
                Some(PathBuf::from(explicit_path))
            } else {
                // Try to find certificate in default locations
                Self::find_default_certificate()
            };

            if let Some(cert_path) = cert_path {
                if cert_path.exists() {
                    println!("Loading client certificate: {}", cert_path.display());
                    
                    // Read the PKCS12 file
                    let cert_data = std_fs::read(&cert_path)
                        .context("Failed to read certificate file")?;
                    
                    // Get certificate password
                    let cert_password = creds.certificate_password
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    
                    // Create identity from PKCS12
                    let identity = Identity::from_pkcs12(&cert_data, cert_password)
                        .context("Failed to create identity from PKCS12 certificate")?;
                    
                    // Create TLS connector with the identity
                    let tls_connector = TlsConnector::builder()
                        .identity(identity)
                        .danger_accept_invalid_certs(true)  // API endpoints sometimes need this
                        .danger_accept_invalid_hostnames(true)
                        .build()
                        .context("Failed to build TLS connector")?;
                    
                    client_builder = client_builder.use_preconfigured_tls(tls_connector);
                    println!("Client certificate loaded successfully");
                } else if creds.certificate_path.is_some() {
                    // Explicit path was provided but file doesn't exist
                    return Err(anyhow::anyhow!("Certificate file not found: {}", cert_path.display()));
                } else {
                    // No explicit path and no default certificate found
                    println!("No client certificate found in default locations:");
                    for location in Self::get_default_cert_locations() {
                        println!("  - {}", location.display());
                    }
                    return Err(anyhow::anyhow!("No client certificate found. Place certificate in default location or specify certificate_path in credentials"));
                }
            }
        }

        let client = client_builder
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            token: None,
            credentials,
        })
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        let login_request = LoginRequest {
            user_name: username,
            password,
        };

        let response = self
            .client
            .post(&format!("{}/v1/login", BASE_URL))
            .json(&login_request)
            .send()
            .await
            .context("Failed to send login request")?;

        println!("Response status: {}", response.status());
        println!("Response headers: {:?}", response.headers());
        
        let response_text = response.text().await.context("Failed to get response text")?;
        println!("Response body: {}", response_text);

        if response_text.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty response from server"));
        }

        // Try to parse as success response first
        if let Ok(login_response) = serde_json::from_str::<LoginResponse>(&response_text) {
            self.token = Some(login_response.token.clone());
            self.save_token(&login_response.token).await?;
            println!("Login successful");
            return Ok(());
        }

        // Try to parse as error response
        if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
            return Err(anyhow::anyhow!(
                "Login failed: {} - {}",
                error.error_message.unwrap_or_default(),
                error.error_description.unwrap_or_default()
            ));
        }

        // If we can't parse as JSON, return the raw response
        return Err(anyhow::anyhow!(
            "Unexpected response format. Response: {}",
            response_text
        ));
    }

    pub async fn logout(&mut self) -> Result<()> {
        if let Some(token) = &self.token {
            let response = self
                .client
                .post(&format!("{}/v1/logout", BASE_URL))
                .bearer_auth(token)
                .send()
                .await
                .context("Failed to send logout request")?;

            if !response.status().is_success() {
                let error: ErrorResponse = response
                    .json()
                    .await
                    .context("Failed to parse error response")?;
                
                println!(
                    "Logout warning: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                );
            }
        }

        self.token = None;
        self.remove_token().await?;
        println!("Logged out");
        Ok(())
    }

    pub async fn load_token(&mut self) -> Result<()> {
        if let Some(token_path) = self.get_token_path() {
            if token_path.exists() {
                let token = fs::read_to_string(&token_path)
                    .await
                    .context("Failed to read token file")?;
                self.token = Some(token.trim().to_string());
            }
        }
        Ok(())
    }

    async fn save_token(&self, token: &str) -> Result<()> {
        if let Some(token_path) = self.get_token_path() {
            if let Some(parent) = token_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .context("Failed to create token directory")?;
            }
            
            fs::write(&token_path, token)
                .await
                .context("Failed to write token file")?;
        }
        Ok(())
    }

    async fn remove_token(&self) -> Result<()> {
        if let Some(token_path) = self.get_token_path() {
            if token_path.exists() {
                fs::remove_file(&token_path)
                    .await
                    .context("Failed to remove token file")?;
            }
        }
        Ok(())
    }

    fn get_token_path(&self) -> Option<PathBuf> {
        // Use XDG config directory first
        if let Some(config_dir) = config_dir() {
            let mut path = config_dir;
            path.push("mmc");
            path.push("token");
            return Some(path);
        }
        
        // Fallback to legacy location
        home_dir().map(|mut path| {
            path.push(".mmcli");
            path.push("token");
            path
        })
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub async fn add_product(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products", BASE_URL);
        let request_body = serde_json::json!({
            "URL": format!("https://mcmaster.com/{}", product)
        });
        
        let response = self
            .client
            .put(&url)  // PUT instead of POST
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request_body)
            .send()
            .await
            .context("Failed to add product")?;

        println!("Add product response status: {}", response.status());
        let response_text = response.text().await.context("Failed to get response text")?;
        println!("Add product response: {}", response_text);

        // Try to parse as JSON to see if it's product data or error
        if let Ok(product_data) = serde_json::from_str::<serde_json::Value>(&response_text) {
            println!("Product {} added to subscription", product);
            println!("{}", serde_json::to_string_pretty(&product_data)?);
        } else {
            return Err(anyhow::anyhow!(
                "Unexpected response format for add product: {}",
                response_text
            ));
        }

        Ok(())
    }

    pub async fn remove_product(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products", BASE_URL);
        let response = self
            .client
            .delete(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&serde_json::json!({ "ProductNumber": product }))
            .send()
            .await
            .context("Failed to remove product")?;

        if response.status().is_success() {
            println!("Product {} removed from subscription", product);
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .context("Failed to parse error response")?;
            
            return Err(anyhow::anyhow!(
                "Failed to remove product: {} - {}",
                error.error_message.unwrap_or_default(),
                error.error_description.unwrap_or_default()
            ));
        }

        Ok(())
    }

    pub async fn get_product(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products/{}", BASE_URL, product);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get product")?;

        if response.status().is_success() {
            let product_data: serde_json::Value = response
                .json()
                .await
                .context("Failed to parse product response")?;
            
            println!("{}", serde_json::to_string_pretty(&product_data)?);
        } else if response.status().as_u16() == 403 {
            // Product is not in subscription - offer to add it
            println!("❌ Product {} is not in your subscription.", product);
            print!("Would you like to add it to your subscription? (Y/n): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input == "y" || input == "yes" || input.is_empty() {
                println!("Adding product {} to subscription...", product);
                self.add_product(product).await?;
                println!("✅ Product added! Getting product information...");
                
                // Retry the product request after adding to subscription
                let url = format!("{}/v1/products/{}", BASE_URL, product);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(self.token.as_ref().unwrap())
                    .send()
                    .await
                    .context("Failed to get product after adding to subscription")?;
                
                if response.status().is_success() {
                    let product_data: serde_json::Value = response
                        .json()
                        .await
                        .context("Failed to parse product response")?;
                    
                    println!("{}", serde_json::to_string_pretty(&product_data)?);
                    return Ok(());
                } else {
                    let status = response.status();
                    let response_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Failed to get product after adding to subscription. Status: {}. Response: {}",
                        status, response_text
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Product {} is not in your subscription. Add it first with: mmc add {}",
                    product, product
                ));
            }
        } else {
            // Try to parse as JSON error response, but handle parsing failures gracefully
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get product: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to get product. Status: {}. Response: {}",
                    status,
                    response_text
                ));
            }
        }

        Ok(())
    }

    pub async fn get_price(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products/{}/price", BASE_URL, product);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get price")?;

        if response.status().is_success() {
            let price_data: serde_json::Value = response
                .json()
                .await
                .context("Failed to parse price response")?;
            
            println!("{}", serde_json::to_string_pretty(&price_data)?);
        } else if response.status().as_u16() == 403 {
            // Product is not in subscription - offer to add it
            println!("❌ Product {} is not in your subscription.", product);
            print!("Would you like to add it to your subscription? (Y/n): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input == "y" || input == "yes" || input.is_empty() {
                println!("Adding product {} to subscription...", product);
                self.add_product(product).await?;
                println!("✅ Product added! Getting price information...");
                
                // Retry the price request after adding to subscription
                let url = format!("{}/v1/products/{}/price", BASE_URL, product);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(self.token.as_ref().unwrap())
                    .send()
                    .await
                    .context("Failed to get price after adding to subscription")?;
                
                if response.status().is_success() {
                    let price_data: serde_json::Value = response
                        .json()
                        .await
                        .context("Failed to parse price response")?;
                    
                    println!("{}", serde_json::to_string_pretty(&price_data)?);
                    return Ok(());
                } else {
                    let status = response.status();
                    let response_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Failed to get price after adding to subscription. Status: {}. Response: {}",
                        status, response_text
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Product {} is not in your subscription. Add it first with: mmc add {}",
                    product, product
                ));
            }
        } else {
            // Try to parse as JSON error response, but handle parsing failures gracefully
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get price: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to get price. Status: {}. Response: {}",
                    status,
                    response_text
                ));
            }
        }

        Ok(())
    }

    pub async fn get_changes(&self, start_date: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/changes?start={}", BASE_URL, urlencoding::encode(start_date));
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get changes")?;

        println!("Changes response status: {}", response.status());
        let response_text = response.text().await.context("Failed to get response text")?;
        println!("Changes response: {}", response_text);

        // Try to parse as JSON
        if let Ok(changes_data) = serde_json::from_str::<serde_json::Value>(&response_text) {
            println!("{}", serde_json::to_string_pretty(&changes_data)?);
        } else {
            return Err(anyhow::anyhow!(
                "Unexpected response format for changes: {}",
                response_text
            ));
        }

        Ok(())
    }

    fn ensure_authenticated(&self) -> Result<()> {
        if !self.is_authenticated() {
            return Err(anyhow::anyhow!(
                "Not authenticated. Please login first with: mmcli login -u <username> -p <password>"
            ));
        }
        Ok(())
    }


    pub async fn login_with_stored_credentials(&mut self) -> Result<()> {
        let credentials = self.credentials.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No credentials loaded. Please login first."))?;

        self.login(credentials.username.clone(), credentials.password.clone()).await
    }

    pub async fn save_credentials_template(&self, path: &str) -> Result<()> {
        let template = if path.ends_with(".json") {
            // For JSON, we can't easily add comments, so include optional field
            serde_json::to_string_pretty(&Credentials {
                username: "your_username".to_string(),
                password: "your_password".to_string(),
                certificate_path: Some("~/.config/mmc/certificate.pfx".to_string()),
                certificate_password: Some("certificate_password".to_string()),
            })?
        } else if path.ends_with(".toml") {
            // For TOML, we can add comments explaining the defaults
            format!(
r#"username = "your_username"
password = "your_password"

# Certificate settings (optional - will auto-discover if not specified)
# Default locations checked:
#   ~/.config/mmc/certificate.pfx
#   ~/.config/mmc/certificate.p12  
#   ~/.mmcli/certificate.pfx (legacy)
#   ~/.mmcli/certificate.p12 (legacy)
certificate_path = "~/.config/mmc/certificate.pfx"
certificate_password = "certificate_password"
"#)
        } else {
            return Err(anyhow::anyhow!("Unsupported file format. Use .json or .toml"));
        };

        fs::write(path, template)
            .await
            .context("Failed to write credentials template")?;

        println!("Credentials template saved to: {}", path);
        println!("Please edit the file with your actual credentials.");
        println!("Certificate will be auto-discovered from ~/.config/mmc/certificate.pfx if certificate_path is not specified.");
        Ok(())
    }


    fn get_default_cert_locations() -> Vec<PathBuf> {
        let mut locations = Vec::new();
        
        // XDG config directory locations (preferred)
        if let Some(config_dir) = config_dir() {
            let mut cert_path = config_dir;
            cert_path.push("mmc");
            
            // Try different common extensions
            for ext in &["pfx", "p12"] {
                let mut path = cert_path.clone();
                path.push(format!("certificate.{}", ext));
                locations.push(path);
            }
        }
        
        // Legacy locations (fallback)
        if let Some(home) = home_dir() {
            let mut legacy_path = home;
            legacy_path.push(".mmcli");
            
            for ext in &["pfx", "p12"] {
                let mut path = legacy_path.clone();
                path.push(format!("certificate.{}", ext));
                locations.push(path);
            }
        }
        
        locations
    }

    fn find_default_certificate() -> Option<PathBuf> {
        for location in Self::get_default_cert_locations() {
            if location.exists() {
                println!("Found certificate at: {}", location.display());
                return Some(location);
            }
        }
        None
    }
}