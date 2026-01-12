mod cli;
mod csv_reader;
mod error;
mod filter;
mod output;
mod schema;
mod stats;
mod types;

use anyhow::Result;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    match &cli.command {
        Some(Commands::Summary(args)) => {
            commands::run_summary(&cli, args)?;
        }
        Some(Commands::Schema(args)) => {
            commands::run_schema(&cli, args)?;
        }
        None => {
            // Default to summary command
            commands::run_summary(&cli, &cli::SummaryArgs::default())?;
        }
    }

    Ok(())
}

mod commands {
    use super::*;
    use crate::csv_reader::CsvReader;
    use crate::filter::Filter;
    use crate::output::{OutputFormat, Renderer};
    use crate::schema::SchemaInferrer;
    use crate::stats::StatsCollector;

    pub fn run_summary(cli: &Cli, args: &cli::SummaryArgs) -> Result<()> {
        let file_path = cli.file.as_ref().ok_or_else(|| {
            anyhow::anyhow!("FILE is required")
        })?;

        let mut reader = CsvReader::from_path(file_path)?;
        let headers = reader.headers()?.clone();

        // Determine columns to process
        let target_cols = if let Some(ref cols) = args.cols {
            cli::parse_columns(cols, &headers)?
        } else {
            headers.iter().map(|s| s.to_string()).collect()
        };

        // Build filter if specified
        let filter = if let Some(ref where_clause) = args.where_clause {
            Some(Filter::parse(where_clause, &headers)?)
        } else {
            None
        };

        // Collect statistics
        let mut collector = StatsCollector::new(&target_cols, &headers);
        let mut total_rows = 0u64;
        let mut matched_rows = 0u64;

        for result in reader.records() {
            let record = result?;
            total_rows += 1;

            // Apply filter
            if let Some(ref f) = filter {
                if !f.matches(&record, &headers)? {
                    continue;
                }
            }

            matched_rows += 1;
            collector.add_record(&record, &headers)?;
        }

        let stats = collector.finalize();

        // Render output
        let format = args.format.as_deref().unwrap_or("table");
        let renderer = Renderer::new(OutputFormat::from_str(format)?);
        renderer.render_summary(
            file_path,
            total_rows,
            matched_rows,
            args.where_clause.as_deref(),
            &stats,
        )?;

        Ok(())
    }

    pub fn run_schema(cli: &Cli, _args: &cli::SchemaArgs) -> Result<()> {
        let file_path = cli.file.as_ref().ok_or_else(|| {
            anyhow::anyhow!("FILE is required")
        })?;

        let mut reader = CsvReader::from_path(file_path)?;
        let headers = reader.headers()?.clone();

        let mut inferrer = SchemaInferrer::new(&headers);

        for result in reader.records() {
            let record = result?;
            inferrer.add_record(&record)?;
        }

        let schema = inferrer.finalize();

        let renderer = Renderer::new(OutputFormat::Table);
        renderer.render_schema(file_path, &schema)?;

        Ok(())
    }
}
