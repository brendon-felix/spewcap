use chrono::Local;
use colored::Colorize;
use regex::Regex;
use std::fs::copy;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs::File, io::BufWriter};

use crate::constants::{
    LINE_BUFFER_SHRINK_TARGET, LINE_BUFFER_SHRINK_THRESHOLD, LOG_FLUSH_INTERVAL,
    LOG_LINE_BUFFER_INITIAL_CAPACITY, LOG_WRITER_BUFFER_CAPACITY, MILLIS_PER_HOUR,
    MILLIS_PER_MINUTE, MILLIS_PER_SECOND, TIMESTAMP_BUFFER_INITIAL_CAPACITY,
    TIMESTAMP_BUFFER_SHRINK_TARGET, TIMESTAMP_BUFFER_SHRINK_THRESHOLD,
};
use crate::utils::{ansi_regex, print_message};
use crate::validation;
use crate::error::Result;



pub struct LogFile {
    inner: Log,
    temp_file_path: PathBuf,
    cleanup_on_drop: bool,
}

impl LogFile {
    pub fn new(prepend_timestamps: bool) -> std::result::Result<Self, std::io::Error> {
        let inner = Log::new(prepend_timestamps)?;
        let temp_file_path = inner.file_path.clone();
        Ok(LogFile {
            inner,
            temp_file_path,
            cleanup_on_drop: true,
        })
    }

    pub fn disable_cleanup(&mut self) {
        self.cleanup_on_drop = false;
    }

    pub fn cleanup_temp_file(&mut self) -> std::io::Result<()> {
        if self.cleanup_on_drop && self.temp_file_path.exists() {
            self.inner.force_flush()?;
            std::fs::remove_file(&self.temp_file_path)?;
            self.cleanup_on_drop = false;
            print_message("Temporary log file cleaned up".green());
        }
        Ok(())
    }

    pub fn ensure_flushed(&mut self) -> std::io::Result<()> {
        self.inner.force_flush()
    }

    pub fn save_as_and_keep(&mut self, new_file_path: &PathBuf) -> Result<()> {
        self.inner.force_flush().map_err(|e| crate::error::SpewcapError::Io(e))?;
        let result = self.inner.save_as(new_file_path);
        if result.is_ok() {
            self.disable_cleanup();
        }
        
        result
    }
}

impl Drop for LogFile {
    fn drop(&mut self) {
        if let Err(e) = self.inner.force_flush() {
            eprintln!("Warning: Failed to flush log during cleanup: {e}");
        }
        if self.cleanup_on_drop && self.temp_file_path.exists() {
            match std::fs::remove_file(&self.temp_file_path) {
                Ok(_) => {
                    if std::env::var("SPEWCAP_DEBUG").is_ok() {
                        print_message("Temporary log file cleaned up on drop".green());
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to clean up temporary log file '{}': {}", 
                             self.temp_file_path.display(), e);
                }
            }
        }
    }
}

impl std::ops::Deref for LogFile {
    type Target = Log;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for LogFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Log {
    writer: BufWriter<File>,
    filename: String,
    file_path: PathBuf,
    enabled: bool,
    unsaved_changes: bool,
    prepend_timestamps: bool,
    ansi_regex: Regex,
    start_time: Instant,

    // performance optimizations
    timestamp_buffer: String,
    line_buffer: String,
    flush_counter: usize,
}
impl Log {
    pub fn new(prepend_timestamps: bool) -> std::result::Result<Self, std::io::Error> {
        let filename = format!("log_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
        let file_path = PathBuf::from(&filename);
        let file = File::create(&file_path)?;
        let writer = BufWriter::with_capacity(LOG_WRITER_BUFFER_CAPACITY, file);
        let ansi_regex = ansi_regex();
        let start_time = Instant::now();
        Ok(Log {
            writer,
            filename,
            file_path,
            enabled: true,
            unsaved_changes: false,
            prepend_timestamps,
            ansi_regex,
            start_time,
            timestamp_buffer: String::with_capacity(TIMESTAMP_BUFFER_INITIAL_CAPACITY),
            line_buffer: String::with_capacity(LOG_LINE_BUFFER_INITIAL_CAPACITY),
            flush_counter: 0,
        })
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if self.enabled {
            print_message(format!("Logging {}", "resumed".green()));
        } else {
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

        // avoid regex if no ANSI codes detected (performance optimization)
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

        // batch flush (performance optimization)
        if self.flush_counter >= LOG_FLUSH_INTERVAL {
            let _ = self.writer.flush();
            self.flush_counter = 0;
        }

        // Prevent string buffer from growing indefinitely
        if self.line_buffer.capacity() > LINE_BUFFER_SHRINK_THRESHOLD {
            self.line_buffer.shrink_to(LINE_BUFFER_SHRINK_TARGET);
        }
        if self.timestamp_buffer.capacity() > TIMESTAMP_BUFFER_SHRINK_THRESHOLD {
            self.timestamp_buffer.shrink_to(TIMESTAMP_BUFFER_SHRINK_TARGET);
        }
    }

    pub fn force_flush(&mut self) -> std::io::Result<()> {
        self.flush_counter = 0;
        self.writer.flush()
    }

    pub fn save_as(&mut self, new_file_path: &PathBuf) -> Result<()> {
        let path_str = new_file_path.to_string_lossy();
        validation::validate_file_path(&path_str)?;
        
        match copy(&self.file_path, new_file_path) {
            Ok(_) => {
                self.unsaved_changes = false;
                println!("Saved to {}", new_file_path.display());
                Ok(())
            }
            Err(e) => {
                eprintln!("Error saving file: {e}");
                Err(crate::error::SpewcapError::Io(e))
            }
        }
    }

    fn create_timestamp_in_buffer(&mut self, duration: Duration) {
        self.timestamp_buffer.clear();
        let total_millis = duration.as_millis();
        let hours = total_millis / MILLIS_PER_HOUR;
        let minutes = (total_millis % MILLIS_PER_HOUR) / MILLIS_PER_MINUTE;
        let seconds = (total_millis % MILLIS_PER_MINUTE) / MILLIS_PER_SECOND;
        let millis = total_millis % MILLIS_PER_SECOND;

        use std::fmt::Write;
        write!(
            self.timestamp_buffer,
            "{:02}:{:02}:{:02}:{:03}ms",
            hours, minutes, seconds, millis
        )
        .expect("Failed to write timestamp");
    }
}
