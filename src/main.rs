use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use std::path::PathBuf;
use dirs::{home_dir, config_dir};
use tokio::fs;

// Import from the new library structure
use mmcli::{McmasterClient, Credentials, OutputFormat};


#[derive(Parser)]
#[command(name = "mmc")]
#[command(about = "A CLI for McMaster-Carr API")]
#[command(version)]
struct Cli {
    /// Path to credentials file (JSON or TOML)
    #[arg(short, long, global = true)]
    credentials: Option<String>,
    
    /// Show detailed output including certificate loading and authentication details
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with McMaster-Carr API
    Login {
        /// Username (optional if using credentials file)
        #[arg(short, long)]
        username: Option<String>,
        /// Password (optional if using credentials file)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Logout from McMaster-Carr API
    Logout,
    /// Generate credentials file template
    InitCredentials {
        /// Path for credentials file (default: ~/.config/mmc/credentials.toml)
        #[arg(short, long)]
        path: Option<String>,
        /// Use JSON format instead of TOML
        #[arg(long)]
        json: bool,
    },
    /// Copy certificate to default location
    InitCert {
        /// Path to source certificate file
        source: String,
        /// Certificate password for conversion (if needed)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Add product to subscription
    Add {
        /// Product number
        product: String,
    },
    /// Remove product from subscription
    Remove {
        /// Product number
        product: String,
    },
    /// Get product information
    Info {
        /// Product number
        product: String,
        /// Output format
        #[arg(short, long, default_value_t = OutputFormat::Human)]
        output: OutputFormat,
        /// Comma-separated list of fields to display (default: all)
        #[arg(short, long, default_value = "all")]
        fields: String,
    },
    /// Get product price
    Price {
        /// Product number
        product: String,
        /// Output format
        #[arg(short, long, default_value_t = OutputFormat::Human)]
        output: OutputFormat,
    },
    /// List changes since a date (MM/dd/yyyy or MM/dd/yyyy HH:mm)
    Changes {
        /// Start date to check for changes (MM/dd/yyyy format)
        #[arg(short, long, default_value = "01/01/2024")]
        start: String,
    },
    /// Download product images
    Image {
        /// Product number
        product: String,
        /// Output directory (default: ~/Downloads/mmc/{product}/images/)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Download product CAD files
    Cad {
        /// Product number
        product: String,
        /// Output directory (default: ~/Downloads/mmc/{product}/cad/)
        #[arg(short, long)]
        output: Option<String>,
        /// Download DWG files
        #[arg(long)]
        dwg: bool,
        /// Download STEP files
        #[arg(long)]
        step: bool,
        /// Download DXF files
        #[arg(long)]
        dxf: bool,
        /// Download IGES files
        #[arg(long)]
        iges: bool,
        /// Download SolidWorks files (SLDPRT, SLDDRW)
        #[arg(long)]
        solidworks: bool,
        /// Download SAT files
        #[arg(long)]
        sat: bool,
        /// Download EDRW files
        #[arg(long)]
        edrw: bool,
        /// Download PDF files
        #[arg(long)]
        pdf: bool,
        /// Download all available CAD formats (default if no specific formats specified)
        #[arg(long)]
        all: bool,
    },
    /// Download product datasheets
    Datasheet {
        /// Product number
        product: String,
        /// Output directory (default: ~/Downloads/mmc/{product}/datasheets/)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate human-readable name for product
    Name {
        /// Product number
        product: String,
    },
    /// List locally tracked subscriptions
    List,
    /// Sync local subscriptions with API
    Sync,
    /// Import subscriptions from file
    Import {
        /// Path to file containing part numbers (one per line)
        file: String,
    },
}

async fn load_credentials_from_file(path: &str) -> Result<Credentials> {
    let credentials_path = PathBuf::from(path);
    if !credentials_path.exists() {
        return Err(anyhow::anyhow!("Credentials file not found: {}", path));
    }

    let content = fs::read_to_string(&credentials_path)
        .await
        .context("Failed to read credentials file")?;

    let credentials: Credentials = if path.ends_with(".json") {
        serde_json::from_str(&content)
            .context("Failed to parse JSON credentials file")?
    } else if path.ends_with(".toml") {
        toml::from_str(&content)
            .context("Failed to parse TOML credentials file")?
    } else {
        return Err(anyhow::anyhow!("Unsupported credentials file format. Use .json or .toml"));
    };

    Ok(credentials)
}

async fn load_default_credentials() -> Result<Credentials> {
    // Try XDG config directory first (~/.config/mmc/)
    if let Some(config_dir) = config_dir() {
        let mut creds_path = config_dir;
        creds_path.push("mmc");
        creds_path.push("credentials.toml");
        
        if creds_path.exists() {
            return load_credentials_from_file(creds_path.to_string_lossy().as_ref()).await;
        }
        
        // Try JSON in config dir
        creds_path.set_extension("json");
        if creds_path.exists() {
            return load_credentials_from_file(creds_path.to_string_lossy().as_ref()).await;
        }
    }
    
    // Fallback to legacy location (~/.mmcli/) for backward compatibility
    if let Some(home) = home_dir() {
        let mut creds_path = home;
        creds_path.push(".mmcli");
        creds_path.push("credentials.toml");
        
        if creds_path.exists() {
            return load_credentials_from_file(creds_path.to_string_lossy().as_ref()).await;
        }
        
        // Try JSON in legacy location
        creds_path.set_extension("json");
        if creds_path.exists() {
            return load_credentials_from_file(creds_path.to_string_lossy().as_ref()).await;
        }
    }

    Err(anyhow::anyhow!("No default credentials file found in ~/.config/mmc/ or ~/.mmcli/"))
}

async fn init_certificate(source_path: &str, _password: Option<&str>) -> Result<()> {
    let source = PathBuf::from(source_path);
    if !source.exists() {
        return Err(anyhow::anyhow!("Source certificate file not found: {}", source_path));
    }

    // Get the default certificate location
    let target_dir = if let Some(config_dir) = config_dir() {
        let mut path = config_dir;
        path.push("mmc");
        path
    } else {
        return Err(anyhow::anyhow!("Could not determine config directory"));
    };

    // Create the target directory if it doesn't exist
    fs::create_dir_all(&target_dir)
        .await
        .context("Failed to create config directory")?;

    // Determine target filename
    let target_filename = if source_path.ends_with(".pfx") || source_path.ends_with(".p12") {
        "certificate.pfx"
    } else {
        return Err(anyhow::anyhow!("Source file must be a .pfx or .p12 certificate file"));
    };

    let target = target_dir.join(target_filename);

    // Copy the certificate file
    fs::copy(&source, &target)
        .await
        .context("Failed to copy certificate file")?;

    println!("Certificate copied to: {}", target.display());
    println!("Certificate will now be auto-discovered by mmcli.");
    println!("You can omit certificate_path from your credentials file.");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load credentials first to create client with certificate
    let credentials = if let Some(creds_path) = &cli.credentials {
        Some(load_credentials_from_file(creds_path).await?)
    } else {
        // Try to load default credentials
        load_default_credentials().await.ok()
    };
    
    // Create client with quiet mode by default, verbose when requested
    let mut client = if cli.verbose {
        // Show detailed certificate and authentication messages
        McmasterClient::new_with_credentials(credentials)?
    } else {
        // Clean output by default
        McmasterClient::new_with_credentials_quiet(credentials)?
    };

    // Load existing token if available
    client.load_token().await?;

    match cli.command {
        Commands::Login { username, password } => {
            match (username, password) {
                (Some(u), Some(p)) => {
                    client.login(u, p).await?;
                }
                (None, None) => {
                    // Try to login with stored credentials
                    client.login_with_stored_credentials().await?;
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Either provide both username and password, or use credentials file"
                    ));
                }
            }
        }
        Commands::Logout => {
            client.logout().await?;
        }
        Commands::InitCredentials { path, json } => {
            let template_path = match path {
                Some(p) => p,
                None => {
                    // Use XDG config directory by default
                    if let Some(config_dir) = config_dir() {
                        let mut default_path = config_dir;
                        default_path.push("mmc");
                        if json {
                            default_path.push("credentials.json");
                        } else {
                            default_path.push("credentials.toml");
                        }
                        default_path.to_string_lossy().to_string()
                    } else {
                        // Fallback to legacy location
                        if json {
                            "~/.mmcli/credentials.json".to_string()
                        } else {
                            "~/.mmcli/credentials.toml".to_string()
                        }
                    }
                }
            };
            
            let expanded_path = if template_path.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    template_path.replace("~", &home.to_string_lossy())
                } else {
                    template_path
                }
            } else {
                template_path
            };

            client.save_credentials_template(&expanded_path).await?;
        }
        Commands::InitCert { source, password } => {
            init_certificate(&source, password.as_deref()).await?;
        }
        Commands::Add { product } => {
            client.add_product(&product).await?;
        }
        Commands::Remove { product } => {
            client.remove_product(&product).await?;
        }
        Commands::Info { product, output, fields } => {
            client.get_product(&product, output, &fields).await?;
        }
        Commands::Price { product, output } => {
            client.get_price(&product, output).await?;
        }
        Commands::Changes { start } => {
            client.get_changes(&start).await?;
        }
        Commands::Image { product, output } => {
            client.download_images(&product, output.as_deref()).await?;
        }
        Commands::Cad { product, output, dwg, step, dxf, iges, solidworks, sat, edrw, pdf, all } => {
            // Collect selected formats
            let mut formats = Vec::new();
            if dwg { formats.push("dwg"); }
            if step { formats.push("step"); }
            if dxf { formats.push("dxf"); }
            if iges { formats.push("iges"); }
            if solidworks { formats.push("solidworks"); }
            if sat { formats.push("sat"); }
            if edrw { formats.push("edrw"); }
            if pdf { formats.push("pdf"); }
            
            // If no specific formats selected or --all is specified, download all
            let download_all = all || formats.is_empty();
            
            client.download_cad(&product, output.as_deref(), &formats, download_all).await?;
        }
        Commands::Datasheet { product, output } => {
            client.download_datasheets(&product, output.as_deref()).await?;
        }
        Commands::Name { product } => {
            client.generate_name(&product).await?;
        }
        Commands::List => {
            client.list_subscriptions()?;
        }
        Commands::Sync => {
            client.sync_subscriptions().await?;
        }
        Commands::Import { file } => {
            client.import_subscriptions(&file)?;
        }
    }

    Ok(())
}
