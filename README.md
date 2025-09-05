# McMaster-Carr CLI (mmc)

A command-line interface for the McMaster-Carr Product Information API. This tool allows you to authenticate, manage product subscriptions, retrieve product data, and generate human-readable technical names for fasteners and components using client certificate authentication.

## Features

- 🔐 **Client Certificate Authentication** - Secure authentication using PKCS12 certificates
- 📁 **XDG Standard Configuration** - Follows XDG Base Directory Specification
- 🔑 **Token Management** - Automatic token storage and reuse (24-hour validity)
- 📦 **Product Management** - Add/remove products from subscription
- 💰 **Product Information** - Get detailed product data and pricing
- 🏷️ **Flexible Name Generation** - Robust intelligent name generation with specification aliases
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

# Note: The 'name' command will automatically prompt to add unsubscribed products
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

# Analyze part specifications for debugging naming system
mmc analyze 90548A142

# Analyze with detailed template and alias information
mmc analyze 90548A142 --template --aliases

# Analyze with all details (equivalent to --template --aliases)
mmc analyze 90548A142 --all

# Get analysis output in JSON format (for automation)
mmc analyze 90548A142 --output json

# List recent changes (requires start date)
mmc changes -s "01/01/2024"

# List changes from a specific date with time
mmc changes -s "08/20/2025 10:30"
```

### Part Analysis and Debugging

McMaster-Carr CLI includes a powerful **analyze command** for debugging the naming system and understanding how parts are processed:

```bash
# Basic part analysis - shows specification mapping and generated name
mmc analyze 98164A133
# Output: Shows detected type, used/unused specs, template info, and suggestions

# Detailed analysis with template information
mmc analyze 98164A133 --template
# Output: Includes expected specifications and template details

# Analysis with specification aliases
mmc analyze 98164A133 --aliases  
# Output: Shows how field name variations are mapped

# Complete analysis (all details)
mmc analyze 98164A133 --all
# Output: Template info, aliases, name breakdown, and suggestions

# JSON output for automation and scripting
mmc analyze 98164A133 --output json
# Output: Machine-readable analysis data
```

#### Analysis Features

- **Part Detection**: Shows how the system identifies fastener types
- **Specification Mapping**: Displays which specs are used vs unused in names
- **Template Compatibility**: Shows expected vs available specifications  
- **Missing Spec Detection**: Identifies specs expected by template but not found
- **Name Breakdown**: Shows how each component maps back to specifications
- **Smart Suggestions**: Provides actionable recommendations for naming improvements
- **Finish Intelligence**: Suggests likely finishes based on material analysis
- **Alias Debugging**: Shows field name variations and mappings

#### Example Analysis Output

```
🔍 Part Analysis: 98164A133

📦 Product Info:
   Category: Screws And Bolts
   Family: Button Head Socket Screw
   Detected Type: button_head_screw

📋 Specifications (15 found):
   ✓ Material: 316 Stainless Steel → (used in name)
   ✓ Thread Size: 8-32 → (used in name)
   ✓ Length: 1/4" → (used in name)
   ✓ Drive Style: Hex → (used in name)
   ✗ Head Diameter: 0.312" → (not used)
   ✗ Thread Type: UNC → (not used)
   ...

🏷️ Template: button_head_screw
   Expected: [Material, Thread Size, Length, Drive Style, Finish]

🔧 Generated Name: BHS-SS316-8x32-0.25-HEX
   Components:
   - BHS: Template prefix (button_head_screw)
   - SS316: Material
   - 8x32: Thread Size
   - 0.25: Length
   - HEX: Drive Style

💡 Suggestions:
   🔍 Missing expected specifications: Finish
   📋 Unmapped specifications available: Head Diameter, Thread Type, ...
```

## Name Generation

McMaster-Carr CLI features a **flexible naming system** that generates human-readable, abbreviated technical names for parts. The system automatically adapts to McMaster-Carr's field variations and handles context-sensitive processing for different part types.

### Usage

```bash
# Generate abbreviated technical name for any part
mmc name 98164A133
# Output: BHS-SS316-8x32-0.25-HEX

