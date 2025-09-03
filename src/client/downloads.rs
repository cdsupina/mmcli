//! Download functionality for images, CAD files, and datasheets

use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::models::auth::ErrorResponse;
use crate::models::api::{ProductResponse, ProductLinks, CadFile, CadFormat};

/// Download-related methods for McmasterClient
impl super::api::McmasterClient {
    /// Download product images
    pub async fn download_images(&self, product: &str, output_dir: Option<&str>) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        // Get product links
        let links = self.get_product_links(product, token).await?;

        if links.images.is_empty() {
            println!("ℹ️  No images available for product {}", product);
            return Ok(());
        }

        // Determine output directory
        let output_path = self.get_output_path(output_dir, product, "images");
        fs::create_dir_all(&output_path).await?;

        println!("📥 Downloading {} images to {}", links.images.len(), output_path.display());

        for (i, image_url) in links.images.iter().enumerate() {
            let filename = if links.images.len() == 1 {
                format!("{}.jpg", product)
            } else {
                format!("{}_{}.jpg", product, i + 1)
            };
            let file_path = output_path.join(&filename);

            match self.download_file(image_url, &file_path).await {
                Ok(_) => println!("  ✅ Downloaded {}", filename),
                Err(e) => eprintln!("  ❌ Failed to download {}: {}", filename, e),
            }
        }

        println!("✅ Image download complete");
        Ok(())
    }

    /// Download CAD files
    pub async fn download_cad(&self, product: &str, output_dir: Option<&str>, formats: &[&str], download_all: bool) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        // Get product links
        let links = self.get_product_links(product, token).await?;

        if links.cad.is_empty() {
            println!("ℹ️  No CAD files available for product {}", product);
            return Ok(());
        }

        // Filter CAD files by requested formats
        let filtered_cad: Vec<&CadFile> = if download_all {
            links.cad.iter().collect()
        } else if formats.is_empty() {
            // If no specific formats requested, download all
            links.cad.iter().collect()
        } else {
            links.cad.iter()
                .filter(|cad_file| {
                    formats.iter().any(|format| cad_file.format.matches_filter(format))
                })
                .collect()
        };

        if filtered_cad.is_empty() {
            println!("ℹ️  No CAD files match the requested formats");
            return Ok(());
        }

        // Determine output directory
        let output_path = self.get_output_path(output_dir, product, "cad");
        fs::create_dir_all(&output_path).await?;

        println!("📥 Downloading {} CAD files to {}", filtered_cad.len(), output_path.display());

        for cad_file in filtered_cad {
            let extension = self.get_cad_extension(&cad_file.format);
            let filename = format!("{}.{}", product, extension);
            let file_path = output_path.join(&filename);

            match self.download_file(&cad_file.url, &file_path).await {
                Ok(_) => println!("  ✅ Downloaded {} ({})", filename, cad_file.key),
                Err(e) => eprintln!("  ❌ Failed to download {}: {}", filename, e),
            }
        }

        println!("✅ CAD download complete");
        Ok(())
    }

    /// Download datasheets
    pub async fn download_datasheets(&self, product: &str, output_dir: Option<&str>) -> Result<()> {
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;

        // Get product links
        let links = self.get_product_links(product, token).await?;

        if links.datasheets.is_empty() {
            println!("ℹ️  No datasheets available for product {}", product);
            return Ok(());
        }

        // Determine output directory
        let output_path = self.get_output_path(output_dir, product, "datasheets");
        fs::create_dir_all(&output_path).await?;

        println!("📥 Downloading {} datasheets to {}", links.datasheets.len(), output_path.display());

        for (i, datasheet_url) in links.datasheets.iter().enumerate() {
            let filename = if links.datasheets.len() == 1 {
                format!("{}.pdf", product)
            } else {
                format!("{}_{}.pdf", product, i + 1)
            };
            let file_path = output_path.join(&filename);

            match self.download_file(datasheet_url, &file_path).await {
                Ok(_) => println!("  ✅ Downloaded {}", filename),
                Err(e) => eprintln!("  ❌ Failed to download {}: {}", filename, e),
            }
        }

        println!("✅ Datasheet download complete");
        Ok(())
    }

    /// Get product links from API
    async fn get_product_links(&self, product: &str, token: &str) -> Result<ProductLinks> {
        let url = format!("https://api.mcmaster.com/v1/products/{}", product);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get product links: {}",
                    error_response.error_message.unwrap_or("Unknown error".to_string())
                ));
            } else {
                return Err(anyhow::anyhow!("Failed to get product links: {}", error_text));
            }
        }

        let product_response: ProductResponse = response.json().await?;
        let links = product_response.links.unwrap_or_default();

        // Parse links into categories
        let mut images = Vec::new();
        let mut cad_files = Vec::new();
        let mut datasheets = Vec::new();

        for link in links {
            if link.key.contains("Image") {
                images.push(link.value);
            } else if let Some(format) = CadFormat::from_api_key(&link.key) {
                cad_files.push(CadFile {
                    format,
                    url: link.value,
                    key: link.key,
                });
            } else if link.key.contains("Data Sheet") || link.key.contains("Datasheet") {
                datasheets.push(link.value);
            }
        }

        Ok(ProductLinks {
            images,
            cad: cad_files,
            datasheets,
        })
    }

    /// Download a file from URL to local path
    async fn download_file(&self, url: &str, file_path: &PathBuf) -> Result<()> {
        // Convert relative URLs to absolute URLs
        let full_url = if url.starts_with('/') {
            format!("https://api.mcmaster.com{}", url)
        } else {
            url.to_string()
        };
        
        // Add authentication token for download requests
        let token = self.token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not authenticated. Please login first with 'mmc login'")
        })?;
        
        let response = self.client.get(&full_url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download file: HTTP {}", response.status()));
        }

        let mut file = fs::File::create(file_path).await?;
        let content = response.bytes().await?;
        file.write_all(&content).await?;
        
        Ok(())
    }

    /// Get output path for downloads
    fn get_output_path(&self, output_dir: Option<&str>, product: &str, category: &str) -> PathBuf {
        if let Some(dir) = output_dir {
            PathBuf::from(dir)
        } else {
            // Default to ~/Downloads/mmc/{product}/{category}/
            dirs::download_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("mmc")
                .join(product)
                .join(category)
        }
    }

    /// Get file extension for CAD format
    fn get_cad_extension(&self, format: &CadFormat) -> &'static str {
        match format {
            CadFormat::Dwg => "dwg",
            CadFormat::Step => "step",
            CadFormat::Dxf => "dxf",
            CadFormat::Iges => "iges",
            CadFormat::Solidworks => "sldprt",
            CadFormat::Sat => "sat",
            CadFormat::Edrw => "edrw",
            CadFormat::Pdf => "pdf",
        }
    }
}