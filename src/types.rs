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
    // v1.2 statistics
    pub median: Option<f64>,
    pub p25: Option<f64>,
    pub p75: Option<f64>,
    pub top_values: Option<Vec<(String, usize)>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnSchema {
    pub name: String,
    pub inferred_type: DataType,
    pub null_count: u64,
    pub total_count: u64,
    pub null_rate: f64,
    pub sample_values: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_value_integer() {
        let (dtype, val) = parse_value("42");
        assert_eq!(dtype, DataType::Integer);
        assert_eq!(val, Value::Integer(42));

        let (dtype, val) = parse_value("-123");
        assert_eq!(dtype, DataType::Integer);
        assert_eq!(val, Value::Integer(-123));

        let (dtype, val) = parse_value("  100  ");
        assert_eq!(dtype, DataType::Integer);
        assert_eq!(val, Value::Integer(100));
    }

    #[test]
    fn test_parse_value_float() {
        let (dtype, _) = parse_value("3.14");
        assert_eq!(dtype, DataType::Float);

        let (dtype, _) = parse_value("-2.5");
        assert_eq!(dtype, DataType::Float);

        let (dtype, _) = parse_value("1.0");
        assert_eq!(dtype, DataType::Float);
    }

    #[test]
    fn test_parse_value_boolean() {
        let (dtype, val) = parse_value("true");
        assert_eq!(dtype, DataType::Boolean);
        assert_eq!(val, Value::Boolean(true));

        let (dtype, val) = parse_value("false");
        assert_eq!(dtype, DataType::Boolean);
        assert_eq!(val, Value::Boolean(false));

        let (dtype, val) = parse_value("TRUE");
        assert_eq!(dtype, DataType::Boolean);
        assert_eq!(val, Value::Boolean(true));

        let (dtype, val) = parse_value("False");
        assert_eq!(dtype, DataType::Boolean);
        assert_eq!(val, Value::Boolean(false));
    }

    #[test]
    fn test_parse_value_string() {
        let (dtype, val) = parse_value("hello");
        assert_eq!(dtype, DataType::String);
        assert_eq!(val, Value::String);

        let (dtype, val) = parse_value("hello world");
        assert_eq!(dtype, DataType::String);
        assert_eq!(val, Value::String);
    }

    #[test]
    fn test_parse_value_empty() {
        let (dtype, val) = parse_value("");
        assert_eq!(dtype, DataType::String);
        assert_eq!(val, Value::Null);

        let (dtype, val) = parse_value("   ");
        assert_eq!(dtype, DataType::String);
        assert_eq!(val, Value::Null);
    }

    #[test]
    fn test_is_null() {
        assert!(is_null(""));
        assert!(is_null("   "));
        assert!(is_null("null"));
        assert!(is_null("NULL"));
        assert!(is_null("Null"));
        assert!(is_null("na"));
        assert!(is_null("NA"));
        assert!(is_null("n/a"));
        assert!(is_null("N/A"));

        assert!(!is_null("0"));
        assert!(!is_null("false"));
        assert!(!is_null("none"));
        assert!(!is_null("hello"));
    }

    #[test]
    fn test_datatype_display() {
        assert_eq!(format!("{}", DataType::Integer), "integer");
        assert_eq!(format!("{}", DataType::Float), "float");
        assert_eq!(format!("{}", DataType::Boolean), "boolean");
        assert_eq!(format!("{}", DataType::String), "string");
    }
}
