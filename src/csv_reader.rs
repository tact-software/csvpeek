use anyhow::Result;
use chardetng::EncodingDetector;
use csv::{Reader, ReaderBuilder, StringRecord};
use encoding_rs::Encoding;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::CsvpeekError;

#[derive(Debug, Clone, Default)]
pub struct CsvOptions {
    pub delimiter: u8,
    pub no_header: bool,
    pub encoding: Option<String>, // None = auto-detect
}

impl CsvOptions {
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            no_header: false,
            encoding: None,
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

    pub fn with_encoding(mut self, encoding: Option<String>) -> Self {
        self.encoding = encoding;
        self
    }
}

pub struct CsvReader {
    reader: Reader<std::io::Cursor<String>>,
    headers: Option<StringRecord>,
    generated_headers: bool,
    detected_encoding: String,
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

        // Read file content
        let mut file = BufReader::new(File::open(path)?);
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Determine encoding
        let (content, detected_encoding) = if let Some(ref enc_name) = options.encoding {
            // Explicit encoding specified
            let encoding = lookup_encoding(enc_name)?;
            let (decoded, _, had_errors) = encoding.decode(&bytes);
            if had_errors {
                eprintln!(
                    "Warning: Some characters could not be decoded with encoding '{}'",
                    enc_name
                );
            }
            (decoded.into_owned(), encoding.name().to_string())
        } else {
            // Auto-detect encoding
            let (content, encoding_name) = detect_and_decode(&bytes);
            (content, encoding_name)
        };

        let cursor = std::io::Cursor::new(content);
        let reader = ReaderBuilder::new()
            .has_headers(!options.no_header)
            .delimiter(options.delimiter)
            .flexible(true)
            .from_reader(cursor);

        Ok(Self {
            reader,
            headers: None,
            generated_headers: options.no_header,
            detected_encoding,
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

    pub fn detected_encoding(&self) -> &str {
        &self.detected_encoding
    }
}

/// Detect encoding and decode bytes to UTF-8 string
fn detect_and_decode(bytes: &[u8]) -> (String, String) {
    // Check for BOM first
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 BOM
        let content = String::from_utf8_lossy(&bytes[3..]).into_owned();
        return (content, "UTF-8".to_string());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16 LE BOM
        let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return (decoded.into_owned(), "UTF-16LE".to_string());
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 BE BOM
        let (decoded, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return (decoded.into_owned(), "UTF-16BE".to_string());
    }

    // Try UTF-8 first (most common)
    if let Ok(s) = std::str::from_utf8(bytes) {
        return (s.to_string(), "UTF-8".to_string());
    }

    // Use chardetng for detection
    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);

    let (decoded, _, _) = encoding.decode(bytes);
    (decoded.into_owned(), encoding.name().to_string())
}

/// Look up encoding by name
fn lookup_encoding(name: &str) -> Result<&'static Encoding> {
    let normalized = name.to_lowercase().replace(['-', '_'], "");

    let encoding = match normalized.as_str() {
        "utf8" => encoding_rs::UTF_8,
        "utf16le" | "utf16" => encoding_rs::UTF_16LE,
        "utf16be" => encoding_rs::UTF_16BE,
        "shiftjis" | "sjis" | "cp932" | "windows31j" => encoding_rs::SHIFT_JIS,
        "eucjp" => encoding_rs::EUC_JP,
        "iso2022jp" => encoding_rs::ISO_2022_JP,
        "gbk" | "gb2312" | "cp936" => encoding_rs::GBK,
        "gb18030" => encoding_rs::GB18030,
        "big5" | "cp950" => encoding_rs::BIG5,
        "euckr" | "cp949" => encoding_rs::EUC_KR,
        "latin1" | "iso88591" | "cp1252" | "windows1252" => encoding_rs::WINDOWS_1252,
        "iso88592" => encoding_rs::ISO_8859_2,
        "iso885915" => encoding_rs::ISO_8859_15,
        "koi8r" => encoding_rs::KOI8_R,
        "koi8u" => encoding_rs::KOI8_U,
        _ => {
            // Try encoding_rs lookup
            Encoding::for_label(name.as_bytes()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown encoding: '{}'. Supported: utf-8, shift_jis, euc-jp, gbk, big5, latin1, etc.",
                    name
                )
            })?
        }
    };

    Ok(encoding)
}

/// List of supported encodings for help text
pub fn supported_encodings() -> &'static str {
    "utf-8, shift_jis (cp932), euc-jp, iso-2022-jp, gbk (gb2312), gb18030, big5, euc-kr, latin1 (windows-1252), etc."
}
