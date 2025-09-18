use chrono::Local;
use colored::Colorize;
use regex::Regex;
use std::fs::copy;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs::File, io::BufWriter};

use crate::utils::{ansi_regex, print_message};
use crate::validation;
use crate::error::Result;

const FLUSH_INTERVAL: usize = 10;

pub struct Log {
    // pub file: File,
    writer: BufWriter<File>,
    filename: String,
    file_path: PathBuf,
    enabled: bool,
    unsaved_changes: bool,
    prepend_timestamps: bool,
    ansi_regex: Regex,
    start_time: Instant,
    // Performance optimizations
    timestamp_buffer: String, // Reuse buffer for timestamps
    line_buffer: String,      // Reuse buffer for line processing
    flush_counter: usize,     // Batch flushes
}
impl Log {
    pub fn new(prepend_timestamps: bool) -> std::result::Result<Self, std::io::Error> {
        let filename = format!("log_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
        let file_path = PathBuf::from(&filename);
        let file = File::create(&file_path)?;
        let writer = BufWriter::with_capacity(8192, file); // Larger buffer
        let ansi_regex = ansi_regex();
        let start_time = Instant::now();
        Ok(Log {
            // file,
            writer,
            filename,
            file_path,
            enabled: true,
            unsaved_changes: false,
            prepend_timestamps,
            ansi_regex,
            start_time,
            timestamp_buffer: String::with_capacity(32),
            line_buffer: String::with_capacity(512),
            flush_counter: 0,
        })
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if self.enabled {
            // let _ = self.write_line("=== Resumed logging ==\n");
            print_message(format!("Logging {}", "resumed".green()));
        } else {
            // let _ = self.write_line("=== Paused  logging ==\n");
            print_message(format!("Logging {}", "paused".yellow()));
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.unsaved_changes
    }

    pub fn get_filename(&self) -> &str {
        &self.filename
    }

    pub fn write_line(&mut self, raw_line: &str) {
        self.line_buffer.clear();

        // Avoid regex if no ANSI codes detected (performance optimization)
        if raw_line.contains('\x1b') {
            self.line_buffer
                .push_str(&self.ansi_regex.replace_all(raw_line, ""));
        } else {
            self.line_buffer.push_str(raw_line);
        }

        if self.prepend_timestamps {
            self.create_timestamp_in_buffer(self.start_time.elapsed());
            self.line_buffer
                .insert_str(0, &format!("[{}] ", self.timestamp_buffer));
        }

        self.writer
            .write_all(self.line_buffer.as_bytes())
            .expect("Failed to write to log");

        self.unsaved_changes = true;
        self.flush_counter += 1;

        // Batch flush for better performance
        if self.flush_counter >= FLUSH_INTERVAL {
            let _ = self.writer.flush();
            self.flush_counter = 0;
        }
    }

    pub fn force_flush(&mut self) -> std::io::Result<()> {
        self.flush_counter = 0;
        self.writer.flush()
    }

    pub fn save_as(&mut self, new_file_path: &PathBuf) -> Result<()> {
        // Validate the file path before attempting to save
        let path_str = new_file_path.to_string_lossy();
        validation::validate_file_path(&path_str)?;
        
        match copy(&self.file_path, new_file_path) {
            Ok(_) => {
                self.unsaved_changes = false;
                println!("Saved to {}", new_file_path.display());
                Ok(())
            }
            Err(e) => {
                eprintln!("Error saving file: {}", e);
                Err(crate::error::SpewcapError::Io(e))
            }
        }
    }

    fn create_timestamp_in_buffer(&mut self, duration: Duration) {
        self.timestamp_buffer.clear();
        let total_millis = duration.as_millis();
        let hours = total_millis / 3_600_000;
        let minutes = (total_millis % 3_600_000) / 60_000;
        let seconds = (total_millis % 60_000) / 1_000;
        let millis = total_millis % 1_000;

        use std::fmt::Write;
        write!(
            self.timestamp_buffer,
            "{:02}:{:02}:{:02}:{:03}ms",
            hours, minutes, seconds, millis
        )
        .expect("Failed to write timestamp");
    }
}
