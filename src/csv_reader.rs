use anyhow::Result;
use csv::{Reader, StringRecord};
use std::fs::File;
use std::path::Path;

use crate::error::CsvpeekError;

pub struct CsvReader {
    reader: Reader<File>,
    headers: Option<StringRecord>,
}

impl CsvReader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(CsvpeekError::FileNotFound(path.display().to_string()).into());
        }

        let reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(path)?;

        Ok(Self {
            reader,
            headers: None,
        })
    }

    pub fn headers(&mut self) -> Result<&StringRecord> {
        if self.headers.is_none() {
            self.headers = Some(self.reader.headers()?.clone());
        }
        Ok(self.headers.as_ref().unwrap())
    }

    pub fn records(&mut self) -> impl Iterator<Item = Result<StringRecord, csv::Error>> + '_ {
        self.reader.records()
    }
}
