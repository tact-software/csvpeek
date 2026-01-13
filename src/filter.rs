use anyhow::Result;
use csv::StringRecord;
use regex::Regex;

use crate::error::CsvpeekError;
use crate::types::is_null;

#[derive(Debug, Clone)]
pub struct Filter {
    expr: Expr,
    column_indices: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone)]
enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Compare(String, CompareOp, Value),
    Contains(String, String),
    Matches(String, Regex),
    In(String, Vec<String>),
    IsNull(String),
    IsNotNull(String),
}

#[derive(Debug, Clone, Copy)]
enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Number(f64),
}

impl Filter {
    pub fn parse(expr_str: &str, headers: &StringRecord) -> Result<Self> {
        let column_indices: std::collections::HashMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.to_string(), i))
            .collect();

        let expr = parse_expr(expr_str, &column_indices)?;

        Ok(Self {
            expr,
            column_indices,
        })
    }

    pub fn matches(&self, record: &StringRecord, _headers: &StringRecord) -> Result<bool> {
        eval_expr(&self.expr, record, &self.column_indices)
    }
}

fn parse_expr(s: &str, columns: &std::collections::HashMap<String, usize>) -> Result<Expr> {
    let s = s.trim();

    // Handle OR (lowest precedence)
    if let Some(pos) = find_operator(s, "||") {
        let left = parse_expr(&s[..pos], columns)?;
        let right = parse_expr(&s[pos + 2..], columns)?;
        return Ok(Expr::Or(Box::new(left), Box::new(right)));
    }

    // Handle AND
    if let Some(pos) = find_operator(s, "&&") {
        let left = parse_expr(&s[..pos], columns)?;
        let right = parse_expr(&s[pos + 2..], columns)?;
        return Ok(Expr::And(Box::new(left), Box::new(right)));
    }

    // Handle NOT
    if s.starts_with('!') && !s.starts_with("!=") {
        let inner = parse_expr(&s[1..], columns)?;
        return Ok(Expr::Not(Box::new(inner)));
    }

    // Handle parentheses
    if s.starts_with('(') && s.ends_with(')') {
        return parse_expr(&s[1..s.len() - 1], columns);
    }

    // Handle function calls
    if let Some(func_expr) = parse_function(s, columns)? {
        return Ok(func_expr);
    }

    // Handle comparisons
    parse_comparison(s, columns)
}

fn find_operator(s: &str, op: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        if escape {
            escape = false;
            continue;
        }

        match chars[i] {
            '\\' => escape = true,
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => depth -= 1,
            _ if !in_string && depth == 0 => {
                if s[i..].starts_with(op) {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_function(
    s: &str,
    columns: &std::collections::HashMap<String, usize>,
) -> Result<Option<Expr>> {
    let s = s.trim();

    // contains(col, "value")
    if s.starts_with("contains(") && s.ends_with(')') {
        let inner = &s[9..s.len() - 1];
        let (col, val) = parse_func_args(inner)?;
        validate_column(&col, columns)?;
        return Ok(Some(Expr::Contains(col, val)));
    }

    // in(col, ["a", "b", "c"])
    if s.starts_with("in(") && s.ends_with(')') {
        let inner = &s[3..s.len() - 1];
        let (col, vals_str) = parse_func_args(inner)?;
        validate_column(&col, columns)?;
        let vals = parse_array(&vals_str)?;
        return Ok(Some(Expr::In(col, vals)));
    }

    // is_null(col)
    if s.starts_with("is_null(") && s.ends_with(')') {
        let col = s[8..s.len() - 1].trim().to_string();
        validate_column(&col, columns)?;
        return Ok(Some(Expr::IsNull(col)));
    }

    // is_not_null(col)
    if s.starts_with("is_not_null(") && s.ends_with(')') {
        let col = s[12..s.len() - 1].trim().to_string();
        validate_column(&col, columns)?;
        return Ok(Some(Expr::IsNotNull(col)));
    }

    // matches(col, "regex_pattern")
    if s.starts_with("matches(") && s.ends_with(')') {
        let inner = &s[8..s.len() - 1];
        let (col, pattern) = parse_func_args(inner)?;
        validate_column(&col, columns)?;
        let regex = Regex::new(&pattern).map_err(|e| {
            CsvpeekError::InvalidFilter(format!("Invalid regex pattern '{}': {}", pattern, e))
        })?;
        return Ok(Some(Expr::Matches(col, regex)));
    }

    Ok(None)
}

fn parse_func_args(s: &str) -> Result<(String, String)> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in s.chars().enumerate() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' => escape = true,
            '"' => in_string = !in_string,
            '[' | '(' if !in_string => depth += 1,
            ']' | ')' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                let col = s[..i].trim().to_string();
                let val = s[i + 1..].trim();
                let val = if val.starts_with('"') && val.ends_with('"') {
                    val[1..val.len() - 1].to_string()
                } else {
                    val.to_string()
                };
                return Ok((col, val));
            }
            _ => {}
        }
    }

    Err(CsvpeekError::InvalidFilter("Invalid function arguments".to_string()).into())
}

