use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Integer,
    Float,
    Boolean,
    String,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Integer => write!(f, "integer"),
            DataType::Float => write!(f, "float"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::String => write!(f, "string"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnStats {
    pub name: String,
    pub data_type: DataType,
    pub count: u64,
    pub null_count: u64,
    pub null_rate: f64,
    pub min: Option<String>,
    pub max: Option<String>,
    pub mean: Option<f64>,
    // v1.1 numeric statistics
    pub sum: Option<f64>,
    pub std: Option<f64>,
    // v1.1 string statistics
    pub min_len: Option<usize>,
    pub max_len: Option<usize>,
    pub unique_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnSchema {
    pub name: String,
    pub inferred_type: DataType,
    pub null_count: u64,
    pub total_count: u64,
    pub null_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Null,
    Integer(i64),
    Float(i64, u64), // mantissa representation for ordering
    Boolean(bool),
    String,
}

pub fn parse_value(s: &str) -> (DataType, Value) {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        return (DataType::String, Value::Null);
    }

    // Try boolean
    match trimmed.to_lowercase().as_str() {
        "true" | "false" => {
            return (
                DataType::Boolean,
                Value::Boolean(trimmed.to_lowercase() == "true"),
            );
        }
        _ => {}
    }

    // Try integer
    if let Ok(i) = trimmed.parse::<i64>() {
        return (DataType::Integer, Value::Integer(i));
    }

    // Try float
    if let Ok(f) = trimmed.parse::<f64>() {
        if f.is_finite() {
            return (DataType::Float, Value::Float(f.to_bits() as i64, f.to_bits()));
        }
    }

    (DataType::String, Value::String)
}

pub fn is_null(s: &str) -> bool {
    let trimmed = s.trim().to_lowercase();
    trimmed.is_empty() || trimmed == "null" || trimmed == "na" || trimmed == "n/a"
}
