# csvpeek

Fast CSV insights from the command line.

## Features

- **Summary statistics** - Min, max, mean, median, standard deviation for numeric columns
- **Schema analysis** - Column names, types, null rates
- **Filtering** - Filter rows with `--where` expressions
- **Multiple encodings** - Auto-detect or specify encoding (UTF-8, Shift_JIS, EUC-JP, GBK, etc.)
- **Parallel processing** - Fast analysis using multiple CPU cores
- **Flexible output** - Table or JSON format

## Installation

### Using mise (recommended)

```bash
mise use "ubi:tact-software/csvpeek[exe=csvp]"
```

### From GitHub Releases

Download the appropriate binary for your platform from the [Releases](https://github.com/tact-software/csvpeek/releases) page.

| Platform | File |
|----------|------|
| Linux (x86_64) | `csvpeek-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (x86_64, musl) | `csvpeek-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `csvpeek-aarch64-unknown-linux-gnu.tar.gz` |
| macOS (Intel) | `csvpeek-x86_64-apple-darwin.tar.gz` |
| macOS (Apple Silicon) | `csvpeek-aarch64-apple-darwin.tar.gz` |
| Windows (x86_64) | `csvpeek-x86_64-pc-windows-msvc.zip` |

### From source

```bash
cargo install --git https://github.com/tact-software/csvpeek
```

## Usage

```bash
# Show summary statistics (default)
csvp data.csv

# Show schema information
csvp schema data.csv

# Analyze specific columns
csvp summary -c name,age,salary data.csv

# Filter rows
csvp summary --where "age > 30" data.csv

# Output as JSON
csvp summary -f json data.csv

# Specify encoding
csvp -e shift_jis data.csv
```

## Commands

### summary (default)

Display summary statistics for columns.

```
csvp summary [OPTIONS] [FILE]

Options:
  -c, --cols <COLS>       Comma-separated list of columns to analyze
  -w, --where <WHERE>     Filter expression
  -f, --format <FORMAT>   Output format (table, json)
```

### schema

Display schema information (column names, types, null rates).

```
csvp schema [OPTIONS] [FILE]

Options:
  -f, --format <FORMAT>   Output format (table, json)
```

## Global Options

```
  -d, --delimiter <CHAR>  Field delimiter [default: ,]
      --no-header         CSV has no header row
  -o, --output <FILE>     Output file path
  -q, --quiet             Suppress progress display
      --color <MODE>      Color output (auto, always, never)
  -e, --encoding <ENC>    Character encoding
```

## License

MIT OR Apache-2.0