fn parse_array(s: &str) -> Result<Vec<String>> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(CsvpeekError::InvalidFilter("Expected array".to_string()).into());
    }

    let inner = &s[1..s.len() - 1];
    let mut result = Vec::new();

    for item in inner.split(',') {
        let item = item.trim();
        let item = if item.starts_with('"') && item.ends_with('"') {
            item[1..item.len() - 1].to_string()
        } else {
            item.to_string()
        };
        if !item.is_empty() {
            result.push(item);
        }
    }

    Ok(result)
}

fn parse_comparison(s: &str, columns: &std::collections::HashMap<String, usize>) -> Result<Expr> {
    let ops = [("==", CompareOp::Eq), ("!=", CompareOp::Ne),
               ("<=", CompareOp::Le), (">=", CompareOp::Ge),
               ("<", CompareOp::Lt), (">", CompareOp::Gt)];

    for (op_str, op) in ops.iter() {
        if let Some(pos) = s.find(op_str) {
            let col = s[..pos].trim().to_string();
            let val_str = s[pos + op_str.len()..].trim();

            validate_column(&col, columns)?;

            let value = if val_str.starts_with('"') && val_str.ends_with('"') {
                Value::String(val_str[1..val_str.len() - 1].to_string())
            } else if let Ok(n) = val_str.parse::<f64>() {
                Value::Number(n)
            } else {
                Value::String(val_str.to_string())
            };

            return Ok(Expr::Compare(col, *op, value));
        }
    }

    Err(CsvpeekError::InvalidFilter(format!("Cannot parse expression: {}", s)).into())
}

fn validate_column(col: &str, columns: &std::collections::HashMap<String, usize>) -> Result<()> {
    if !columns.contains_key(col) {
        return Err(CsvpeekError::ColumnNotFound {
            name: col.to_string(),
            suggestion: None,
        }
        .into());
    }
    Ok(())
}

fn eval_expr(
    expr: &Expr,
    record: &StringRecord,
    columns: &std::collections::HashMap<String, usize>,
) -> Result<bool> {
    match expr {
        Expr::And(left, right) => {
            Ok(eval_expr(left, record, columns)? && eval_expr(right, record, columns)?)
        }
        Expr::Or(left, right) => {
            Ok(eval_expr(left, record, columns)? || eval_expr(right, record, columns)?)
        }
        Expr::Not(inner) => Ok(!eval_expr(inner, record, columns)?),
        Expr::Compare(col, op, val) => {
            let idx = columns.get(col).copied().unwrap_or(0);
            let cell = record.get(idx).unwrap_or("");
            eval_compare(cell, op, val)
        }
        Expr::Contains(col, substr) => {
            let idx = columns.get(col).copied().unwrap_or(0);
            let cell = record.get(idx).unwrap_or("");
            Ok(cell.contains(substr.as_str()))
        }
        Expr::Matches(col, regex) => {
            let idx = columns.get(col).copied().unwrap_or(0);
            let cell = record.get(idx).unwrap_or("");
            Ok(regex.is_match(cell))
        }
        Expr::In(col, vals) => {
            let idx = columns.get(col).copied().unwrap_or(0);
            let cell = record.get(idx).unwrap_or("");
            Ok(vals.iter().any(|v| v == cell))
        }
        Expr::IsNull(col) => {
            let idx = columns.get(col).copied().unwrap_or(0);
            let cell = record.get(idx).unwrap_or("");
            Ok(is_null(cell))
        }
        Expr::IsNotNull(col) => {
            let idx = columns.get(col).copied().unwrap_or(0);
            let cell = record.get(idx).unwrap_or("");
            Ok(!is_null(cell))
        }
    }
}

fn eval_compare(cell: &str, op: &CompareOp, val: &Value) -> Result<bool> {
    match val {
        Value::Number(n) => {
            if let Ok(cell_num) = cell.trim().parse::<f64>() {
                Ok(match op {
                    CompareOp::Eq => (cell_num - n).abs() < f64::EPSILON,
                    CompareOp::Ne => (cell_num - n).abs() >= f64::EPSILON,
                    CompareOp::Lt => cell_num < *n,
                    CompareOp::Le => cell_num <= *n,
                    CompareOp::Gt => cell_num > *n,
                    CompareOp::Ge => cell_num >= *n,
                })
            } else {
                // Type mismatch - row is filtered out
                Ok(false)
            }
        }
        Value::String(s) => Ok(match op {
            CompareOp::Eq => cell == s,
            CompareOp::Ne => cell != s,
            CompareOp::Lt => cell < s.as_str(),
            CompareOp::Le => cell <= s.as_str(),
            CompareOp::Gt => cell > s.as_str(),
            CompareOp::Ge => cell >= s.as_str(),
        }),
    }
}
