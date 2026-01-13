use anyhow::Result;
use csv::StringRecord;
use std::collections::HashSet;

use crate::types::{is_null, parse_value, ColumnStats, DataType};

pub struct StatsCollector {
    columns: Vec<ColumnAccumulator>,
    column_indices: Vec<usize>,
}

struct ColumnAccumulator {
    name: String,
    count: u64,
    null_count: u64,
    data_type: Option<DataType>,

    // Numeric stats
    sum: f64,
    sum_squares: f64, // For standard deviation
    numeric_count: u64,
    min_numeric: Option<f64>,
    max_numeric: Option<f64>,

    // String stats (for min/max)
    min_string: Option<String>,
    max_string: Option<String>,

    // v1.1 string stats
    min_len: Option<usize>,
    max_len: Option<usize>,
    unique_values: HashSet<String>,
}

impl ColumnAccumulator {
    fn new(name: String) -> Self {
        Self {
            name,
            count: 0,
            null_count: 0,
            data_type: None,
            sum: 0.0,
            sum_squares: 0.0,
            numeric_count: 0,
            min_numeric: None,
            max_numeric: None,
            min_string: None,
            max_string: None,
            min_len: None,
            max_len: None,
            unique_values: HashSet::new(),
        }
    }

    fn add_value(&mut self, value: &str) {
        self.count += 1;

        if is_null(value) {
            self.null_count += 1;
            return;
        }

        let trimmed = value.trim();
        let (dtype, _) = parse_value(trimmed);

        // Update data type (promote to more general type if needed)
        self.data_type = Some(match (self.data_type, dtype) {
            (None, t) => t,
            (Some(DataType::Integer), DataType::Float) => DataType::Float,
            (Some(DataType::Float), DataType::Integer) => DataType::Float,
            (Some(DataType::Integer), DataType::Integer) => DataType::Integer,
            (Some(DataType::Float), DataType::Float) => DataType::Float,
            (Some(DataType::Boolean), DataType::Boolean) => DataType::Boolean,
            (Some(_), DataType::String) => DataType::String,
            (Some(DataType::String), _) => DataType::String,
            (Some(t), _) => t,
        });

        // Update numeric stats if applicable
        if let Ok(num) = trimmed.parse::<f64>() {
            self.sum += num;
            self.sum_squares += num * num;
            self.numeric_count += 1;
            self.min_numeric = Some(
                self.min_numeric
                    .map_or(num, |m| if num < m { num } else { m }),
            );
            self.max_numeric = Some(
                self.max_numeric
                    .map_or(num, |m| if num > m { num } else { m }),
            );
        }

        // Update string stats
        let trimmed_str = trimmed.to_string();
        let len = trimmed.chars().count();

        self.min_len = Some(self.min_len.map_or(len, |m| m.min(len)));
        self.max_len = Some(self.max_len.map_or(len, |m| m.max(len)));
        self.unique_values.insert(trimmed_str.clone());

        self.min_string = Some(match &self.min_string {
            None => trimmed_str.clone(),
            Some(m) => {
                if trimmed_str < *m {
                    trimmed_str.clone()
                } else {
                    m.clone()
                }
            }
        });
        self.max_string = Some(match &self.max_string {
            None => trimmed_str,
            Some(m) => {
                if trimmed_str > *m {
                    trimmed_str
                } else {
                    m.clone()
                }
            }
        });
    }

    fn finalize(self) -> ColumnStats {
        let total = self.count;
        let null_rate = if total > 0 {
            (self.null_count as f64) / (total as f64) * 100.0
        } else {
            0.0
        };

        let data_type = self.data_type.unwrap_or(DataType::String);

        let (min, max, mean, sum, std) = match data_type {
            DataType::Integer | DataType::Float => {
                let mean = if self.numeric_count > 0 {
                    Some(self.sum / self.numeric_count as f64)
                } else {
                    None
                };

                // Calculate standard deviation
                let std = if self.numeric_count > 1 {
                    let n = self.numeric_count as f64;
                    let variance = (self.sum_squares - (self.sum * self.sum) / n) / (n - 1.0);
                    Some(variance.sqrt())
                } else {
                    None
                };

                let sum = if self.numeric_count > 0 {
                    Some(self.sum)
                } else {
                    None
                };

                (
                    self.min_numeric.map(|v| format_number(v, data_type)),
                    self.max_numeric.map(|v| format_number(v, data_type)),
                    mean,
                    sum,
                    std,
                )
            }
            _ => (self.min_string, self.max_string, None, None, None),
        };

        ColumnStats {
            name: self.name,
            data_type,
            count: total - self.null_count,
            null_count: self.null_count,
            null_rate,
            min,
            max,
            mean,
            sum,
            std,
            min_len: self.min_len,
            max_len: self.max_len,
            unique_count: Some(self.unique_values.len()),
        }
    }
}

fn format_number(v: f64, dtype: DataType) -> String {
    match dtype {
        DataType::Integer => format!("{}", v as i64),
        _ => format!("{:.6}", v).trim_end_matches('0').trim_end_matches('.').to_string(),
    }
}

impl StatsCollector {
    pub fn new(target_columns: &[String], headers: &StringRecord) -> Self {
        let header_vec: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

        let mut columns = Vec::new();
        let mut column_indices = Vec::new();

        for col in target_columns {
            if let Some(idx) = header_vec.iter().position(|h| h == col) {
                columns.push(ColumnAccumulator::new(col.clone()));
                column_indices.push(idx);
            }
        }

        Self {
            columns,
            column_indices,
        }
    }

    pub fn add_record(&mut self, record: &StringRecord, _headers: &StringRecord) -> Result<()> {
        for (acc, &idx) in self.columns.iter_mut().zip(self.column_indices.iter()) {
            let value = record.get(idx).unwrap_or("");
            acc.add_value(value);
        }
        Ok(())
    }

    pub fn finalize(self) -> Vec<ColumnStats> {
        self.columns.into_iter().map(|acc| acc.finalize()).collect()
    }
}
