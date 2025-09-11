use chrono::Local;
use colored::Colorize;
use regex::Regex;
use std::fs::copy;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs::File, io::BufWriter};

use crate::utils::{ansi_regex, print_message};

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
}
impl Log {
    pub fn new(prepend_timestamps: bool) -> Result<Self, std::io::Error> {
        let filename = format!("log_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
        let file_path = PathBuf::from(&filename);
        let file = File::create(&file_path)?;
        let writer = BufWriter::new(file);
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
        let mut line = self.ansi_regex.replace_all(raw_line, "").to_string();
        if self.prepend_timestamps {
            let timestamp = create_timestamp(self.start_time.elapsed());
            line = format!("[{}] {}", timestamp, line);
        }
        self.writer
            .write_all(line.as_bytes())
            .expect("Failed to write to log");
        self.unsaved_changes = true;
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    pub fn save_as(&mut self, new_file_path: &PathBuf) {
        match copy(&self.file_path, new_file_path) {
            Ok(_) => {
                self.unsaved_changes = false;
                println!("Saved to {}", new_file_path.display())
            }
            Err(e) => eprintln!("Error saving file: {}", e),
        }
    }
}

fn create_timestamp(duration: Duration) -> String {
    let total_millis = duration.as_millis();
    let hours = total_millis / 3_600_000;
    let minutes = (total_millis % 3_600_000) / 60_000;
    let seconds = (total_millis % 60_000) / 1_000;
    let millis = total_millis % 1_000;

    format!("{:02}:{:02}:{:02}:{:03}ms", hours, minutes, seconds, millis)
}
