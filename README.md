# CSV Analyzer Tool

A Rust rewrite of the Pascal csvanalyzer tool from [mj-core-core](https://github.com/mailgun/mj-core-core) for analyzing CSV files for contact imports.

## Features

- Detects CSV format characteristics:
  - Header presence
  - Field separator (comma, semicolon, pipe, tab, etc.)
  - Text delimiter (quotes)
  - Character encoding/charset
  - Email column (mandatory)
  - Data types per column (string, integer, float, boolean, datetime)
  - Date/time format patterns
  - Decimal point format

- Matches CSV columns to contact properties via database metadata
- Returns JSON output matching the import batch job config format

## Usage

```bash
csvanalyzertool -a <akid> -f <csv_file> -l <locale> [-c <config_file>]
```

Arguments:
- `-a, --akid <AKID>` - Account ID (required)
- `-f, --file <FILE>` - CSV file path (required)
- `-l, --locale <LOCALE>` - Locale string (required, e.g., "en_US")
- `-c, --config <CONFIG>` - Config file path (optional)

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

## Building

```bash
cd csvanalyzer
cargo build --release
```

The binary will be at `target/release/csvanalyzertool`.

## Dependencies

- PostgreSQL client (for contact metadata queries)
- CSV parsing
- Charset detection
- Date/time format detection
