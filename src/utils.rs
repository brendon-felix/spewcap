use std::time::Duration;
use std::fmt::Display;
use std::ops::Deref;
use regex::Regex;
use std::{sync::{Arc, Mutex}, thread::JoinHandle};
// use colored::Colorize;

use crate::state::State;
use crate::settings::Settings;

const ANSI_REGEX: &str = r"\x1b\[[0-9;]*[mK]";

pub fn sleep(num_ms: u64) {
    std::thread::sleep(Duration::from_millis(num_ms));
}

pub fn clear_console() {
    let _ = std::process::Command::new("cmd").args(["/c", "cls"]).status();
}

pub fn ansi_regex() -> Regex {
    Regex::new(ANSI_REGEX).unwrap()
}

pub fn reset_ansi() {
    print!("\x1b[0m")
}

pub fn start_thread<F>(settings: Settings, state: &Arc<Mutex<State>>, task: F) -> JoinHandle<()>
  where
    F: Fn(Settings, Arc<Mutex<State>>) + Send + 'static,
    {
    let state_clone = Arc::clone(&state);
    std::thread::spawn(move || {
        task(settings, state_clone);
    })
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

pub fn start_quit(state: &Arc<Mutex<State>>) {
    print_separator("QUITTING SPEWCAP");
    let mut state = state.lock().unwrap();
    state.quitting = true;
}

pub fn did_quit(state: &Arc<Mutex<State>>) -> bool {
    let state = state.lock().unwrap();
    state.quitting
}

pub fn print_welcome() {
    println!(r"
 __ _  __    __ _  _ 
(_ |_)|_ | |/  |_||_)
__)|  |__|^|\__| ||  

==================================
Press `H` for help and `Q` to quit
==================================
");
}