mmc name 90480A005  
# Output: HN-S-4x40-ZP
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
| Captive Panel Screw | `CPS-[Material]-[Thread]-[Length]-[Drive]` | 400 Series SS Captive Panel, 10x32 x 0.41", Phillips | `CPS-SS400-10x32-0.41-PH` |

*Note: Supports 20+ head types including T-Handle, Pentagon, Oval, Square, Knob, Ring, and specialty types. See code for complete list.*

| Generic Screw | `SCREW-[Material]-[Thread]-[Length]` | Brass Machine Screw, 6x32 x 0.5" | `SCREW-Brass-6x32-0.5` |

#### Nuts

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Locknut | `LN-[Material]-[Thread]-[Finish]` | 18-8 SS Nylon-Insert Locknut, 4x40 | `LN-SS188-4x40` |
| Hex Nut | `HN-[Material]-[Thread]-[Finish]` | 316 SS Hex Nut, 1/4x20, Zinc-Plated | `HN-SS316-1/4x20-ZP` |
| Wing Nut | `WN-[Material]-[Thread]-[Finish]` | Brass Wing Nut, 8x32 | `WN-Brass-8x32` |
| Cap Nut | `CN-[Material]-[Thread]-[Finish]` | SS Cap Nut, M8x1.25 | `CN-SS-M8x1.25` |
| Generic Nut | `N-[Material]-[Thread]-[Finish]` | Steel Nut, 5/16x18 | `N-S-5/16x18` |

*Note: Supports 36+ nut types including Flange (FN), Socket (SN), Speed (SPEEDN), Square (SQN), and 11 specialized locking nut types. See code for complete list.*

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
| Notched Washer | `NW-[Material]-[Screw Size]-[Finish]` | Aluminum Notched Washer for 6x32 | `NW-AL-6x32` |
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

#### Threaded Standoffs

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Male-Female Standoff | `MFSO-[Material]-[Thread]-[Length]-[Finish]` | SS Male-Female Standoff, 4x40 x 0.5" | `MFSO-SS-4x40-0.5` |
| Female Standoff | `FSO-[Material]-[Thread]-[Length]-[Finish]` | Brass Female Standoff, M6x1.0 x 25mm | `FSO-Brass-M6x1.0-25` |
| Standoff (Generic) | `SO-[Material]-[Thread]-[Length]-[Finish]` | Aluminum Threaded Standoff, 8x32 x 0.75" | `SO-AL-8x32-0.75` |

*Note: Supports various standoff configurations including male-female, female-only, and specialized types for electronics and mechanical assemblies.*

#### Unthreaded Spacers

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Generic Spacer | `SP-[Material]-[Screw Size]-[OD]-[Length]-[Finish]` | Acetal Spacer, 1/4" screw, 1/2" OD, 2" long | `SP-ACET-0.25-0.5-2` |
| Aluminum Spacer | `SP-[Material]-[Screw Size]-[OD]-[Length]-[Finish]` | Aluminum Spacer, M5 screw, 8mm OD, 8mm long | `SP-AL-M5-8-8` |
| Stainless Steel Spacer | `SSSP-[Material]-[Screw Size]-[OD]-[Length]-[Finish]` | 18-8 SS Spacer, 5/16" screw, 5/8" OD, 1.5" long | `SSSP-SS188-5/16-0.625-1.5` |
| Nylon Spacer | `NSP-[Material]-[Screw Size]-[Length]` | Nylon Spacer, 1/8" screw, 0.5" long | `NSP-Nylon-0.125-0.5` |

*Note: Unthreaded spacers are distinguished from threaded standoffs by the absence of threading. They provide precise spacing between components without fastening capability.*

