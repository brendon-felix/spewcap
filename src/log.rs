use std::fs::File;
use std::io::Write;
use regex::Regex;
use std::time::{Instant, Duration};
use chrono::Local;
use crate::utils::{ansi_regex, print_separator};

pub struct Log {
    pub file: File,
    pub enabled: bool,
    timestamps: bool,
    ansi_regex: Regex,
    start_time: Instant,
}
impl Log {
    pub fn new(timestamps: bool) -> Result<Self, std::io::Error> {
        let file = create_log_file()?;
        let ansi_regex = ansi_regex();
        let start_time = Instant::now();
        print_separator("Started new log");
        Ok(Log {
            file,
            enabled: true,
            timestamps,
            ansi_regex,
            start_time,
        })
    }
    
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if self.enabled {
            let _ = self.write_line("=== Resumed logging ==\n");
            print_separator("Resumed logging");
        } else {
            let _ = self.write_line("=== Paused  logging ==\n");
            print_separator("Paused logging");
        }
    }

    pub fn write_line(&mut self, raw_line: &str) {
        let mut line = self.ansi_regex.replace_all(raw_line, "").to_string();
        if self.timestamps {
            let timestamp = Log::create_timestamp(self.start_time.elapsed());
            line = format!("[{}] {}", timestamp, line);
        }
        self.file.write_all(line.as_bytes()).expect("Failed to write to log");
    }

    pub fn create_timestamp(duration: Duration) -> String {
        let total_millis = duration.as_millis();
        let hours = total_millis / 3_600_000;
        let minutes = (total_millis % 3_600_000) / 60_000;
        let seconds = (total_millis % 60_000) / 1_000;
        let millis = total_millis % 1_000;
    
        format!("{:02}:{:02}:{:02}:{:03}ms", hours, minutes, seconds, millis)
    }

}


fn create_log_file() -> Result<File, std::io::Error> {
    let filename = format!("log_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
    let file_path = format!("{}", filename);
    File::create(file_path)
}
