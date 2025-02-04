use anyhow::{Context, Result};
use std::time::Duration;
use std::fmt::Display;
// use unicode_segmentation::UnicodeSegmentation;
use std::ops::Deref;
// use std::fs::{self, File};
use regex::Regex;
use std::fs::File;

use chrono::Local;

const ANSI_REGEX: &str = r"\x1b\[[0-9;]*[mK]";
pub fn ansi_regex() -> Regex {
    Regex::new(ANSI_REGEX).unwrap()
}

pub fn sleep(num_ms: u64) {
    std::thread::sleep(Duration::from_millis(num_ms));
}

pub fn clear_console() {
    let _ = std::process::Command::new("cmd").args(["/c", "cls"]).status();
}

pub fn reset_ansi() {
    print!("\x1b[0m")
}

pub fn print_separator<T: ToString + Deref<Target = str> + Display>(text: T) {
    reset_ansi();
    if let Some((width, _)) = term_size::dimensions() {
        let re = ansi_regex();
        let length = re.replace_all(&text, "").len();
        match length {
            n if n >= width - 2 => println!("{}", text),
            n if n == 0 => {
                let separator = "-".repeat(width);
                println!("{}", separator);
            }
            _ => {
                let side_length = (width - length - 2) / 2;
                let separator = "-".repeat(side_length);
                println!("{} {} {}", separator, text, separator);
            }
        }
    } else {
        println!("----------------------- {} -----------------------", text);
    }
}

pub fn create_log_file() -> Result<File> {
    let filename = format!("log_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
    let file_path = format!("{}", filename);
    // if fs::metadata(file_path).is_ok() {
    //     fs::remove_file(file_path).context("Failed to remove existing output file")?;
    // }
    let file = File::create(file_path).context("Failed to open output file")?;
    Ok(file)
}
