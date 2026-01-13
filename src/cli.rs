use anyhow::Result;
use clap::{Parser, Subcommand};
use csv::StringRecord;

use crate::error::{ColumnSuggestion, CsvpeekError, find_similar_column};

const MAIN_HELP: &str = r#"
EXAMPLES:
    csvp data.csv                    Show summary statistics
    csvp schema data.csv             Show schema information
    csvp data.csv -c "name,age"      Analyze specific columns
    csvp data.csv -w "age > 30"      Filter rows before analysis
    csvp data.csv -d ";" -e sjis     Semicolon-delimited, Shift_JIS encoding

OUTPUT FORMATS:
    -f table    Pretty table (default)
    -f json     JSON format
    -f ndjson   Newline-delimited JSON
    -f csv      CSV format

For detailed help on specific topics, use:
    csvp guide filters    Filter expression syntax
    csvp guide stats      Available statistics
    csvp guide columns    Column specification syntax
    csvp guide formats    Output format details
"#;

#[derive(Parser, Debug)]
#[command(name = "csvp")]
#[command(author, version, about = "Fast CSV insights from the command line")]
#[command(after_long_help = MAIN_HELP)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// CSV file path
    #[arg(global = true)]
    pub file: Option<String>,

    /// Field delimiter character
    #[arg(long, short = 'd', global = true, default_value = ",")]
    pub delimiter: String,

    /// CSV has no header row (columns will be named col0, col1, ...)
    #[arg(long, global = true, default_value = "false")]
    pub no_header: bool,

    /// Output file path (default: stdout)
    #[arg(long, short = 'o', global = true)]
    pub output: Option<String>,

    /// Suppress progress display
    #[arg(long, short = 'q', global = true, default_value = "false")]
    pub quiet: bool,

    /// Color output control (auto, always, never)
    #[arg(long, global = true, default_value = "auto")]
    pub color: String,

    /// Character encoding (auto-detect if not specified)
    /// Supported: utf-8, shift_jis, euc-jp, gbk, big5, latin1, etc.
    #[arg(long, short = 'e', global = true)]
    pub encoding: Option<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display summary statistics for columns (default)
    Summary(SummaryArgs),

    /// Display schema information (column names, types, null rates)
    Schema(SchemaArgs),

    /// Show detailed help for a specific topic
    Guide(GuideArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct GuideArgs {
    /// Topic to display help for (filters, stats, columns, formats, encoding)
    #[arg(value_name = "TOPIC")]
    pub topic: Option<String>,
}

const SUMMARY_HELP: &str = r#"
STATISTICS COMPUTED:
    Numeric columns: count, null%, unique, min, max, mean, median, std, p25, p75
    String columns:  count, null%, unique, min_len, max_len, top values

COLUMN SELECTION (-c):
    -c "name,age"       By column names
    -c "0,1,2"          By index (0-based)
    -c "0..5"           Range (exclusive end)
    -c "0..=5"          Range (inclusive end)

FILTER EXPRESSIONS (-w):
    Comparison: age > 30, name == "Alice", price <= 100
    Logic:      age > 20 && age < 30, status == "A" || status == "B"
    Functions:  contains(name, "test"), is_null(email), matches(id, "^A\\d+")

EXAMPLES:
    csvp summary data.csv -c "0..5" -w "status == \"active\""
    csvp data.csv -w "price > 100 && is_not_null(discount)"

Run 'csvp guide filters' for complete filter syntax reference.
"#;

#[derive(Parser, Debug, Default, Clone)]
#[command(after_long_help = SUMMARY_HELP)]
pub struct SummaryArgs {
    /// Comma-separated list of columns to analyze (names, indices, or ranges)
    #[arg(long, short = 'c')]
    pub cols: Option<String>,

    /// Filter expression (e.g., "age > 30", "name == \"Alice\"")
    #[arg(long = "where", short = 'w')]
    pub where_clause: Option<String>,

    /// Output format (table, json, ndjson, csv)
    #[arg(long, short = 'f')]
    pub format: Option<String>,
}

const SCHEMA_HELP: &str = r#"
SCHEMA INFORMATION:
    column      Column name from header (or col0, col1... if --no-header)
    type        Inferred type: Integer, Float, Boolean, or String
    null%       Percentage of null/empty values
    samples     First few unique non-null values

TYPE INFERENCE:
    Integer     All non-null values are integers
    Float       Values contain decimals (or mix of int/float)
    Boolean     All values are true/false (case-insensitive)
    String      Any other values

EXAMPLES:
    csvp schema data.csv              Table format
    csvp schema data.csv -f json      JSON format for programmatic use
    csvp schema data.csv -f csv       CSV format for export
"#;

#[derive(Parser, Debug, Default, Clone)]
#[command(after_long_help = SCHEMA_HELP)]
pub struct SchemaArgs {
    /// Output format (table, json, ndjson, csv)
    #[arg(long, short = 'f')]
    pub format: Option<String>,
}

pub fn parse_columns(cols_str: &str, headers: &StringRecord) -> Result<Vec<String>> {
    let header_vec: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    let mut result = Vec::new();

    for col in cols_str.split(',') {
        let col = col.trim();
        if col.is_empty() {
            continue;
        }

        // Check for range syntax (e.g., 0..5 or 0..=5)
        if let Some(range_cols) = parse_range(col, &header_vec)? {
            result.extend(range_cols);
            continue;
        }

        // Try to parse as index first
        if let Ok(idx) = col.parse::<usize>() {
            if idx < header_vec.len() {
                result.push(header_vec[idx].clone());
                continue;
            } else {
                return Err(CsvpeekError::ColumnIndexOutOfRange {
                    index: idx,
                    max: header_vec.len() - 1,
                }
                .into());
            }
        }

        // Check if column name exists
        if header_vec.iter().any(|h| h == col) {
            result.push(col.to_string());
        } else {
            // Try to find a suggestion
            let suggestion = find_similar_column(col, &header_vec);
            return Err(CsvpeekError::ColumnNotFound {
                name: col.to_string(),
                suggestion: suggestion.map(|s| ColumnSuggestion { suggested: s }),
            }
            .into());
        }
    }

    Ok(result)
}

fn parse_range(s: &str, headers: &[String]) -> Result<Option<Vec<String>>> {
    // Check for inclusive range (0..=5)
    if let Some((start_str, end_str)) = s.split_once("..=") {
        let start: usize = start_str.trim().parse().map_err(|_| {
            CsvpeekError::InvalidFilter(format!("Invalid range start: {}", start_str))
        })?;
        let end: usize = end_str
            .trim()
            .parse()
            .map_err(|_| CsvpeekError::InvalidFilter(format!("Invalid range end: {}", end_str)))?;

        if end >= headers.len() {
            return Err(CsvpeekError::ColumnIndexOutOfRange {
                index: end,
                max: headers.len() - 1,
            }
            .into());
        }

        let cols: Vec<String> = (start..=end).map(|i| headers[i].clone()).collect();
        return Ok(Some(cols));
    }

    // Check for exclusive range (0..5)
    if let Some((start_str, end_str)) = s.split_once("..") {
        let start: usize = start_str.trim().parse().map_err(|_| {
            CsvpeekError::InvalidFilter(format!("Invalid range start: {}", start_str))
        })?;
        let end: usize = end_str
            .trim()
            .parse()
            .map_err(|_| CsvpeekError::InvalidFilter(format!("Invalid range end: {}", end_str)))?;

        if end > headers.len() {
            return Err(CsvpeekError::ColumnIndexOutOfRange {
                index: end - 1,
                max: headers.len() - 1,
            }
            .into());
        }

        let cols: Vec<String> = (start..end).map(|i| headers[i].clone()).collect();
        return Ok(Some(cols));
    }

    Ok(None)
}
