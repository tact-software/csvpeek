use anyhow::Result;
use csv::{Reader, StringRecord};
use std::fs::File;
use std::path::Path;

use crate::error::CsvpeekError;

#[derive(Debug, Clone, Default)]
pub struct CsvOptions {
    pub delimiter: u8,
    pub no_header: bool,
}

impl CsvOptions {
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            no_header: false,
        }
    }

    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_no_header(mut self, no_header: bool) -> Self {
        self.no_header = no_header;
        self
    }
}

pub struct CsvReader {
    reader: Reader<File>,
    headers: Option<StringRecord>,
    generated_headers: bool,
}

impl CsvReader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_path_with_options(path, CsvOptions::new())
    }

    pub fn from_path_with_options<P: AsRef<Path>>(path: P, options: CsvOptions) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(CsvpeekError::FileNotFound(path.display().to_string()).into());
        }

        let reader = csv::ReaderBuilder::new()
            .has_headers(!options.no_header)
            .delimiter(options.delimiter)
            .flexible(true)
            .from_path(path)?;

        Ok(Self {
            reader,
            headers: None,
            generated_headers: options.no_header,
        })
    }

    pub fn headers(&mut self) -> Result<&StringRecord> {
        if self.headers.is_none() {
            if self.generated_headers {
                // Generate headers like col0, col1, col2...
                // We need to peek at first record to know column count
                let first_record = self.reader.records().next();
                if let Some(Ok(record)) = first_record {
                    let count = record.len();
                    let mut headers = StringRecord::new();
                    for i in 0..count {
                        headers.push_field(&format!("col{}", i));
                    }
                    self.headers = Some(headers);
                } else {
                    self.headers = Some(StringRecord::new());
                }
            } else {
                self.headers = Some(self.reader.headers()?.clone());
            }
        }
        Ok(self.headers.as_ref().unwrap())
    }

    pub fn records(&mut self) -> impl Iterator<Item = Result<StringRecord, csv::Error>> + '_ {
        self.reader.records()
    }
}
