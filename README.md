# McMaster-Carr CLI (mmc)

A command-line interface for the McMaster-Carr Product Information API. This tool allows you to authenticate, manage product subscriptions, retrieve product data, and download files using client certificate authentication.

## Features

- 🔐 **Client Certificate Authentication** - Secure authentication using PKCS12 certificates
- 📁 **XDG Standard Configuration** - Follows XDG Base Directory Specification
- 🔑 **Token Management** - Automatic token storage and reuse (24-hour validity)
- 📦 **Product Management** - Add/remove products from subscription
- 💰 **Product Information** - Get detailed product data and pricing
- 📊 **Change Tracking** - Monitor product updates and changes
- 💾 **File Downloads** - Download CAD files, images, and datasheets with clean filenames
- 🚫 **No Flags Required** - Works without `-c` credentials flag for everyday use

## Installation

### Prerequisites
- Rust (latest stable version)
- OpenSSL development libraries
- Valid McMaster-Carr API account and client certificate

### Build from Source
```bash
git clone <repository-url>
cd mmcli
cargo build --release
```

## Setup

### 1. Certificate Conversion (Required)

McMaster-Carr provides PFX certificates that may use deprecated encryption. You'll need to convert them to a modern format:

```bash
# Step 1: Convert old PFX to PEM format (requires certificate password)
openssl pkcs12 -in your-certificate.pfx -out certificate.pem -nodes -legacy

# Step 2: Convert back to modern PKCS12 format
openssl pkcs12 -export -in certificate.pem -out certificate_new.pfx -passout pass:YOUR_CERT_PASSWORD
```

**Why this is needed:** Original PFX files often use RC2-40-CBC encryption which is deprecated and not supported by modern OpenSSL versions.

**Recommended:** Copy the converted certificate to `~/.config/mmc/certificate.pfx` for automatic discovery:

```bash
# Use the built-in command to copy certificate to default location
mmc init-cert certificate_new.pfx

# Or manually copy
mkdir -p ~/.config/mmc
cp certificate_new.pfx ~/.config/mmc/certificate.pfx
```

### 2. Credentials Setup

Create a credentials file to store your authentication information:

```bash
# Generate a template in XDG config directory (~/.config/mmc/credentials.toml)
mmc init-credentials

# Or specify a custom path
mmc init-credentials -p ./my-credentials.toml

# For JSON format in XDG config directory
mmc init-credentials --json
```

Edit the generated file with your actual credentials:

**TOML format:**
```toml
username = "your@email.com"
password = "your_password"

# Certificate settings (optional - will auto-discover if not specified)
# Default locations checked:
#   ~/.config/mmc/certificate.pfx
#   ~/.config/mmc/certificate.p12
#   ~/.mmcli/certificate.pfx (legacy)
#   ~/.mmcli/certificate.p12 (legacy)
certificate_path = "~/.config/mmc/certificate.pfx"
certificate_password = "certificate_password"
```

**JSON format:**
```json
{
  "username": "your@email.com",
  "password": "your_password",
  "certificate_path": "~/.config/mmc/certificate.pfx",
  "certificate_password": "certificate_password"
}
```

**Simplified TOML (auto-discovery):**
```toml
username = "your@email.com"
password = "your_password"
certificate_password = "certificate_password"
```
*Place your certificate at `~/.config/mmc/certificate.pfx` and omit certificate_path*

### 3. Directory Structure

**Recommended XDG Standard Setup:**
```
~/.config/mmc/
├── credentials.toml           # Your credentials
├── certificate.pfx            # Your converted certificate (auto-discovered)
└── token                      # Auth token (auto-generated)
```

**Alternative Setup (custom paths):**
```
mmcli/
├── certs/
│   ├── certificate_new.pfx    # Your converted certificate
│   └── certificate.pem        # Intermediate PEM file (can delete)
├── credentials.toml            # Your credentials with explicit certificate_path
└── target/                     # Build artifacts
```

## Usage

### Authentication

```bash
# Login with default credentials (~/.config/mmc/credentials.toml)
mmc login

# Login with custom credentials file
mmc -c credentials.toml login

# Login with username/password directly (requires credentials file for certificate)
mmc login -u username -p password
```

### Product Management

```bash
# Add product to subscription (required before accessing product data)
mmc add 90128a211
# Returns: "✅ Added 90128a211 to subscription" with product details

# Remove product from subscription
mmc remove 90128a211
# Returns: "✅ Removed 90128a211 from subscription"
```

### Product Information

```bash
# Get detailed product information (human-friendly)
mmc info 98164A133

# Get product information in JSON format (scriptable)
mmc info 98164A133 --output json

# Get specific fields only
mmc info 98164A133 --fields part-number,material,thread-size

# Get product pricing (human-friendly)
mmc price 98164A133

# Get pricing in JSON format
mmc price 98164A133 --output json

# List recent changes (requires start date)
mmc changes -s "01/01/2024"

# List changes from a specific date with time
mmc changes -s "08/20/2025 10:30"
```

### File Downloads

```bash
# Download product images
mmc image 91831A030

# Download specific CAD formats
mmc cad 91831A030 --step --dwg

# Download all available CAD formats
mmc cad 91831A030 --all

# Download datasheets
mmc datasheet 91831A030
```

**Note**: Files are saved with clean naming using just the McMaster-Carr part number and appropriate extension.

### Subscription Management

```bash
# List locally tracked subscriptions
mmc list

# Sync local subscriptions with API
mmc sync

# Import subscriptions from file (one part number per line)
mmc import parts.txt
```

