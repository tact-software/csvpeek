use anyhow::Result;
use csv::StringRecord;

use crate::types::{is_null, parse_value, ColumnSchema, DataType};

pub struct SchemaInferrer {
    columns: Vec<ColumnTypeAccumulator>,
}

struct ColumnTypeAccumulator {
    name: String,
    total_count: u64,
    null_count: u64,
    integer_count: u64,
    float_count: u64,
    boolean_count: u64,
    string_count: u64,
}

impl ColumnTypeAccumulator {
    fn new(name: String) -> Self {
        Self {
            name,
            total_count: 0,
            null_count: 0,
            integer_count: 0,
            float_count: 0,
            boolean_count: 0,
            string_count: 0,
        }
    }

    fn add_value(&mut self, value: &str) {
        self.total_count += 1;

        if is_null(value) {
            self.null_count += 1;
            return;
        }

        let (dtype, _) = parse_value(value);
        match dtype {
            DataType::Integer => self.integer_count += 1,
            DataType::Float => self.float_count += 1,
            DataType::Boolean => self.boolean_count += 1,
            DataType::String => self.string_count += 1,
        }
    }

    fn infer_type(&self) -> DataType {
        let non_null = self.total_count - self.null_count;

        if non_null == 0 {
            return DataType::String;
        }

        // If all non-null values are integers, it's an integer column
        if self.integer_count == non_null {
            return DataType::Integer;
        }

        // If all non-null values are floats or integers, it's a float column
        if self.integer_count + self.float_count == non_null {
            return DataType::Float;
        }

        // If all non-null values are booleans, it's a boolean column
        if self.boolean_count == non_null {
            return DataType::Boolean;
        }

        DataType::String
    }

    fn finalize(self) -> ColumnSchema {
        let null_rate = if self.total_count > 0 {
            (self.null_count as f64) / (self.total_count as f64) * 100.0
        } else {
            0.0
        };

        let inferred_type = self.infer_type();

        ColumnSchema {
            name: self.name,
            inferred_type,
            null_count: self.null_count,
            total_count: self.total_count,
            null_rate,
        }
    }
}

impl SchemaInferrer {
    pub fn new(headers: &StringRecord) -> Self {
        let columns = headers
            .iter()
            .map(|h| ColumnTypeAccumulator::new(h.to_string()))
            .collect();

        Self { columns }
    }

    pub fn add_record(&mut self, record: &StringRecord) -> Result<()> {
        for (i, acc) in self.columns.iter_mut().enumerate() {
            let value = record.get(i).unwrap_or("");
            acc.add_value(value);
        }
        Ok(())
    }

    pub fn finalize(self) -> Vec<ColumnSchema> {
        self.columns.into_iter().map(|acc| acc.finalize()).collect()
    }
}
