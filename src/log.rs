use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::fs::copy;
use regex::Regex;
use std::time::{Instant, Duration};
use chrono::Local;
use colored::Colorize;

use crate::utils::{ansi_regex, print_message};

pub struct Log {
    pub file: File,
    pub filename: String,
    pub file_path: PathBuf,
    pub enabled: bool,
    pub unsaved_changes: bool,
    prepend_timestamps: bool,
    ansi_regex: Regex,
    start_time: Instant,
}
impl Log {
    pub fn new(prepend_timestamps: bool) -> Result<Self, std::io::Error> {
        let filename = format!("log_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
        let file_path = PathBuf::from(&filename);
        let file = File::create(&file_path)?;
        let ansi_regex = ansi_regex();
        let start_time = Instant::now();
        Ok(Log {
            file,
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

    pub fn write_line(&mut self, raw_line: &str) {
        let mut line = self.ansi_regex.replace_all(raw_line, "").to_string();
        if self.prepend_timestamps {
            let timestamp = create_timestamp(self.start_time.elapsed());
            line = format!("[{}] {}", timestamp, line);
        }
        self.file.write_all(line.as_bytes()).expect("Failed to write to log");
        self.unsaved_changes = true;
    }

    pub fn save_as(&mut self, new_file_path: &PathBuf) {
        match copy(&self.file_path, new_file_path) {
            Ok(_) => {
                self.unsaved_changes = false;
                println!("Saved to {}", new_file_path.display())
            },
            Err(e) => eprintln!("Error saving file: {}", e)
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


