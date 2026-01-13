use anyhow::Result;
use clap::{Parser, Subcommand};
use csv::StringRecord;

use crate::error::{find_similar_column, ColumnSuggestion, CsvpeekError};

#[derive(Parser, Debug)]
#[command(name = "csvp")]
#[command(author, version, about = "Fast CSV insights from the command line")]
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
}

#[derive(Parser, Debug, Default, Clone)]
pub struct SummaryArgs {
    /// Comma-separated list of columns to analyze
    #[arg(long, short = 'c')]
    pub cols: Option<String>,

    /// Filter expression
    #[arg(long = "where", short = 'w')]
    pub where_clause: Option<String>,

    /// Output format (table, json)
    #[arg(long, short = 'f')]
    pub format: Option<String>,
}

#[derive(Parser, Debug, Default, Clone)]
pub struct SchemaArgs {
    /// Output format (table, json)
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
        let end: usize = end_str.trim().parse().map_err(|_| {
            CsvpeekError::InvalidFilter(format!("Invalid range end: {}", end_str))
        })?;

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
        let end: usize = end_str.trim().parse().map_err(|_| {
            CsvpeekError::InvalidFilter(format!("Invalid range end: {}", end_str))
        })?;

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
