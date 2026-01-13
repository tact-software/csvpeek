mod cli;
mod csv_reader;
mod error;
mod filter;
mod guide;
mod output;
mod progress;
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
        Some(Commands::Guide(args)) => {
            guide::print_guide(args.topic.as_deref());
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
    use crate::csv_reader::{CsvOptions, CsvReader};
    use crate::filter::Filter;
    use crate::output::{ColorMode, OutputFormat, Renderer};
    use crate::progress::ProgressTracker;
    use crate::schema::SchemaInferrer;
    use crate::stats::StatsCollector;

    fn build_csv_options(cli: &Cli) -> CsvOptions {
        let delimiter = parse_delimiter(&cli.delimiter);
        CsvOptions::new()
            .with_delimiter(delimiter)
            .with_no_header(cli.no_header)
            .with_encoding(cli.encoding.clone())
    }

    fn parse_delimiter(s: &str) -> u8 {
        match s.to_lowercase().as_str() {
            "tab" | "\\t" | "\t" => b'\t',
            "comma" | "," => b',',
            "semicolon" | ";" => b';',
            "pipe" | "|" => b'|',
            "space" | " " => b' ',
            _ => s.as_bytes().first().copied().unwrap_or(b','),
        }
    }

    pub fn run_summary(cli: &Cli, args: &cli::SummaryArgs) -> Result<()> {
        let file_path = cli
            .file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("FILE is required"))?;

        let options = build_csv_options(cli);
        let mut reader = CsvReader::from_path_with_options(file_path, options)?;
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

        // Collect statistics with progress tracking
        let mut collector = StatsCollector::new(&target_cols, &headers);
        let mut total_rows = 0u64;
        let mut matched_rows = 0u64;
        let mut progress = ProgressTracker::new(file_path, cli.quiet);

        for result in reader.records() {
            let record = result?;
            total_rows += 1;

            // Update progress
            progress.update(&record);

            // Apply filter
            if let Some(ref f) = filter
                && !f.matches(&record, &headers)?
            {
                continue;
            }

            matched_rows += 1;
            collector.add_record(&record, &headers)?;
        }

        progress.finish();

        let stats = collector.finalize();

        // Render output
        let format = args.format.as_deref().unwrap_or("table");
        let renderer = Renderer::new(OutputFormat::from_str(format)?)
            .with_output(cli.output.clone())
            .with_color(ColorMode::from_str(&cli.color));
        renderer.render_summary(
            file_path,
            total_rows,
            matched_rows,
            args.where_clause.as_deref(),
            &stats,
        )?;

        Ok(())
    }

    pub fn run_schema(cli: &Cli, args: &cli::SchemaArgs) -> Result<()> {
        let file_path = cli
            .file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("FILE is required"))?;

        let options = build_csv_options(cli);
        let mut reader = CsvReader::from_path_with_options(file_path, options)?;
        let headers = reader.headers()?.clone();

        let mut inferrer = SchemaInferrer::new(&headers);
        let mut progress = ProgressTracker::new(file_path, cli.quiet);

        for result in reader.records() {
            let record = result?;
            progress.update(&record);
            inferrer.add_record(&record)?;
        }

        progress.finish();
        let schema = inferrer.finalize();

        let format = args.format.as_deref().unwrap_or("table");
        let renderer = Renderer::new(OutputFormat::from_str(format)?)
            .with_output(cli.output.clone())
            .with_color(ColorMode::from_str(&cli.color));
        renderer.render_schema(file_path, &schema)?;

        Ok(())
    }
}
