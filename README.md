# McMaster-Carr CLI (mmc)

A command-line interface for the McMaster-Carr Product Information API. This tool allows you to authenticate, manage product subscriptions, and retrieve product data from McMaster-Carr's API using client certificate authentication.

## Features

- 🔐 **Client Certificate Authentication** - Secure authentication using PKCS12 certificates
- 📁 **XDG Standard Configuration** - Follows XDG Base Directory Specification
- 🔑 **Token Management** - Automatic token storage and reuse (24-hour validity)
- 📦 **Product Management** - Add/remove products from subscription
- 💰 **Product Information** - Get detailed product data and pricing
- 📊 **Change Tracking** - Monitor product updates and changes
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

# Remove product from subscription
mmc remove 90128a211
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

# Generate human-readable part name
mmc name 98164A133

# List recent changes (requires start date)
mmc changes -s "01/01/2024"

# List changes from a specific date with time
mmc changes -s "08/20/2025 10:30"
```

## Name Generation

McMaster-Carr CLI can generate human-readable, abbreviated technical names for parts. This makes parts easier to remember, communicate, and use in BOMs or CAD systems.

### Usage

```bash
# Generate abbreviated technical name for any part
mmc name 98164A133
# Output: BHS-SS316-8x32-0.25-HEX

mmc name 90480A005  
# Output: HEXNUT-Steel-4x40-ZP
```

### Supported Categories

#### Screws & Bolts

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Button Head Screw | `BHS-[Material]-[Thread]-[Length]-[Drive]-[Finish]` | 316 SS Button Head Hex, 8x32 x 0.25" | `BHS-SS316-8x32-0.25-HEX` |
| Socket Head Screw | `SHS-[Material]-[Thread]-[Length]-[Drive]-[Finish]` | Steel Socket Head Hex, 1/4x20 x 1" | `SHS-Steel-1/4x20-1-HEX` |
| Flat Head Screw | `FHS-[Material]-[Thread]-[Length]-[Drive]-[Finish]` | 18-8 SS Flat Head Phillips, M6x1.0 x 20mm | `FHS-SS188-M6x1.0-20-PH` |
| Pan Head Screw | `PHS-[Material]-[Thread]-[Length]-[Drive]-[Finish]` | Brass Pan Head Phillips, 6x32 x 0.5" | `PHS-Brass-6x32-0.5-PH` |
| Hex Head Screw | `HHS-[Material]-[Thread]-[Length]-[Drive]-[Finish]` | SS Hex Head Screw, 1/4x20 x 1" | `HHS-SS-1/4x20-1-EHEX` |
| Rounded Head Screw | `RHS-[Material]-[Thread]-[Length]-[Drive]-[Finish]` | Steel Rounded Head Phillips, 8x32 x 0.5" | `RHS-Steel-8x32-0.5-PH` |
| Thumb Screw | `THUMB-[Material]-[Thread]-[Length]-[Finish]` | Brass Thumb Screw, M6x1.0 x 20mm | `THUMB-Brass-M6x1.0-20` |
| Eye Screw | `EYE-[Material]-[Thread]-[Length]-[Finish]` | Steel Eye Screw, 1/4x20 x 2" | `EYE-Steel-1/4x20-2` |
| Hook Screw | `HOOK-[Material]-[Thread]-[Length]-[Finish]` | SS Hook Screw, 8x32 x 1" | `HOOK-SS-8x32-1` |

*Note: Supports 20+ head types including T-Handle, Pentagon, Oval, Square, Knob, Ring, and specialty types. See code for complete list.*

| Generic Screw | `SCREW-[Material]-[Thread]-[Length]` | Brass Machine Screw, 6x32 x 0.5" | `SCREW-Brass-6x32-0.5` |

#### Nuts

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Locknut | `LOCKNUT-[Material]-[Thread]-[Finish]` | 18-8 SS Nylon-Insert Locknut, 4x40 | `LOCKNUT-SS188-4x40` |
| Hex Nut | `HEXNUT-[Material]-[Thread]-[Finish]` | 316 SS Hex Nut, 1/4x20, Zinc-Plated | `HEXNUT-SS316-1/4x20-ZP` |
| Wing Nut | `WINGNUT-[Material]-[Thread]-[Finish]` | Brass Wing Nut, 8x32 | `WINGNUT-Brass-8x32` |
| Cap Nut | `CAPNUT-[Material]-[Thread]-[Finish]` | SS Cap Nut, M8x1.25 | `CAPNUT-SS-M8x1.25` |
| Generic Nut | `NUT-[Material]-[Thread]-[Finish]` | Steel Nut, 5/16x18 | `NUT-Steel-5/16x18` |

#### Washers

McMaster-Carr CLI supports 19 different washer types with specific naming patterns:

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Cup Washer | `CW-[Material]-[Screw Size]-[Finish]` | 316 SS Cup Washer for 1/4" Screws | `CW-SS316-1/4` |
| Curved Washer | `CRVW-[Material]-[Screw Size]-[Finish]` | Steel Curved Washer for 8x32 | `CRVW-Steel-8x32` |
| Dished Washer | `DW-[Material]-[Screw Size]-[Finish]` | Brass Dished Washer for M6 | `DW-Brass-M6` |
| Domed Washer | `DMW-[Material]-[Screw Size]-[Finish]` | 18-8 SS Domed Washer for 1/4x20 | `DMW-SS188-1/4x20` |
| Double Clipped Washer | `DCW-[Material]-[Screw Size]-[Finish]` | Steel Double Clipped for #10 | `DCW-Steel-10` |
| Clipped Washer | `CLW-[Material]-[Screw Size]-[Finish]` | Steel Clipped Washer for 5/16" | `CLW-Steel-5/16` |
| Flat Washer | `FW-[Material]-[Screw Size]-[Finish]` | 316 SS Flat Washer for 1/4" | `FW-SS316-1/4` |
| Hillside Washer | `HW-[Material]-[Screw Size]-[Finish]` | Steel Hillside Washer for M8 | `HW-Steel-M8` |
| Notched Washer | `NW-[Material]-[Screw Size]-[Finish]` | Aluminum Notched Washer for 6x32 | `NW-Al-6x32` |
| Perforated Washer | `PW-[Material]-[Screw Size]-[Finish]` | SS Perforated Washer for 1/2" | `PW-SS-1/2` |
| Pronged Washer | `PRW-[Material]-[Screw Size]-[Finish]` | Steel Pronged Washer for M5 | `PRW-Steel-M5` |
| Rectangular Washer | `RW-[Material]-[Screw Size]-[Finish]` | Nylon Rectangular for 10x24 | `RW-Nylon-10x24` |
| Sleeve Washer | `SW-[Material]-[Screw Size]-[Finish]` | Brass Sleeve Washer for 1/4" | `SW-Brass-1/4` |
| Slotted Washer | `SLW-[Material]-[Screw Size]-[Finish]` | SS Slotted Washer for M6 | `SLW-SS-M6` |
| Spherical Washer | `SPW-[Material]-[Screw Size]-[Finish]` | Steel Spherical for 5/16" | `SPW-Steel-5/16` |
| Split Washer (Lock) | `SPLW-[Material]-[Screw Size]-[Finish]` | 18-8 SS Split Lock for 8x32 | `SPLW-SS188-8x32` |
| Square Washer | `SQW-[Material]-[Screw Size]-[Finish]` | Steel Square Washer for M8 | `SQW-Steel-M8` |
| Tab Washer | `TW-[Material]-[Screw Size]-[Finish]` | SS Tab Washer for 1/4x20 | `TW-SS-1/4x20` |
| Tapered Washer | `TPW-[Material]-[Screw Size]-[Finish]` | Steel Tapered for 3/8" | `TPW-Steel-3/8` |
| Tooth Washer | `TOW-[Material]-[Screw Size]-[Finish]` | SS Tooth Lock Washer for M10 | `TOW-SS-M10` |
| Wave Washer | `WW-[Material]-[Screw Size]-[Finish]` | Spring Steel Wave for 1/4" | `WW-Steel-1/4` |
| Wedge Washer | `WDW-[Material]-[Screw Size]-[Finish]` | Steel Wedge Washer for M12 | `WDW-Steel-M12` |

*Note: The system automatically detects washer type from the family description and applies the appropriate template. If no specific type is detected, it defaults to flat washer naming.*

#### Material Abbreviations

| Full Name | Abbreviation | Notes |
|-----------|--------------|-------|
| 316 Stainless Steel | `SS316` | Marine grade, high corrosion resistance |
| 18-8 Stainless Steel | `SS188` | Standard grade, good corrosion resistance |
| Stainless Steel (generic) | `SS` | When specific grade not specified |
| Steel | `Steel` | Carbon/alloy steel |
| Brass | `Brass` | Brass alloy |
| Aluminum | `Al` | Aluminum alloy |
| Copper | `Cu` | Copper alloy |
| Nylon | `Nylon` | Nylon plastic |
| Plastic | `Plastic` | Various plastic materials |
| Rubber | `Rubber` | Rubber materials |

#### Finish Abbreviations

| Full Name | Abbreviation | Notes |
|-----------|--------------|-------|
| Zinc Plated | `ZP` | Standard zinc coating |
| Zinc Yellow-Chromate Plated | `ZYC` | Zinc with yellow chromate |
| Black Oxide | `BO` | Black oxide coating |
| Cadmium Plated | `CD` | Cadmium coating |
| Nickel Plated | `NI` | Nickel coating |
| Chrome Plated | `CR` | Chrome coating |
| Galvanized | `GAL` | Hot-dip galvanized |
| Passivated | `PASS` | Omitted in names (not meaningful info) |

#### Drive Style Abbreviations

| Full Name | Abbreviation | Notes |
|-----------|--------------|-------|
| External Hex | `EHEX` | External hex head |
| Hex | `HEX` | Internal hex (Allen/socket) |
| Phillips | `PH` | Phillips head |
| Torx | `TX` | Standard Torx |
| Torx Plus | `TXP` | Torx Plus drive |
| Slotted | `SL` | Flat/slotted drive |
| Square | `SQUARE` | Robertson/square drive |
| Tamper-Resistant Hex | `TRHEX` | Security hex |
| Tamper-Resistant Torx | `TRTX` | Security Torx |
| Pozidriv® | `PZ` | Pozidriv drive |

*Note: McMaster-Carr supports 40+ drive styles. See code for complete list.*

### Dimension Formatting

- **Imperial Lengths**: Fractions automatically converted to decimals (`1/4"` → `0.25`)
- **Metric Lengths**: mm suffix removed (`20mm` → `20`)
- **Thread Sizes**: Use "x" separator for size/pitch (`8-32` → `8x32`, `M3 x 0.50mm` → `M3x0.50`)
- **Washer Sizes**: Preserve fractions for screw compatibility (`1/4"` → `1/4`)
- **Quote Marks**: Removed for cleaner names (`"` removed)

