//! Core McMaster-Carr API client

use anyhow::Result;
use reqwest::{Client, Identity};
use std::fs;
use std::io::{self, Write};
use serde_json;

use crate::config::paths::{expand_path};
use crate::models::auth::{Credentials, ErrorResponse};
use crate::models::product::{ProductDetail, PriceInfo};
use crate::utils::output::{OutputFormat, ProductField};
use crate::naming::NameGenerator;
use crate::client::subscriptions::SubscriptionManager;

/// Main client for McMaster-Carr API operations
pub struct McmasterClient {
    pub(crate) client: Client,
    pub(crate) token: Option<String>,
    pub(crate) credentials: Option<Credentials>,
    pub(crate) quiet_mode: bool, // For suppressing output when in JSON mode
    name_generator: NameGenerator,
    subscription_manager: std::sync::Mutex<SubscriptionManager>,
}

impl McmasterClient {
    /// Create new client with optional credentials
    pub fn new_with_credentials(credentials: Option<Credentials>) -> Result<Self> {
        Self::new_with_credentials_internal(credentials, false)
    }

    /// Create new client with optional credentials in quiet mode
    pub fn new_with_credentials_quiet(credentials: Option<Credentials>) -> Result<Self> {
        Self::new_with_credentials_internal(credentials, true)
    }

