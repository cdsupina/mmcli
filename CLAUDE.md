# McMaster-Carr CLI (mmcli) - Claude Configuration

## Project Overview

A comprehensive command-line interface for interacting with McMaster-Carr's Product Information API.

## Quick Commands

### Build & Test
```bash
# Build release version
cargo build --release
```

### Development
```bash
# Update dependencies
cargo update

# Clean build
cargo clean && cargo build --release

# Check for warnings
cargo build --release 2>&1 | grep warning
```

## Project Structure

### Modular Architecture

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

## Key Features

### 1. API Integration
- **Certificate-based auth**: Uses PKCS12 client certificates
- **Token management**: Automatic token loading/saving
- **Full CRUD**: Add/remove products, get info/pricing, download files
- **Subscription management**: List, sync, and import subscriptions

### 2. Output Formats
- **Human-readable**: Formatted with emojis and clear descriptions
- **JSON**: Machine-readable for automation
- **Field selection**: Choose specific product attributes

## Authentication Setup

### Certificate Configuration
1. Place certificate at `~/.config/mmc/certificate.pfx`
2. Create credentials file: `mmc init-credentials`
3. Edit credentials with your username/password

### Default Locations Checked
- `~/.config/mmc/certificate.pfx` (preferred)
- `~/.config/mmc/certificate.p12`
- `~/.mmcli/certificate.pfx` (legacy)
- `~/.mmcli/certificate.p12` (legacy)

## Common Operations

### Product Information
```bash
# Get full product details
mmc info 91831A030

# Get specific fields
mmc info 91831A030 -f "material,thread-size"

# JSON output
mmc info 91831A030 -o json
```

### File Downloads
```bash
# Download images (saves as {part_number}.jpg)
mmc image 91831A030

# Download specific CAD formats (saves as {part_number}.step, {part_number}.dwg)
mmc cad 91831A030 --step --dwg

# Download all CAD files
mmc cad 91831A030 --all

# Download datasheets (saves as {part_number}.pdf)
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

### Other Commands
```bash
# Logout from API
mmc logout

# Copy certificate to default location
mmc init-cert /path/to/certificate.pfx

# List changes since a date
mmc changes --start "01/01/2024"
```

## Dependencies

### Key Dependencies
- `clap` - CLI argument parsing with derive macros
- `tokio` - Async runtime for HTTP operations
- `reqwest` - HTTP client with TLS support
- `serde` / `serde_json` - Serialization/deserialization
- `anyhow` - Error handling
- `dirs` - Cross-platform directory paths
- `toml` - Configuration file parsing
- `native-tls` - TLS/certificate handling for API authentication
- `urlencoding` - URL encoding utilities

## Testing Strategy

### API Tests
```bash
# Test authentication
mmc login

# Test product operations
mmc add 91831A030
mmc info 91831A030
mmc price 91831A030
```

## Development Notes

### Code Quality
- Clean compilation without warnings
- Modular design for maintainability
- Comprehensive error handling
- XDG-compliant configuration paths

### Architecture Principles
- Separation of concerns across modules
- Type-safe API interactions
- Configurable output formats

## Troubleshooting

### Certificate Issues
- Ensure certificate is in PKCS12 format (.pfx/.p12)
- Check certificate password in credentials file
- Verify certificate is in default location

### API Errors
- Confirm valid McMaster-Carr API credentials
- Check network connectivity
- Verify product is in subscription (use `mmc add`)

### Build Issues
- Run `cargo clean` then `cargo build --release`
- Update Rust toolchain if needed
- Check dependency versions with `cargo tree`
