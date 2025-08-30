use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};
use dirs::{home_dir, config_dir};
use tokio::fs;
use tokio::io::AsyncWriteExt;
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

#[derive(Debug, Deserialize)]
pub struct LinkItem {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ProductResponse {
    #[serde(rename = "Links")]
    pub links: Option<Vec<LinkItem>>,
}

#[derive(Debug, Clone)]
pub struct CadFile {
    pub format: CadFormat,
    pub url: String,
    pub key: String, // Original API key like "2-D DWG", "3-D STEP"
}

#[derive(Debug, Clone, PartialEq)]
pub enum CadFormat {
    Dwg,
    Step,
    Dxf,
    Iges,
    Solidworks,
    Sat,
    Edrw,
    Pdf,
}

impl CadFormat {
    fn from_api_key(key: &str) -> Option<Self> {
        match key {
            k if k.contains("DWG") => Some(CadFormat::Dwg),
            k if k.contains("STEP") => Some(CadFormat::Step),
            k if k.contains("DXF") => Some(CadFormat::Dxf),
            k if k.contains("IGES") => Some(CadFormat::Iges),
            k if k.contains("SLDPRT") || k.contains("SLDDRW") || k.contains("Solidworks") => Some(CadFormat::Solidworks),
            k if k.contains("SAT") => Some(CadFormat::Sat),
            k if k.contains("EDRW") => Some(CadFormat::Edrw),
            k if k.contains("PDF") => Some(CadFormat::Pdf),
            _ => None,
        }
    }
    
    fn matches_filter(&self, filter: &str) -> bool {
        match filter {
            "dwg" => matches!(self, CadFormat::Dwg),
            "step" => matches!(self, CadFormat::Step),
            "dxf" => matches!(self, CadFormat::Dxf),
            "iges" => matches!(self, CadFormat::Iges),
            "solidworks" => matches!(self, CadFormat::Solidworks),
            "sat" => matches!(self, CadFormat::Sat),
            "edrw" => matches!(self, CadFormat::Edrw),
            "pdf" => matches!(self, CadFormat::Pdf),
            _ => false,
        }
    }
}

pub struct ProductLinks {
    pub images: Vec<String>,
    pub cad: Vec<CadFile>,
    pub datasheets: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProductInfo {
    #[serde(rename = "PartNumber")]
    pub part_number: Option<String>,
    #[serde(rename = "DetailDescription")]
    pub detail_description: Option<String>,
    #[serde(rename = "FamilyDescription")]
    pub family_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PriceInfo {
    #[serde(rename = "Amount")]
    pub amount: f64,
    #[serde(rename = "MinimumQuantity")]
    pub minimum_quantity: f64,
    #[serde(rename = "UnitOfMeasure")]
    pub unit_of_measure: String,
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

        if response.status().is_success() {
            let response_text = response.text().await.context("Failed to get response text")?;
            
            // Try to parse product info for clean display
            if let Ok(product_info) = serde_json::from_str::<ProductInfo>(&response_text) {
                println!("✅ Added {} to subscription", product);
                
                // Build description line
                let mut description_parts = Vec::new();
                if let Some(detail) = &product_info.detail_description {
                    description_parts.push(detail.as_str());
                }
                if let Some(family) = &product_info.family_description {
                    description_parts.push(family.as_str());
                }
                
                if !description_parts.is_empty() {
                    println!("   {}", description_parts.join(" - "));
                }
            } else {
                // Fallback if we can't parse the response
                println!("✅ Product {} added to subscription", product);
            }
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to add product {}. Status: {}. Response: {}",
                product, status, error_text
            ));
        }

        Ok(())
    }

    pub async fn remove_product(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products", BASE_URL);
        let request_body = serde_json::json!({
            "URL": format!("https://mcmaster.com/{}", product)
        });
        
        let response = self
            .client
            .delete(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request_body)
            .send()
            .await
            .context("Failed to remove product")?;

        if response.status().is_success() {
            println!("✅ Removed {} from subscription", product);
        } else {
            // Try to parse as JSON error response, but handle parsing failures gracefully
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(anyhow::anyhow!(
                    "Failed to remove product: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to remove product {}. Status: {}. Response: {}",
                    product, status, response_text
                ));
            }
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
            let price_data: Vec<PriceInfo> = response
                .json()
                .await
                .context("Failed to parse price response")?;
            
            self.format_price_output(product, &price_data);
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
                    let price_data: Vec<PriceInfo> = response
                        .json()
                        .await
                        .context("Failed to parse price response")?;
                    
                    self.format_price_output(product, &price_data);
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

    // Helper method to get product links
    async fn get_product_links(&self, product: &str) -> Result<ProductLinks> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products/{}", BASE_URL, product);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get product information")?;