### Fallback Naming

For unsupported categories, the system generates fallback names using:
- Key words from the family description
- Part number as suffix
- Example: `BALL-BEARING-STEEL-12345A678`

### Integration Examples

#### BOM Usage
```csv
Part Number,Description,Generated Name,Quantity
98164A133,316 SS Button Head Hex Drive Screw,BHS-SS316-8x32-0.25-HEX,10
90480A005,Low-Strength Steel Hex Nut,HEXNUT-Steel-4x40-ZP,10
```

#### Scripting
```bash
# Generate names for a list of parts
for part in 98164A133 90480A005; do
  echo "$part: $(mmc name $part)"
done

# Create part name lookup table
mmc name 98164A133 > part_names.txt
echo "98164A133 -> $(mmc name 98164A133)"
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
mmc product 90128a211
# Returns: specifications, CAD links, material properties, etc.

# Check pricing
mmc price 92141A008
# Returns: $1.53 per pack of 100 washers

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
| `product` | `/v1/products/*` | Get product information |
| `price` | `/v1/products/*/price` | Get product pricing |
| `changes` | `/v1/changes?start=MM/dd/yyyy` | Get change notifications since date |

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
├── main.rs          # CLI interface and command parsing
├── client.rs        # McMaster-Carr API client
└── lib.rs          # Library exports (if needed)
```

### Dependencies

- `clap` - Command line parsing
- `reqwest` - HTTP client with native-tls support
- `serde` - JSON/TOML serialization
- `tokio` - Async runtime
- `anyhow` - Error handling
- `native-tls` - TLS with client certificate support
- `dirs` - XDG standard directory locations
- `toml` - TOML file parsing
- `urlencoding` - URL parameter encoding

## API Integration

For API integration details, contact McMaster-Carr at: **eCommerce@mcmaster.com**

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

[Contributing guidelines here]