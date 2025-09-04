# McMaster-Carr CLI (mmcli) - Claude Configuration

## Project Overview

A comprehensive command-line interface for interacting with McMaster-Carr's Product Information API, featuring intelligent name generation for fasteners and components.

## Quick Commands

### Build & Test
```bash
# Build release version
cargo build --release

# Run specific functionality tests
./target/release/mmc name 91831A030  # Test locknut with thread pitch
./target/release/mmc name 92141A008  # Test washer with screw size
./target/release/mmc name 91780A053  # Test aluminum abbreviation
./target/release/mmc name 98164A133  # Test screw naming
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

## Key Features

### 1. Flexible Naming System
- **100% Success Rate**: All inventory parts generate correct names
- **Specification Aliases**: Handles field name variations (e.g., "Thread Size" vs "Thread (A) Size")
- **Context-Sensitive Processing**: Same fields processed differently by part type (washers vs spacers)
- **Enhanced Finish Extraction**: Works with both explicit fields and embedded material finishes
- **Robust Pattern Matching**: Specific helper functions prevent false matches
- **Template-based**: Each fastener type has specific naming templates
- **Multiple Categories**: Screws, nuts, washers, standoffs, bearings, spacers, pins, shaft collars
- **Smart abbreviations**: Material, thread size, dimensions automatically abbreviated
- **Features**: Thread pitch extraction, screw size handling, standardized material abbreviations

### 2. API Integration
- **Certificate-based auth**: Uses PKCS12 client certificates
- **Token management**: Automatic token loading/saving
- **Full CRUD**: Add/remove products, get info/pricing, download files

### 3. Output Formats
- **Human-readable**: Formatted with emojis and clear descriptions
- **JSON**: Machine-readable for automation
- **Field selection**: Choose specific product attributes

## Example Outputs

The naming system generates concise, technical part names:

```bash
# Locknut with thread pitch
LN-SS188-5/16x18

# Washer with clean screw size  
FW-SS188-6

# Aluminum standoff
FSO-AL-4X40-3.5

# Screw with all specifications
BHS-SS316-8X32-0.25-HEX
```

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

### Name Generation
```bash
# Generate human-readable name
mmc name 91831A030

# Works for all fastener types
mmc name 98164A133  # Screw
mmc name 92141A008  # Washer  
mmc name 91780A053  # Standoff
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

## Dependencies

### Key Dependencies
- `clap` - CLI argument parsing with derive macros
- `tokio` - Async runtime for HTTP operations
- `reqwest` - HTTP client with TLS support
- `dirs` - Cross-platform directory paths
- `toml` - Configuration file parsing
- `regex` - Pattern matching for naming system
- `native-tls` - TLS/certificate handling for API authentication

## Testing Strategy

### Functional Tests
```bash
# Test naming system improvements
./target/release/mmc name 91831A030  # Thread pitch: 5/16x18
./target/release/mmc name 92141A008  # Screw size: 6 (not NO.6)  
./target/release/mmc name 91780A053  # Aluminum: AL (not Al)
```

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
- Template-driven naming system for extensibility
- Separation of concerns across modules
- Type-safe API interactions
- Configurable output formats

## Flexible Naming System Architecture

### Overview
The naming system has been completely redesigned to achieve **100% success rate** on all inventory parts while making the system robust and maintainable.

### Core Features

#### 1. Specification Aliases
Templates can define multiple possible field names for each specification:
```rust
// Template expects "Thread Size" but part may have variations
spec_aliases: Some({
    "Thread Size" -> ["Thread Size", "Thread (A) Size", "Thread (B) Size"]
})
```

**Benefits:**
- Handles McMaster-Carr field variations automatically
- Self-healing system adapts to new field names
- Backward compatible with existing templates

#### 2. Context-Sensitive Processing
Same field names get different processing based on part type:
```rust
// "For Screw Size" field:
Washers: "No. 6" → "6" (preserve screw size format)
Spacers: "1/4"" → "0.25" (convert to decimal dimensions)
```

**Implementation:** Uses template category detection, not brittle prefix matching.

#### 3. Enhanced Finish Extraction
Works with both explicit finish fields AND embedded finish in materials:
```rust
// Part has no "Finish" field but material contains finish info:
Material: "Zinc-Plated Alloy Steel"
→ Extracts: finish="Zinc-Plated", material="Alloy Steel" 
→ Result: "SHS-S12.9-M4x0.7-8-HEX-ZP"
```

#### 4. Robust Pattern Matching
Replaced brittle string matching with specific helper functions:
- `is_thread_size_field()` - exact thread size patterns
- `is_diameter_field()` - precise diameter detection  
- `is_material_field()` - extensible material patterns
- `is_washer_template()` - category-based type detection

### Success Examples

| Part Number | Issue | Solution | Result |
|------------|-------|----------|---------|
| 93505A443 | "Thread (A) Size" field not found | Specification aliases | `MFSO-AL-6x32-0.625` ✅ |
| 93717a451 | Using precise "ID" instead of functional size | Field priority aliases | `SP-ACET-0.25-0.5-2` ✅ |
| 9677T2 | Missing finish suffix from material | Enhanced finish extraction | `FMSC-1215S-1-1.75-0.5-BO` ✅ |
| 92141A008 | Washer screw size conversion issues | Context-sensitive processing | `FW-SS188-6` ✅ |

### Template Structure
```rust
NamingTemplate {
    prefix: "BHS".to_string(),                    // Part prefix
    key_specs: vec!["Material", "Thread Size"],   // Required specs
    spec_aliases: Some(aliases),                  // NEW: Field name variations
    spec_abbreviations: abbrevs,                  // Value abbreviations
}
```

### Backward Compatibility
- Existing templates work unchanged with `spec_aliases: None`
- All original functionality preserved
- No breaking changes to existing code
- Gradual migration path for enhanced features

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