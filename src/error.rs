use thiserror::Error;

#[derive(Error, Debug)]
pub enum CsvpeekError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Column not found: {name}")]
    ColumnNotFound {
        name: String,
        #[source]
        suggestion: Option<ColumnSuggestion>,
    },

    #[error("Invalid filter expression: {0}")]
    InvalidFilter(String),

    #[error("Parse error at position {position}: {message}")]
    ParseError { position: usize, message: String },

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ColumnSuggestion {
    pub suggested: String,
}

impl std::fmt::Display for ColumnSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "did you mean \"{}\"?", self.suggested)
    }
}

impl std::error::Error for ColumnSuggestion {}

pub fn find_similar_column(name: &str, headers: &[String]) -> Option<String> {
    let name_lower = name.to_lowercase();

    // Exact case-insensitive match
    for h in headers {
        if h.to_lowercase() == name_lower {
            return Some(h.clone());
        }
    }

    // Simple edit distance check
    headers
        .iter()
        .filter(|h| {
            let h_lower = h.to_lowercase();
            levenshtein_distance(&name_lower, &h_lower) <= 2
        })
        .min_by_key(|h| levenshtein_distance(&name_lower, &h.to_lowercase()))
        .cloned()
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}
