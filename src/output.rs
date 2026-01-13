use anyhow::Result;
use colored::Colorize;
use comfy_table::{Cell, ContentArrangement, Table};
use std::fs::File;
use std::io::{self, BufWriter, IsTerminal, Write};

use crate::types::{ColumnSchema, ColumnStats, DataType};

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
    NdJson,
    Csv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "ndjson" => Ok(OutputFormat::NdJson),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(anyhow::anyhow!(
                "Unknown output format: {}. Supported: table, json, ndjson, csv",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl ColorMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "always" => ColorMode::Always,
            "never" => ColorMode::Never,
            _ => ColorMode::Auto,
        }
    }

    pub fn should_colorize(&self, is_tty: bool) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => is_tty,
        }
    }
}

pub struct Renderer {
    format: OutputFormat,
    output_path: Option<String>,
    color_mode: ColorMode,
}

impl Renderer {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            output_path: None,
            color_mode: ColorMode::Auto,
        }
    }

    pub fn with_output(mut self, path: Option<String>) -> Self {
        self.output_path = path;
        self
    }

    pub fn with_color(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    fn use_color(&self) -> bool {
        let is_tty = self.output_path.is_none() && io::stdout().is_terminal();
        self.color_mode.should_colorize(is_tty)
    }

    fn get_writer(&self) -> Result<Box<dyn Write>> {
        match &self.output_path {
            Some(path) => {
                let file = File::create(path)?;
                Ok(Box::new(BufWriter::new(file)))
            }
            None => Ok(Box::new(io::stdout())),
        }
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
            OutputFormat::NdJson => self.render_summary_ndjson(stats),
            OutputFormat::Csv => self.render_summary_csv(stats),
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
        let use_color = self.use_color();
        let mut w = self.get_writer()?;

        // Header info with optional color
        if use_color {
            writeln!(w, "{} {}", "file:".cyan(), file)?;
            writeln!(
                w,
                "{} {} ({} {})",
                "rows:".cyan(),
                total_rows,
                "matched:".cyan(),
                matched_rows
            )?;
            if let Some(f) = filter {
                writeln!(w, "{} {}", "filter:".cyan(), f)?;
            }
        } else {
            writeln!(w, "file: {}", file)?;
            writeln!(w, "rows: {} (matched: {})", total_rows, matched_rows)?;
            if let Some(f) = filter {
                writeln!(w, "filter: {}", f)?;
            }
        }
        writeln!(w)?;

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("column"),
            Cell::new("type"),
            Cell::new("count"),
            Cell::new("null%"),
            Cell::new("unique"),
            Cell::new("min"),
            Cell::new("max"),
            Cell::new("mean"),
            Cell::new("median"),
            Cell::new("std"),
        ]);

        for stat in stats {
            let type_str = if use_color {
                colorize_type(stat.data_type)
            } else {
                stat.data_type.to_string()
            };
            table.add_row(vec![
                Cell::new(&stat.name),
                Cell::new(type_str),
                Cell::new(stat.count.to_string()),
                Cell::new(format!("{:.1}%", stat.null_rate)),
                Cell::new(stat.unique_count.map_or("-".to_string(), |v| v.to_string())),
                Cell::new(stat.min.as_deref().unwrap_or("-")),
                Cell::new(stat.max.as_deref().unwrap_or("-")),
                Cell::new(
                    stat.mean
                        .map(|m| format!("{:.2}", m))
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::new(
                    stat.median
                        .map(|m| format!("{:.2}", m))
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::new(
                    stat.std
                        .map(|s| format!("{:.2}", s))
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ]);
        }

        writeln!(w, "{table}")?;

        // Show top values for string columns
        let has_top_values = stats.iter().any(|s| s.top_values.is_some());
        if has_top_values {
            writeln!(w)?;
            writeln!(w, "Top values:")?;
            for stat in stats {
                if let Some(ref top) = stat.top_values {
                    let top_str: Vec<String> =
                        top.iter().map(|(v, c)| format!("{}({})", v, c)).collect();
                    writeln!(w, "  {}: {}", stat.name, top_str.join(", "))?;
                }
            }
        }

        Ok(())
    }

    fn render_summary_json(&self, stats: &[ColumnStats]) -> Result<()> {
        let mut w = self.get_writer()?;
        let json = serde_json::to_string_pretty(stats)?;
        writeln!(w, "{}", json)?;
        Ok(())
    }

    fn render_summary_ndjson(&self, stats: &[ColumnStats]) -> Result<()> {
        let mut w = self.get_writer()?;
        for stat in stats {
            let json = serde_json::to_string(stat)?;
            writeln!(w, "{}", json)?;
        }
        Ok(())
    }

    fn render_summary_csv(&self, stats: &[ColumnStats]) -> Result<()> {
        let mut w = self.get_writer()?;
        writeln!(
            w,
            "column,type,count,null_count,null_rate,unique_count,min,max,mean,median,p25,p75,sum,std,min_len,max_len"
        )?;
        for stat in stats {
            writeln!(
                w,
                "{},{},{},{},{:.2},{},{},{},{},{},{},{},{},{},{},{}",
                escape_csv(&stat.name),
                stat.data_type,
                stat.count,
                stat.null_count,
                stat.null_rate,
                stat.unique_count.map_or(String::new(), |v| v.to_string()),
                stat.min.as_deref().map_or(String::new(), escape_csv),
                stat.max.as_deref().map_or(String::new(), escape_csv),
                stat.mean.map_or(String::new(), |v| format!("{:.6}", v)),
                stat.median.map_or(String::new(), |v| format!("{:.6}", v)),
                stat.p25.map_or(String::new(), |v| format!("{:.6}", v)),
                stat.p75.map_or(String::new(), |v| format!("{:.6}", v)),
                stat.sum.map_or(String::new(), |v| format!("{:.6}", v)),
                stat.std.map_or(String::new(), |v| format!("{:.6}", v)),
                stat.min_len.map_or(String::new(), |v| v.to_string()),
                stat.max_len.map_or(String::new(), |v| v.to_string()),
            )?;
        }
        Ok(())
    }

    pub fn render_schema(&self, file: &str, schema: &[ColumnSchema]) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.render_schema_table(file, schema),
            OutputFormat::Json => self.render_schema_json(schema),
            OutputFormat::NdJson => self.render_schema_ndjson(schema),
            OutputFormat::Csv => self.render_schema_csv(schema),
        }
    }

    fn render_schema_table(&self, file: &str, schema: &[ColumnSchema]) -> Result<()> {
        let use_color = self.use_color();
        let mut w = self.get_writer()?;

        if use_color {
            writeln!(w, "{} {}", "file:".cyan(), file)?;
            writeln!(w, "{} {}", "columns:".cyan(), schema.len())?;
        } else {
            writeln!(w, "file: {}", file)?;
            writeln!(w, "columns: {}", schema.len())?;
        }
        writeln!(w)?;

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("column"),
            Cell::new("type"),
            Cell::new("null%"),
            Cell::new("samples"),
        ]);

        for col in schema {
            let samples = if col.sample_values.is_empty() {
                "-".to_string()
            } else {
                col.sample_values.join(", ")
            };
            let type_str = if use_color {
                colorize_type(col.inferred_type)
            } else {
                col.inferred_type.to_string()
            };
            table.add_row(vec![
                Cell::new(&col.name),
                Cell::new(type_str),
                Cell::new(format!("{:.1}%", col.null_rate)),
                Cell::new(samples),
            ]);
        }

        writeln!(w, "{table}")?;
        Ok(())
    }

    fn render_schema_json(&self, schema: &[ColumnSchema]) -> Result<()> {
        let mut w = self.get_writer()?;
        let json = serde_json::to_string_pretty(schema)?;
        writeln!(w, "{}", json)?;
        Ok(())
    }

    fn render_schema_ndjson(&self, schema: &[ColumnSchema]) -> Result<()> {
        let mut w = self.get_writer()?;
        for col in schema {
            let json = serde_json::to_string(col)?;
            writeln!(w, "{}", json)?;
        }
        Ok(())
    }

    fn render_schema_csv(&self, schema: &[ColumnSchema]) -> Result<()> {
        let mut w = self.get_writer()?;
        writeln!(
            w,
            "column,type,null_count,total_count,null_rate,sample_values"
        )?;
        for col in schema {
            let samples = col.sample_values.join("; ");
            writeln!(
                w,
                "{},{},{},{},{:.2},{}",
                escape_csv(&col.name),
                col.inferred_type,
                col.null_count,
                col.total_count,
                col.null_rate,
                escape_csv(&samples),
            )?;
        }
        Ok(())
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn colorize_type(dtype: DataType) -> String {
    match dtype {
        DataType::Integer => "integer".blue().to_string(),
        DataType::Float => "float".green().to_string(),
        DataType::Boolean => "boolean".yellow().to_string(),
        DataType::String => "string".magenta().to_string(),
    }
}