#### Clevis Pins

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Clevis Pin | `CP-[Material]-[Diameter]-[Usable Length]-[Finish]` | Steel Clevis Pin, 1/4" dia, 1.5" usable length | `CP-S-0.25-1.5` |
| Clevis Pin with Retaining Ring Groove | `CPRRG-[Material]-[Diameter]-[Usable Length]-[Finish]` | 18-8 SS Clevis Pin w/ RRG, 1/4" dia, 2-3/8" usable | `CPRRG-SS188-0.25-2.375` |

*Note: Clevis pins are used for pivot connections and removable mechanical linkages. Retaining ring groove variants include a groove for secure retention.*

#### Shaft Collars

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Face-Mount Shaft Collar | `FMSC-[Material]-[Shaft Dia]-[OD]-[Width]-[Finish]` | 303 SS Face-Mount, 1/2" shaft, 1-1/8" OD, 13/32" width | `FMSC-SS303-0.5-1.125-0.40625` |
| Flange-Mount Shaft Collar | `FLSC-[Material]-[Shaft Dia]-[OD]-[Width]-[Finish]` | 18-8 SS Flange-Mount, 3/8" shaft, 7/8" OD, 1/4" width | `FLSC-SS188-0.375-0.875-0.25` |

*Note: Shaft collars provide axial positioning and support for rotating shafts. Face-mount variants have tapped holes on the face, while flange-mount types have through-holes and counterbored mounting holes.*

#### Bearings

McMaster-Carr CLI provides comprehensive bearing support with specialized naming for different bearing types:

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Flanged Sleeve Bearing | `FSB-[Material]-[Shaft Diameter]-[OD]-[Length]` | MDS-Filled Nylon, 1/4" shaft, 3/8" OD, 1/4" long | `FSB-MDSNYL-0.25-0.375-0.25` |
| Plain Sleeve Bearing | `SB-[Material]-[Shaft Diameter]-[OD]-[Length]` | Bronze SAE 841, 3/8" shaft, 1/2" OD, 1/2" long | `SB-BR841-0.375-0.5-0.5` |
| Flanged Bearing (generic) | `FB-[Material]-[Shaft Diameter]-[OD]-[Length]` | Steel flanged bearing, 1/2" shaft, 5/8" OD | `FB-STL-0.5-0.625-0.5` |
| Ball Bearing | `BB-[Material]-[Bore]-[OD]` | Stainless steel ball bearing, 6mm bore, 19mm OD | `BB-SS-6-19` |
| Linear Bearing | `LB-[Material]-[Shaft Diameter]-[Length]` | Steel linear bearing for 3/8" shaft, 2" long | `LB-STL-0.375-2` |
| Needle Bearing | `NB-[Material]-[Bore]-[OD]-[Length]` | Steel needle bearing, 1/4" bore, 3/8" OD | `NB-STL-0.25-0.375-0.5` |
| Roller Bearing | `RB-[Material]-[Bore]-[OD]-[Length]` | Bronze roller bearing, 20mm bore, 35mm OD | `RB-BR-20-35-12` |
| Flange Mounted Ball Bearing | `MFBB-[Housing Material]-[Shaft Dia]-[Mount Holes C-to-C]-[Height]` | Steel flange mount, 3/4" shaft, 2-1/2" hole spacing, 2" height | `MFBB-STL-0.75-2.5-2` |
| Low-Profile Flange Mounted Ball Bearing | `LPMFBB-[Housing Material]-[Shaft Dia]-[Mount Holes C-to-C]-[Height]` | Steel low-profile, 1" shaft, 3" hole spacing, 2.78" height | `LPMFBB-STL-1-3-2.78125` |
| Pillow Block Mounted Ball Bearing | `PBMBB-[Housing Material]-[Shaft Dia]-[Mount Holes C-to-C]-[Height]` | Steel pillow block, 1/2" shaft, 4" hole spacing | `PBMBB-STL-0.5-4-1.5` |
| Generic Mounted Bearing | `MBB-[Housing Material]-[Shaft Dia]-[Height]` | Steel mounted bearing for 5/8" shaft | `MBB-STL-0.625-2` |
| Generic Bearing | `BRG-[Material]-[Type]` | PTFE bearing assembly | `BRG-PTFE-ASSEMBLY` |

