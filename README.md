# CSV Analyzer Tool

[![CI](https://github.com/obennett-m/csvanalyzer/workflows/CI/badge.svg)](https://github.com/obennett-m/csvanalyzer/actions/workflows/ci.yml)
[![Release](https://github.com/obennett-m/csvanalyzer/workflows/Release%20Build/badge.svg)](https://github.com/obennett-m/csvanalyzer/actions/workflows/release.yml)

A Rust rewrite of the Pascal csvanalyzer tool from [mj-core-core](https://github.com/mailgun/mj-core-core) for analyzing CSV files for contact imports.

## Overview

csvanalyzertool automatically detects and analyzes CSV file characteristics for contact list imports. It identifies format settings, validates data, maps columns to contact properties, and returns structured JSON output ready for import processing.

## Features

### Format Detection
- **Delimiter detection**: Comma, semicolon, pipe, tab, and custom separators
- **Quote character detection**: Single and double quotes
- **Header detection**: Automatic identification of header rows
- **Character encoding**: Auto-detection and conversion (UTF-8, UTF-16, ISO-8859-1, Windows-1252, ANSI/ASCII, etc.)
- **Email column identification**: Required email field detection

### Data Analysis
- **Data type detection**: String, Integer, Float, Boolean, DateTime
- **DateTime format detection**: Multiple date/time patterns with RFC3339 support
- **Decimal format detection**: Comma vs period decimal separators
- **Column validation**: Max 200 columns, configurable string length limits
- **Database integration**: Maps CSV columns to contact properties via PostgreSQL

### Output
- **JSON format**: Structured output matching import batch job config
- **Sample data**: Returns first N rows (configurable, default 10)
- **Error handling**: Detailed error messages for troubleshooting
- **Validation**: Binary file detection, column count limits, required field checks

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform:

| Platform | Binary | Notes |
|----------|--------|-------|
| Linux x86_64 | [csvanalyzertool-linux-x86_64] | Dynamic linking (glibc) |
| Linux x86_64 (musl) | [csvanalyzertool-linux-x86_64-musl] | Static binary, no dependencies |
| macOS Intel | [csvanalyzertool-macos-x86_64] | Intel-based Macs |
| macOS Apple Silicon | [csvanalyzertool-macos-aarch64] | M1/M2/M3 Macs |
| Windows x86_64 | [csvanalyzertool-windows-x86_64.exe] | 64-bit Windows |

[csvanalyzertool-linux-x86_64]: https://github.com/mailgun/mj-core-core/releases/latest/download/csvanalyzertool-linux-x86_64
[csvanalyzertool-linux-x86_64-musl]: https://github.com/mailgun/mj-core-core/releases/latest/download/csvanalyzertool-linux-x86_64-musl
[csvanalyzertool-macos-x86_64]: https://github.com/mailgun/mj-core-core/releases/latest/download/csvanalyzertool-macos-x86_64
[csvanalyzertool-macos-aarch64]: https://github.com/mailgun/mj-core-core/releases/latest/download/csvanalyzertool-macos-aarch64
[csvanalyzertool-windows-x86_64.exe]: https://github.com/mailgun/mj-core-core/releases/latest/download/csvanalyzertool-windows-x86_64.exe

**Linux/macOS Installation:**
```bash
# Download and install
curl -LO https://github.com/mailgun/mj-core-core/releases/latest/download/csvanalyzertool-linux-x86_64
chmod +x csvanalyzertool-linux-x86_64
sudo mv csvanalyzertool-linux-x86_64 /usr/local/bin/csvanalyzertool

# Verify installation
csvanalyzertool --version
```

### Build from Source

```bash
# Clone repository
git clone https://github.com/mailgun/mj-core-core.git
cd mj-core-core/csvanalyzer

# Build release binary
cargo build --release
```

The binary will be at `target/release/csvanalyzertool`.

## Usage

### Basic Command

```bash
csvanalyzertool -a <akid> -f <csv_file> -l <locale> [-c <config_file>]
```

### Required Arguments

- `--akid`, `-a`: Account ID for database queries (integer)
- `--locale`, `-l`: User locale (e.g., "en_US", "fr_FR", "de_DE")
- `--filename`, `-f`: Path to CSV file to analyze

### Optional Arguments

- `--config`, `-c`: Path to config file (default: `/etc/mailjet.conf`)
- `--scan-lines`: Number of lines to scan (default: 1000)
- `--return-lines`: Number of sample rows to return (default: 10)

## Output

The tool outputs JSON to stdout. On success, it returns analysis results including:
- Detected format settings (separators, charset, etc.)
- Header names and field mappings
- Data types for each column
- Sample data (first 10 rows)

On error, it returns an error JSON object with error details.

## Feature Comparison

| Feature | Pascal | Rust |
|---------|--------|------|
| Max scan lines | 1000 | 1000 (configurable) |
| Max return lines | 10 | 10 (configurable) |
| Max columns | 200 | 200 |
| Max string size | 1000 | 1000 |
| Charset detection | charsetdetector lib | chardetng (Mozilla port) |
| Email-based delimiter detection | ✓ | ✓ |
| Frequency-based delimiter fallback | ✓ | ✓ |
| Quote character detection | ✓ | ✓ |
| Header detection | ✓ | ✓ |
| Column count validation | ✓ | ✓ |
| Data type detection | ✓ | ✓ |
| Boolean downgrade logic (MJAPP-2440) | ✓ | ✓ |
| DateTime format detection | ✓ | ✓ |
| RFC3339 support | ✓ | ✓ |
| Database property matching | ✓ | ✓ |
| Error JSON response | ✓ | ✓ |

## Development

### Building

```bash
cd csvanalyzer
cargo build --release
```

The binary will be at `target/release/csvanalyzertool`.

### Running Tests

```bash
cargo test
```

### Cross-Compilation

The project uses GitHub Actions to automatically build binaries for multiple platforms. See `.github/workflows/release.yml` for the build configuration.

To cross-compile locally:

```bash
# Install target
rustup target add x86_64-unknown-linux-musl

# Build for target
cargo build --release --target x86_64-unknown-linux-musl

# Binary location
ls target/x86_64-unknown-linux-musl/release/csvanalyzertool
```

## Dependencies

- PostgreSQL client (for contact metadata queries)
- CSV parsing
- Charset detection
- Date/time format detection

## Release Process

Releases are automatically created when a new tag is pushed:

```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

This triggers the GitHub Actions workflow that:
1. Builds binaries for all supported platforms
2. Creates SHA256 checksums for each binary
3. Creates a GitHub release with all artifacts