### Session Management

```bash
# Logout (invalidates current token)
mmc logout
```

## Working Examples

Here are real examples using actual McMaster-Carr part numbers:

### Complete Setup (First Time)
```bash
# 1. Convert certificate (if needed)
openssl pkcs12 -in original.pfx -out temp.pem -nodes -legacy
openssl pkcs12 -export -in temp.pem -out certificate.pfx -passout pass:YOUR_PASSWORD

# 2. Copy certificate to default location
mmc init-cert certificate.pfx

# 3. Generate credentials template
mmc init-credentials

# 4. Edit ~/.config/mmc/credentials.toml with your credentials
# (certificate_path is optional - will auto-discover)

# 5. Login (once per day, token lasts 24 hours)
mmc login
```

### Daily Usage (No Flags Needed!)
```bash
# Add parts to subscription
mmc add 90128a211  # M4x0.7mm Socket Head Screws
mmc add 92141A008  # #6 Stainless Steel Washers
mmc add 92141A029  # 1/4" Stainless Steel Washers

# Get detailed product information
mmc info 90128a211
# Returns: specifications, CAD links, material properties, etc.

# Check pricing
mmc price 92141A008
# Returns: pricing information per unit/pack

# Monitor changes since start of year
mmc changes -s "01/01/2024"
# Returns: list of part numbers that have been updated
```

## Configuration

### Default Locations (XDG Standard)

- **Credentials**: `~/.config/mmc/credentials.toml` or `~/.config/mmc/credentials.json`
- **Auth Token**: `~/.config/mmc/token`
- **Legacy Support**: Falls back to `~/.mmcli/` for backward compatibility

### Global Options

- `-c, --credentials <FILE>` - Specify credentials file path
- `-v, --verbose` - Show detailed output including certificate loading and authentication details
- `-h, --help` - Show help information
- `-V, --version` - Show version information

## API Reference

The CLI interacts with McMaster-Carr's Product Information API:

- **Base URL**: `https://api.mcmaster.com`
- **Authentication**: Client certificate + username/password
- **Token Validity**: 24 hours
- **Rate Limits**: Applied to bandwidth-intensive endpoints

### Available Commands

| Command | Endpoint/Action | Description |
|---------|----------|-------------|
| `login` | `/v1/login` | Authenticate and get token |
| `logout` | `/v1/logout` | Invalidate current token |
| `init-credentials` | Local | Generate credentials file template |
| `init-cert` | Local | Copy certificate to default location |
| `add` | `/v1/products` | Add product to subscription |
| `remove` | `/v1/products` | Remove product from subscription |
| `info` | `/v1/products/*` | Get product information |
| `price` | `/v1/products/*/price` | Get product pricing |
| `changes` | `/v1/changes?start=MM/dd/yyyy` | Get change notifications since date |
| `image` | API | Download product images |
| `cad` | API | Download product CAD files |
| `datasheet` | API | Download product datasheets |
| `list` | Local | List locally tracked subscriptions |
| `sync` | API | Sync local subscriptions with API |
| `import` | Local | Import subscriptions from file |

## Security

- 🔒 All credential files are excluded from git via `.gitignore`
- 🗂️ Certificates stored in `certs/` directory (also git-ignored)
- 🔑 Tokens stored locally in `~/.config/mmc/` directory (XDG standard)
- 🛡️ Uses TLS with client certificate authentication

## Troubleshooting

### Certificate Issues

**Error: `Failed to create identity from PKCS12 certificate`**
- Your PFX uses deprecated encryption (RC2-40-CBC)
- Follow the certificate conversion steps above

**Error: `Certificate file not found`**
- Verify the path in your credentials file
- Ensure certificate file exists and is readable

### API Issues

**Error: `Unexpected response format. Response: <!DOCTYPE html>`**
- Wrong API endpoint (should be `api.mcmaster.com` not `www.mcmaster.com`)
- Missing or invalid client certificate

**Error: `Not authenticated`**
- Token expired (tokens last 24 hours)
- Run `mmc login` to get a new token

### SSL Issues

**Error: `certificate verify failed`**
- SSL verification issues with API endpoint
- The CLI handles this automatically with certificate validation bypass

## Development

### Project Structure

```
src/
├── lib.rs                    # Library root with exports
├── main.rs                   # CLI entry point
├── client/                   # API client functionality
│   ├── mod.rs               # Module declarations
│   ├── api.rs               # Core API operations
│   ├── auth.rs              # Authentication handling
│   ├── downloads.rs         # File downloads
│   └── subscriptions.rs     # Subscription management
├── models/                   # Data structures
│   ├── mod.rs               # Model exports
│   ├── api.rs               # API response models
│   ├── auth.rs              # Authentication models
│   └── product.rs           # Product data models
├── config/                   # Configuration management
│   ├── mod.rs               # Module declarations
│   └── paths.rs             # XDG-compliant path handling
└── utils/                    # Utilities
    ├── mod.rs               # Module declarations
    ├── output.rs            # Output formatting
    └── error.rs             # Error handling
```

### Dependencies

- `clap` - Command line parsing with derive macros
- `reqwest` - HTTP client with native-tls support
- `serde` - JSON/TOML serialization
- `tokio` - Async runtime
- `anyhow` - Error handling
- `native-tls` - TLS with client certificate support
- `dirs` - Cross-platform directory paths
- `toml` - TOML configuration file parsing
- `urlencoding` - URL parameter encoding

## API Integration

For API integration details, contact McMaster-Carr at: **eCommerce@mcmaster.com**

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

[Contributing guidelines here]