**Special Features:**
- **Automatic Material Detection**: Combines filler materials (e.g., MDS-Filled Nylon)
- **Dimension Conversion**: Fractions automatically converted to decimals (1/4" → 0.25)
- **Metric Support**: Handles both imperial and metric dimensions
- **Mounted Bearing Support**: Includes housing material, mounting dimensions, and profile variations
- **Comprehensive Coverage**: Supports plain, flanged, ball, linear, needle, roller, and mounted bearings

**Bearing Material Abbreviations:**

| Full Name | Abbreviation | Applications |
|-----------|--------------|-------------|
| MDS-Filled Nylon Plastic | `MDSNYL` | Dry-running, self-lubricating applications |
| Nylon Plastic | `NYL` | Light-duty, corrosion-resistant |
| Bronze SAE 841 | `BR841` | Oil-impregnated, general purpose |
| Bronze SAE 863 | `BR863` | High-load applications |
| Cast Bronze | `CB` | Heavy-duty applications |
| Oil-Filled Bronze | `OFB` | Self-lubricating bronze |
| PTFE | `PTFE` | Chemical resistance, low friction |
| Rulon | `RUL` | Dry-running plastic bearing |
| Graphite | `GRAPH` | High-temperature applications |
| Steel-Backed PTFE | `SBPTFE` | High-load PTFE applications |
| Bronze (generic) | `BR` | General bronze bearings |
| Steel | `STL` | High-strength applications |
| Stainless Steel | `SS` | Corrosion-resistant steel |

*Note: The system automatically detects bearing type from product specifications and applies the appropriate template. Filler materials are automatically combined with base materials for accurate naming.*

#### Pulleys

McMaster-Carr CLI provides comprehensive pulley support for various rope, wire rope, and belt applications:

| Type | Template | Example Input | Generated Name |
|------|----------|---------------|----------------|
| Wire Rope Pulley | `WRP-[Material]-[Rope Diameter]-[OD]-[Bearing Type]` | Steel wire rope pulley, 3/16" rope, 1-1/4" OD, ball bearing | `WRP-S-0.1875-1.25-BALL` |
| Generic Rope Pulley | `RP-[Material]-[Rope Diameter]-[OD]-[Bearing Type]` | SS rope pulley, 1/4" rope, 2" OD, plain bearing | `RP-SS-0.25-2-PLAIN` |
| V-Belt Pulley | `VBP-[Material]-[Belt Width]-[OD]-[Bearing Type]` | Aluminum V-belt pulley, 1/2" belt, 3" OD | `VBP-AL-0.5-3-NONE` |
| Generic Pulley | `PUL-[Material]-[OD]-[Bearing Type]` | Bronze pulley, 4" OD, roller bearing | `PUL-BR-4-ROLLER` |
| Sheave | `SHV-[Material]-[Rope Diameter]-[OD]-[Bearing Type]` | Steel sheave for lifting, 5/16" rope, 6" OD | `SHV-S-0.3125-6-BALL` |

**Pulley Material Abbreviations:**

| Full Name | Abbreviation | Applications |
|-----------|--------------|-------------|
| Steel | `S` | General purpose, high strength |
| Stainless Steel | `SS` | Corrosion resistant applications |
| 303 Stainless Steel | `SS303` | Machined stainless components |
| 316 Stainless Steel | `SS316` | Marine and chemical environments |
| Aluminum | `AL` | Lightweight applications |
| Bronze | `BR` | Corrosion resistant, heavy duty |
| Cast Iron | `CI` | Industrial, heavy-duty applications |
| Plastic | `PL` | Lightweight, chemical resistant |
| Nylon | `NYL` | Self-lubricating, quiet operation |

**Bearing Type Abbreviations:**

| Full Name | Abbreviation | Applications |
|-----------|--------------|-------------|
| Ball | `BALL` | Low friction, general purpose |
| Plain | `PLAIN` | Simple, maintenance-free |
| Roller | `ROLLER` | Heavy load applications |
| None | `NONE` | Direct shaft mounting |

**Application Categories:**

| Application | Abbreviation | Usage |
|------------|--------------|-------|
| For Pulling | `PULL` | Horizontal load applications |
| For Lifting | `LIFT` | Vertical load applications |
| For Horizontal Pulling | `HPULL` | Specific horizontal applications |

**Key Features:**
- **Automatic Type Detection**: Distinguishes between wire rope, rope, V-belt, and generic pulleys
- **Bearing Integration**: Includes bearing type in naming for maintenance planning
- **Dimensional Precision**: Rope/belt diameters and pulley OD for accurate specifications
- **Application Context**: Recognizes lifting vs pulling applications
- **Sheave Recognition**: Handles alternative pulley terminology used in rigging

*Note: The system automatically detects pulley type from family description and applies the appropriate template. Sheaves are treated as specialized pulleys for lifting applications.*

#### Material Abbreviations

| Full Name | Abbreviation | Notes |
|-----------|--------------|-------|
| 316 Stainless Steel | `SS316` | Marine grade, high corrosion resistance |
| 400 Series Stainless Steel | `SS400` | Magnetic stainless, good corrosion resistance |
| 18-8 Stainless Steel | `SS188` | Standard grade, good corrosion resistance |
| 303 Stainless Steel | `SS303` | Free-machining stainless steel |
| Stainless Steel (generic) | `SS` | When specific grade not specified |
| 1004-1045 Carbon Steel | `S` | Low to medium carbon steel |
| Grade 1 Steel | `SG1` | Low carbon steel |
| Grade 2 Steel | `SG2` | Low carbon steel |
| Grade 5 Steel | `SG5` | Medium carbon steel |
| Grade 8 Steel | `SG8` | High strength alloy steel |
| Class 8.8 Steel | `S8.8` | Metric medium strength |
| Class 10.9 Steel | `S10.9` | Metric high strength |
| Class 12.9 Steel | `S12.9` | Metric very high strength |
| 1215 Carbon Steel | `1215S` | Free-machining carbon steel |
| Steel (generic) | `S` | Carbon/alloy steel when grade not specified |
| Brass | `Brass` | Brass alloy |
| Aluminum | `AL` | Aluminum alloy |
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
90480A005,Low-Strength Steel Hex Nut,HN-S-4x40-ZP,10
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
mmc info 90128a211
# Returns: specifications, CAD links, material properties, etc.

# Check pricing
mmc price 92141A008
# Returns: pricing information per unit/pack

# Generate technical names (prompts to add if not subscribed)
mmc name 92141A008
# If not subscribed, prompts: "❌ Product 92141A008 is not in your subscription. Would you like to add it to your subscription? (Y/n):"
# Returns: FW-SS188-6 (Flat Washer, 18-8 SS, #6 screw size)

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
├── lib.rs                    # Library root with exports
├── main.rs                   # CLI entry point
├── client/                   # API client functionality
│   ├── mod.rs               # Module declarations
│   ├── api.rs               # Core API operations
│   ├── auth.rs              # Authentication handling
│   └── downloads.rs         # File downloads
├── naming/                   # Name generation system
│   ├── mod.rs               # Module declarations
│   ├── generator.rs         # Core name generation logic
│   ├── abbreviations.rs     # Value abbreviation logic
│   ├── converters.rs        # Data conversion utilities
│   ├── detectors.rs         # Fastener type detection
│   └── templates/           # Naming templates by category
│       ├── mod.rs
│       ├── screws.rs        # Screw naming templates
│       ├── nuts.rs          # Nut naming templates
│       ├── washers.rs       # Washer naming templates
│       ├── standoffs.rs     # Standoff naming templates
│       └── bearings.rs      # Bearing naming templates
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
- `regex` - Pattern matching for naming system

## API Integration

For API integration details, contact McMaster-Carr at: **eCommerce@mcmaster.com**

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

[Contributing guidelines here]