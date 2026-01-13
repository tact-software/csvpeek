use csv::StringRecord;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

pub struct ProgressTracker {
    bar: Option<ProgressBar>,
    update_interval: u64,
    count: u64,
    bytes_read: u64,
}

impl ProgressTracker {
    pub fn new(file_path: &str, quiet: bool) -> Self {
        if quiet {
            return Self {
                bar: None,
                update_interval: 0,
                count: 0,
                bytes_read: 0,
            };
        }

        // Get file size to estimate progress
        let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

        // Only show progress bar for files > 1MB
        if file_size < 1_000_000 {
            return Self {
                bar: None,
                update_interval: 0,
                count: 0,
                bytes_read: 0,
            };
        }

        let bar = ProgressBar::new(file_size);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        Self {
            bar: Some(bar),
            update_interval: 1000, // Update every 1000 rows for performance
            count: 0,
            bytes_read: 0,
        }
    }

    pub fn update(&mut self, record: &StringRecord) {
        self.count += 1;
        // Estimate bytes read from record size (approximate)
        let record_bytes: usize = record.iter().map(|f| f.len() + 1).sum();
        self.bytes_read += record_bytes as u64;

        if let Some(ref bar) = self.bar
            && self.count.is_multiple_of(self.update_interval)
        {
            bar.set_position(self.bytes_read);
        }
    }

    pub fn finish(&self) {
        if let Some(ref bar) = self.bar {
            bar.finish_and_clear();
        }
    }
}
