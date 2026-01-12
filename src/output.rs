use anyhow::Result;
use comfy_table::{Cell, ContentArrangement, Table};

use crate::types::{ColumnSchema, ColumnStats};

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            _ => Err(anyhow::anyhow!("Unknown output format: {}", s)),
        }
    }
}

pub struct Renderer {
    format: OutputFormat,
}

impl Renderer {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn render_summary(
        &self,
        file: &str,
        total_rows: u64,
        matched_rows: u64,
        filter: Option<&str>,
        stats: &[ColumnStats],
    ) -> Result<()> {
        match self.format {
            OutputFormat::Table => {
                self.render_summary_table(file, total_rows, matched_rows, filter, stats)
            }
            OutputFormat::Json => self.render_summary_json(stats),
        }
    }

    fn render_summary_table(
        &self,
        file: &str,
        total_rows: u64,
        matched_rows: u64,
        filter: Option<&str>,
        stats: &[ColumnStats],
    ) -> Result<()> {
        println!("file: {}", file);
        println!(
            "rows: {} (matched: {})",
            total_rows, matched_rows
        );
        if let Some(f) = filter {
            println!("filter: {}", f);
        }
        println!();

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("column"),
            Cell::new("type"),
            Cell::new("count"),
            Cell::new("null%"),
            Cell::new("min"),
            Cell::new("max"),
            Cell::new("mean"),
        ]);

        for stat in stats {
            table.add_row(vec![
                Cell::new(&stat.name),
                Cell::new(stat.data_type.to_string()),
                Cell::new(stat.count.to_string()),
                Cell::new(format!("{:.1}%", stat.null_rate)),
                Cell::new(stat.min.as_deref().unwrap_or("-")),
                Cell::new(stat.max.as_deref().unwrap_or("-")),
                Cell::new(
                    stat.mean
                        .map(|m| format!("{:.2}", m))
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ]);
        }

        println!("{table}");
        Ok(())
    }

    fn render_summary_json(&self, stats: &[ColumnStats]) -> Result<()> {
        let json = serde_json::to_string_pretty(stats)?;
        println!("{}", json);
        Ok(())
    }

    pub fn render_schema(&self, file: &str, schema: &[ColumnSchema]) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.render_schema_table(file, schema),
            OutputFormat::Json => self.render_schema_json(schema),
        }
    }

    fn render_schema_table(&self, file: &str, schema: &[ColumnSchema]) -> Result<()> {
        println!("file: {}", file);
        println!("columns: {}", schema.len());
        println!();

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("column"),
            Cell::new("type"),
            Cell::new("null%"),
        ]);

        for col in schema {
            table.add_row(vec![
                Cell::new(&col.name),
                Cell::new(col.inferred_type.to_string()),
                Cell::new(format!("{:.1}%", col.null_rate)),
            ]);
        }

        println!("{table}");
        Ok(())
    }

    fn render_schema_json(&self, schema: &[ColumnSchema]) -> Result<()> {
        let json = serde_json::to_string_pretty(schema)?;
        println!("{}", json);
        Ok(())
    }
}