        if response.status().is_success() {
            let product_data: ProductResponse = response
                .json()
                .await
                .context("Failed to parse product response")?;
            
            if let Some(link_items) = product_data.links {
                let mut images = Vec::new();
                let mut cad = Vec::new();
                let mut datasheets = Vec::new();
                
                for link in link_items {
                    match link.key.as_str() {
                        "Image" => images.push(link.value),
                        key if CadFormat::from_api_key(key).is_some() => {
                            if let Some(format) = CadFormat::from_api_key(key) {
                                cad.push(CadFile {
                                    format,
                                    url: link.value,
                                    key: link.key,
                                });
                            }
                        },
                        "Datasheet" | "Data Sheet" => datasheets.push(link.value),
                        _ => {} // Ignore other link types like "Price", "ProductDetail"
                    }
                }
                
                Ok(ProductLinks {
                    images,
                    cad,
                    datasheets,
                })
            } else {
                Err(anyhow::anyhow!("No asset links found for product {}", product))
            }
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
                println!("✅ Product added! Getting asset links...");
                
                // Retry the request after adding to subscription
                let url = format!("{}/v1/products/{}", BASE_URL, product);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(self.token.as_ref().unwrap())
                    .send()
                    .await
                    .context("Failed to get product information after adding to subscription")?;
                
                if response.status().is_success() {
                    let product_data: ProductResponse = response
                        .json()
                        .await
                        .context("Failed to parse product response")?;
                    
                    if let Some(link_items) = product_data.links {
                        let mut images = Vec::new();
                        let mut cad = Vec::new();
                        let mut datasheets = Vec::new();
                        
                        for link in link_items {
                            match link.key.as_str() {
                                "Image" => images.push(link.value),
                                key if CadFormat::from_api_key(key).is_some() => {
                                    if let Some(format) = CadFormat::from_api_key(key) {
                                        cad.push(CadFile {
                                            format,
                                            url: link.value,
                                            key: link.key,
                                        });
                                    }
                                },
                                "Datasheet" | "Data Sheet" => datasheets.push(link.value),
                                _ => {} // Ignore other link types like "Price", "ProductDetail"
                            }
                        }
                        
                        return Ok(ProductLinks {
                            images,
                            cad,
                            datasheets,
                        });
                    } else {
                        return Err(anyhow::anyhow!("No asset links found for product {}", product));
                    }
                } else {
                    let status = response.status();
                    let response_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Failed to get product information after adding to subscription. Status: {}. Response: {}",
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
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to get product information. Status: {}. Response: {}",
                status,
                response_text
            ));
        }
    }

    // Helper method to get default download directory
    fn get_default_download_dir(product: &str, asset_type: &str) -> Result<PathBuf> {
        let home = home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        
        let mut path = home;
        path.push("Downloads");
        path.push("mmc");
        path.push(product);
        path.push(asset_type);
        
        Ok(path)
    }

    // Helper method to ensure directory exists
    async fn ensure_directory_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .await
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        }
        Ok(())
    }

    // Helper method to format price output
    fn format_price_output(&self, product: &str, price_data: &[PriceInfo]) {
        println!("💰 Price for {}:", product);
        
        for price in price_data {
            // Format the unit of measure (singular if minimum is 1)
            let unit = if price.minimum_quantity == 1.0 && price.unit_of_measure.ends_with('s') {
                // Remove the 's' for singular form when minimum is 1
                price.unit_of_measure.trim_end_matches('s')
            } else {
                &price.unit_of_measure
            };
            
            println!("   ${:.2} per {}", price.amount, unit.to_lowercase());
            
            if price.minimum_quantity > 1.0 {
                println!("   Minimum order: {} {}", 
                    if price.minimum_quantity.fract() == 0.0 {
                        format!("{:.0}", price.minimum_quantity)
                    } else {
                        format!("{}", price.minimum_quantity)
                    },
                    price.unit_of_measure.to_lowercase()
                );
            } else {
                println!("   Minimum order: 1 {}", unit.to_lowercase());
            }
        }
    }

    // Helper method to download a single asset
    async fn download_asset(&self, asset_path: &str, output_dir: &Path, product: &str) -> Result<String> {
        let url = format!("{}{}", BASE_URL, asset_path);
        
        // Extract file extension from original filename
        let original_filename = asset_path
            .split('/')
            .last()
            .unwrap_or("download");
        
        let extension = if let Some(dot_pos) = original_filename.rfind('.') {
            &original_filename[dot_pos..].to_lowercase()
        } else {
            ""
        };
        
        // Generate clean filename: product.ext or product_variant.ext
        let final_filename = if asset_path.contains("NO%20THREADS") || asset_path.contains("NO THREADS") {
            format!("{}_no_threads{}", product, extension)
        } else if asset_path.contains("3D_") || asset_path.contains("3-D") {
            // For 3D PDFs or other 3D variants, add _3d suffix
            if extension == ".pdf" && (asset_path.contains("3D_") || asset_path.contains("3-D")) {
                format!("{}_3d{}", product, extension)
            } else {
                format!("{}{}", product, extension)
            }
        } else {
            format!("{}{}", product, extension)
        };
        
        let output_path = output_dir.join(&final_filename);
        
        println!("Downloading {}...", final_filename);
        
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .with_context(|| format!("Failed to download {}", final_filename))?;
        
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let mut file = fs::File::create(&output_path).await
                .with_context(|| format!("Failed to create file: {}", output_path.display()))?;
            
            file.write_all(&bytes).await
                .with_context(|| format!("Failed to write file: {}", output_path.display()))?;
            
            println!("✅ Downloaded {} ({} bytes)", final_filename, bytes.len());
            Ok(final_filename)
        } else {
            Err(anyhow::anyhow!(
                "Failed to download {}. Status: {}",
                final_filename,
                response.status()
            ))
        }
    }

    pub async fn download_images(&self, product: &str, output_dir: Option<&str>) -> Result<()> {
        let links = self.get_product_links(product).await?;
        
        let output_path = if let Some(dir) = output_dir {
            PathBuf::from(dir)
        } else {
            Self::get_default_download_dir(product, "images")?
        };
        
        Self::ensure_directory_exists(&output_path).await?;
        
        if links.images.is_empty() {
            println!("No images available for product {}", product);
            return Ok(());
        }
        
        println!("Found {} image(s) for product {}", links.images.len(), product);
        
        let mut downloaded = 0;
        for image_path in &links.images {
            match self.download_asset(image_path, &output_path, product).await {
                Ok(_) => downloaded += 1,
                Err(e) => println!("⚠️  Failed to download image: {}", e),
            }
        }
        
        println!("\n✅ Downloaded {}/{} images to: {}", 
            downloaded, links.images.len(), output_path.display());
        
        Ok(())
    }

    pub async fn download_cad(&self, product: &str, output_dir: Option<&str>, formats: &[&str], download_all: bool) -> Result<()> {
        let links = self.get_product_links(product).await?;
        
        let output_path = if let Some(dir) = output_dir {
            PathBuf::from(dir)
        } else {
            Self::get_default_download_dir(product, "cad")?
        };
        
        Self::ensure_directory_exists(&output_path).await?;
        
        if links.cad.is_empty() {
            println!("No CAD files available for product {}", product);
            return Ok(());
        }
        
        // Filter CAD files based on requested formats
        let files_to_download: Vec<&CadFile> = if download_all {
            links.cad.iter().collect()
        } else {
            links.cad.iter()
                .filter(|cad_file| formats.iter().any(|&format| cad_file.format.matches_filter(format)))
                .collect()
        };
        
        if files_to_download.is_empty() {
            if !download_all {
                println!("No CAD files found matching the requested formats: {}", formats.join(", "));
                println!("Available formats for product {}: {:?}", product, 
                    links.cad.iter().map(|f| &f.key).collect::<Vec<_>>());
            }
            return Ok(());
        }
        
        if !download_all && !formats.is_empty() {
            println!("Found {} CAD file(s) matching requested formats [{}] for product {}", 
                files_to_download.len(), formats.join(", "), product);
        } else {
            println!("Found {} CAD file(s) for product {}", files_to_download.len(), product);
        }
        
        let mut downloaded = 0;
        for cad_file in files_to_download {
            match self.download_asset(&cad_file.url, &output_path, product).await {
                Ok(filename) => {
                    println!("  📁 {} ({})", filename, cad_file.key);
                    downloaded += 1;
                }
                Err(e) => println!("⚠️  Failed to download {}: {}", cad_file.key, e),
            }
        }
        
        println!("\n✅ Downloaded {} CAD files to: {}", downloaded, output_path.display());
        
        Ok(())
    }

    pub async fn download_datasheets(&self, product: &str, output_dir: Option<&str>) -> Result<()> {
        let links = self.get_product_links(product).await?;
        
        let output_path = if let Some(dir) = output_dir {
            PathBuf::from(dir)
        } else {
            Self::get_default_download_dir(product, "datasheets")?
        };
        
        Self::ensure_directory_exists(&output_path).await?;
        
        if links.datasheets.is_empty() {
            println!("No datasheets available for product {}", product);
            return Ok(());
        }
        
        println!("Found {} datasheet(s) for product {}", links.datasheets.len(), product);
        
        let mut downloaded = 0;
        for datasheet_path in &links.datasheets {
            match self.download_asset(datasheet_path, &output_path, product).await {
                Ok(_) => downloaded += 1,
                Err(e) => println!("⚠️  Failed to download datasheet: {}", e),
            }
        }
        
        println!("\n✅ Downloaded {}/{} datasheets to: {}", 
            downloaded, links.datasheets.len(), output_path.display());
        
        Ok(())
    }
}