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

        // Check if column exists
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