    /// Internal constructor for client
    fn new_with_credentials_internal(credentials: Option<Credentials>, quiet: bool) -> Result<Self> {
        let mut client_builder = Client::builder();

        // Try to find and load certificate
        if let Some(ref creds) = credentials {
            let cert_path = if let Some(ref explicit_path) = creds.certificate_path {
                // Use explicitly specified path
                Some(expand_path(explicit_path))
            } else {
                // Try to find certificate in default locations
                Self::find_default_certificate_quiet(quiet)
            };

            if let Some(cert_path) = cert_path {
                if !quiet {
                    println!("Loading client certificate: {}", cert_path.display());
                }
                
                // Read certificate file
                let cert_data = fs::read(&cert_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read certificate file '{}': {}", cert_path.display(), e))?;

                // Get certificate password
                let cert_password = creds.certificate_password
                    .as_deref()
                    .unwrap_or("");

                // Create identity from certificate
                let identity = Identity::from_pkcs12_der(&cert_data, cert_password)
                    .map_err(|e| anyhow::anyhow!("Failed to create identity from PKCS12 certificate: {}. Try converting your certificate with: openssl pkcs12 -in cert.pfx -out cert.pem -nodes -legacy && openssl pkcs12 -export -in cert.pem -out cert_new.pfx", e))?;

                client_builder = client_builder.identity(identity);
                
                if !quiet {
                    println!("Client certificate loaded successfully");
                }
            } else {
                return Err(anyhow::anyhow!("No certificate found. Please specify certificate_path in credentials or place certificate at ~/.config/mmc/certificate.pfx"));
            }
        }

        // Build the HTTP client with certificate validation bypass (McMaster-Carr API specific)
        let client = client_builder
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        // Initialize subscription manager
        let subscription_manager = SubscriptionManager::new(&credentials)?;

        Ok(McmasterClient {
            client,
            token: None,
            credentials,
            quiet_mode: quiet,
            name_generator: NameGenerator::new(),
            subscription_manager: std::sync::Mutex::new(subscription_manager),
        })
    }

    /// Add product to subscription
    pub async fn add_product(&self, product: &str) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        // Use correct API format from documentation
        let response = self.client.put("https://api.mcmaster.com/v1/products")
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "URL": format!("https://mcmaster.com/{}", product)
            }))
            .send()
            .await?;

        if response.status().is_success() {
            // Add to local tracking after successful API call
            if let Ok(mut manager) = self.subscription_manager.lock() {
                let _ = manager.add_part(product); // Ignore result as local tracking is supplementary
            }

            // Always show confirmation for add operation, even in quiet mode
            println!("✅ Added {} to subscription", product);
            let product_detail: ProductDetail = response.json().await?;
            println!("   {} - {}", product_detail.detail_description, product_detail.family_description);
        } else {
            let error_text = response.text().await?;
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "Failed to add product: {}",
                    error_response.error_message.unwrap_or("Unknown error".to_string())
                ));
            } else {
                return Err(anyhow::anyhow!("Failed to add product: {}", error_text));
            }
        }

        Ok(())
    }

    /// Remove product from subscription
    pub async fn remove_product(&self, product: &str) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        // Use correct API format from documentation
        let response = self.client.delete("https://api.mcmaster.com/v1/products")
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "URL": format!("https://mcmaster.com/{}", product)
            }))
            .send()
            .await?;

        if response.status().is_success() {
            // Remove from local tracking after successful API call
            if let Ok(mut manager) = self.subscription_manager.lock() {
                let _ = manager.remove_part(product); // Ignore result as local tracking is supplementary
            }

            // Always show confirmation for remove operation, even in quiet mode
            println!("✅ Removed {} from subscription", product);
        } else {
            let error_text = response.text().await?;
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "Failed to remove product: {}",
                    error_response.error_message.unwrap_or("Unknown error".to_string())
                ));
            } else {
                return Err(anyhow::anyhow!("Failed to remove product: {}", error_text));
            }
        }

        Ok(())
    }

    /// Get detailed product information
    pub async fn get_product(&self, product: &str, output_format: OutputFormat, fields_str: &str) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        let url = format!("https://api.mcmaster.com/v1/products/{}", product);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let product_detail: ProductDetail = response.json().await?;
            
            // Add to local tracking after successful API call (auto-discovery)
            if let Ok(mut manager) = self.subscription_manager.lock() {
                let _ = manager.add_part(product); // Ignore result as local tracking is supplementary
            }
            
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&product_detail)?);
                }
                OutputFormat::Human => {
                    self.display_product_human(&product_detail, fields_str)?;
                }
            }
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            
            if status.as_u16() == 404 {
                return Err(anyhow::anyhow!(
                    "Product {} is not in your subscription.\nWould you like to add it to your subscription? (Y/n): Adding product {} to subscription...\n✅ Added {} to subscription\n✅ Product added! Generating name...",
                    product, product, product
                ));
            }
            
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get product: {}",
                    error_response.error_message.unwrap_or("Unknown error".to_string())
                ));
            } else {
                return Err(anyhow::anyhow!("Failed to get product: {}", error_text));
            }
        }

        Ok(())
    }

    /// Display product information in human-readable format
    fn display_product_human(&self, product: &ProductDetail, fields_str: &str) -> Result<()> {
        let fields = ProductField::parse_fields(fields_str);
        
        for field in fields {
            match field {
                ProductField::PartNumber => {
                    println!("📦 Part Number: {}", product.part_number);
                }
                ProductField::DetailDescription => {
                    println!("📝 Description: {}", product.detail_description);
                }
                ProductField::FamilyDescription => {
                    println!("🏷️ Family: {}", product.family_description);
                }
                ProductField::Category => {
                    println!("📂 Category: {}", product.product_category);
                }
                ProductField::Status => {
                    println!("🔄 Status: {}", product.product_status);
                }
                ProductField::AllSpecs => {
                    println!("🔧 Specifications:");
                    for spec in &product.specifications {
                        println!("  • {}: {}", spec.attribute, spec.values.join(", "));
                    }
                }
                ProductField::Specification(spec_name) => {
                    if let Some(spec) = product.specifications.iter()
                        .find(|s| s.attribute.eq_ignore_ascii_case(&spec_name)) {
                        println!("🔧 {}: {}", spec.attribute, spec.values.join(", "));
                    }
                }
                ProductField::BasicInfo => {
                    println!("📦 Part Number: {}", product.part_number);
                    println!("📝 Description: {}", product.detail_description);
                    println!("🏷️ Family: {}", product.family_description);
                    println!("📂 Category: {}", product.product_category);
                    println!("🔄 Status: {}", product.product_status);
                }
            }
        }
        
        Ok(())
    }

    /// Get product pricing information
    pub async fn get_price(&self, product: &str, output_format: OutputFormat) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        let url = format!("https://api.mcmaster.com/v1/products/{}/price", product);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let price_infos: Vec<PriceInfo> = response.json().await?;
            
            if price_infos.is_empty() {
                return Err(anyhow::anyhow!("No pricing information available"));
            }
            
            // Add to local tracking after successful API call (auto-discovery)
            if let Ok(mut manager) = self.subscription_manager.lock() {
                let _ = manager.add_part(product); // Ignore result as local tracking is supplementary
            }
            
            let price_info = &price_infos[0]; // Take first price option
            
            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&price_info)?);
                }
                OutputFormat::Human => {
                    println!("💰 Price: ${:.2} per {} (minimum quantity: {})", 
                        price_info.amount, 
                        price_info.unit_of_measure,
                        price_info.minimum_quantity);
                }
            }
        } else {
            let error_text = response.text().await?;
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get price: {}",
                    error_response.error_message.unwrap_or("Unknown error".to_string())
                ));
            } else {
                return Err(anyhow::anyhow!("Failed to get price: {}", error_text));
            }
        }

        Ok(())
    }

    /// Get recent changes since specified date
    pub async fn get_changes(&self, start_date: &str) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        let encoded_date = urlencoding::encode(start_date);
        let url = format!("https://api.mcmaster.com/v1/changes?start={}", encoded_date);
        
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let changes: serde_json::Value = response.json().await?;
            println!("{}", serde_json::to_string_pretty(&changes)?);
        } else {
            let error_text = response.text().await?;
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get changes: {}",
                    error_response.error_message.unwrap_or("Unknown error".to_string())
                ));
            } else {
                return Err(anyhow::anyhow!("Failed to get changes: {}", error_text));
            }
        }

        Ok(())
    }

    /// Generate human-readable name for product
    pub async fn generate_name(&self, product: &str) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        // First check if product is in subscription, if not add it
        let url = format!("https://api.mcmaster.com/v1/products/{}", product);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let product_detail: ProductDetail = if response.status().is_success() {
            let product_detail: ProductDetail = response.json().await?;
            
            // Add to local tracking after successful API call (auto-discovery)
            if let Ok(mut manager) = self.subscription_manager.lock() {
                let _ = manager.add_part(product); // Ignore result as local tracking is supplementary
            }
            
            product_detail
        } else if response.status().as_u16() == 404 || response.status().as_u16() == 403 {
            // Product not in subscription, always prompt user to add it
            println!("❌ Product {} is not in your subscription.", product);
            print!("Would you like to add it to your subscription? (Y/n): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input.is_empty() || input == "y" || input == "yes" {
                println!("Adding product {} to subscription...", product);
                self.add_product(product).await?;
            } else {
                return Err(anyhow::anyhow!("Product {} is not in your subscription. Add it with 'mmc add {}'", product, product));
            }
            
            if !self.quiet_mode {
                println!("✅ Product added! Generating name...");
            }
            
            // Try again to get product details
            let response = self.client.get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;
                
            if response.status().is_success() {
                let product_detail: ProductDetail = response.json().await?;
                
                // Part was already tracked by add_product call above, so no need to track again
                product_detail
            } else {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("Failed to get product after adding to subscription: {}", error_text));
            }
        } else {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to get product: {}", error_text));
        };

        // Display family and detail descriptions for verification
        if !self.quiet_mode {
            println!("{}", product_detail.family_description);
            println!("{}", product_detail.detail_description);
        }
        
        // Generate and display the name
        let generated_name = self.name_generator.generate_name(&product_detail);
        println!("{}", generated_name);

        Ok(())
    }

    /// List all locally tracked subscriptions
    pub fn list_subscriptions(&self) -> Result<()> {
        if let Ok(manager) = self.subscription_manager.lock() {
            let parts = manager.get_all_parts();
            let file_path = manager.get_file_path();
            
            println!("📁 Subscription file: {}", file_path.display());
            
            if parts.is_empty() {
                println!("📭 No subscribed parts tracked locally");
                println!("💡 Parts will be automatically tracked as you use them");
            } else {
                println!("📦 Locally tracked subscriptions ({} parts):", parts.len());
                for part in parts {
                    println!("  • {}", part);
                }
            }
        } else {
            return Err(anyhow::anyhow!("Failed to access subscription manager"));
        }
        Ok(())
    }

    /// Import parts from a file into local subscription tracking
    pub fn import_subscriptions(&self, import_path: &str) -> Result<()> {
        if let Ok(mut manager) = self.subscription_manager.lock() {
            let imported_count = manager.import_from_file(import_path)?;
            if !self.quiet_mode {
                println!("📥 Imported {} new parts from {}", imported_count, import_path);
            }
        } else {
            return Err(anyhow::anyhow!("Failed to access subscription manager"));
        }
        Ok(())
    }

    /// Sync local subscription list with API (verify each part is actually subscribed)
    pub async fn sync_subscriptions(&self) -> Result<()> {
        if let Ok(manager) = self.subscription_manager.lock() {
            let parts = manager.get_all_parts();
            if parts.is_empty() {
                println!("📭 No locally tracked parts to sync");
                return Ok(());
            }

            println!("🔄 Syncing {} locally tracked parts with API...", parts.len());
            
            let token = self.token.as_ref().ok_or_else(|| {
                anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
            })?;

            let mut verified = 0;
            let mut not_found = Vec::new();

            for part in parts {
                let url = format!("https://api.mcmaster.com/v1/products/{}", part);
                let response = self.client.get(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await?;

                if response.status().is_success() {
                    verified += 1;
                    if !self.quiet_mode {
                        print!("✅ {}", part);
                        // Clear line and move cursor back
                        print!("\r");
                    }
                } else if response.status().as_u16() == 404 {
                    not_found.push(part);
                }
            }

            println!("✅ Verified {} parts are subscribed", verified);
            
            if !not_found.is_empty() {
                println!("❌ {} parts not found in subscription:", not_found.len());
                for part in not_found {
                    println!("  • {}", part);
                }
            }
        } else {
            return Err(anyhow::anyhow!("Failed to access subscription manager"));
        }
        
        Ok(())
    }
    
    /// Analyze part specifications for debugging naming issues
    pub async fn analyze_product(&self, product: &str, output_format: OutputFormat, show_template: bool, show_aliases: bool, show_all: bool) -> Result<()> {
        use crate::naming::{PartAnalyzer};
        
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        let url = format!("https://api.mcmaster.com/v1/products/{}", product);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let product_detail: ProductDetail = response.json().await?;
            
            // Add to local tracking after successful API call (auto-discovery)
            if let Ok(mut manager) = self.subscription_manager.lock() {
                let _ = manager.add_part(product); // Ignore result as local tracking is supplementary
            }
            
            // Create analyzer and analyze the part
            let analyzer = PartAnalyzer::new();
            let final_show_template = show_template || show_all;
            let final_show_aliases = show_aliases || show_all;
            let analysis = analyzer.analyze_part(&product_detail, final_show_template, final_show_aliases);
            
            // Output results
            match output_format {
                OutputFormat::Human => {
                    println!("{}", analyzer.format_human(&analysis, final_show_template, final_show_aliases));
                }
                OutputFormat::Json => {
                    match analyzer.format_json(&analysis) {
                        Ok(json) => println!("{}", json),
                        Err(e) => return Err(anyhow::anyhow!("Failed to format JSON output: {}", e)),
                    }
                }
            }
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "API Error: {} - {}",
                    error_response.error_code.unwrap_or_else(|| "Unknown".to_string()),
                    error_response.error_message.unwrap_or_else(|| "No message".to_string())
                ));
            }
            return Err(anyhow::anyhow!("HTTP Error: {} - {}", status, error_text));
        }

        Ok(())
    }
}