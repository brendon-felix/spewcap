use std::time::Duration;
use std::fmt::Display;
use std::ops::Deref;
use regex::Regex;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::path::PathBuf;
use colored::Colorize;
use rfd::FileDialog;
use crossterm::terminal;

use crate::state::State;
use crate::settings::Settings;
use crate::log::Log;

const ANSI_REGEX: &str = r"\x1b\[[0-9;]*[mK]";

pub fn get_state(shared_state: &Arc<Mutex<State>>) -> Result<MutexGuard<State>, String> {
    shared_state.lock().map_err(|e| format!("Failed to acquire lock on shared state: {e}"))
}

pub fn sleep(num_ms: u64) {
    std::thread::sleep(Duration::from_millis(num_ms));
}

pub fn clear_console() {
    let _ = std::process::Command::new("cmd").args(["/c", "cls"]).status();
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

// pub fn print_separator<T: ToString + Deref<Target = str> + Display>(text: T) {
    
//     if let Some((width, _)) = term_size::dimensions() {
//         let re = ansi_regex();
//         let length = re.replace_all(&text, "").len();
//         match length {
//             n if n >= width - 2 => println!("{}", text),
//             n if n == 0 => {
//                 let separator = "-".repeat(width);
//                 reset_ansi();
//                 println!("{}", separator);
//             }
//             _ => {
//                 let side_length = (width - length - 2) / 2;
//                 let separator = "-".repeat(side_length);
//                 reset_ansi();
//                 println!("{} {} {}", separator, text, separator);
//             }
//         }
//     } else {
//         println!("----------------------- {} -----------------------", text);
//     }
// }


pub fn print_separator() {
    reset_ansi();
    if let Some((width, _)) = term_size::dimensions() {
        let separator = "-".repeat(width);
        println!("{}", separator);
    } else {
        println!("----------------------------------------------");
    }
}
pub fn print_message<T: ToString + Deref<Target = str> + Display>(message: T) {
    print_separator();
    println!("{}", message);
    print_separator();
}
pub fn print_success(message: &str) {
    let full_message = format!("{}: {}", "Success".green(), message);
    print_message(full_message);
}
pub fn print_warning(message: &str) {
    let full_message = format!("{}: {}", "Warning".yellow(), message);
    print_message(full_message);
}
pub fn print_error(message: &str) {
    let full_message = format!("{}: {}", "Error".red(), message);
    print_message(full_message);
}


pub fn request_quit(settings: &Settings, shared_state: &Arc<Mutex<State>>) {
    print_message("Quitting...");
    terminal::disable_raw_mode().unwrap_or_else(|_| {
        print_error("Failed to disable raw terminal mode");
    });
    let mut state = shared_state.lock().unwrap();
    let need_save = state.active_log.as_ref().map_or(false, |log| log.unsaved_changes);
    if need_save {
        save_active_log(settings, shared_state);
    }
    state.quit_requested = true;
}
pub fn quit_requested(state: &Arc<Mutex<State>>) -> bool {
    let state = state.lock().unwrap();
    state.quit_requested
}

pub fn start_new_log(settings: &Settings, shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    match Log::new(settings.timestamps) {
        Ok(log) => {
            state.active_log = Some(log);
            print_success(&format!("Started new log file: {}", state.active_log.as_ref().unwrap().filename));
        },
        _ => print_error("Failed to create log file")
    }
}
pub fn run_file_dialog(filename: &str, directory: &Option<PathBuf>) -> Option<PathBuf> {
    let dialog = FileDialog::new();

    let dialog = if let Some(path) = directory {
        dialog.set_directory(path)
    } else {
        dialog
    };
    dialog
        .add_filter("log", &["txt", "log"])
        .set_title("Save Log File")
        .set_file_name(filename)
        .save_file()
}
pub fn save_active_log(settings: &Settings, shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    match state.active_log {
        Some(ref mut log) => {
            if log.unsaved_changes {
                match run_file_dialog(&log.filename, &settings.log_folder) {
                    Some(log_path) => {
                        log.save_as(&log_path);
                        print_success(&format!("Saved log to {}", log_path.display()));
                    },
                    None => print_warning("Save operation was canceled!"),
                }
            } else {
                print_warning("No unsaved changes to save!");
            }
        },
        None => print_warning("No log started! Press `L` to start one"),
    }
}

pub fn get_exe_directory() -> Option<PathBuf> {
    if let Some(exe_path) = std::env::current_exe().ok() {
        if let Some(exe_directory) = exe_path.parent() {
            return Some(exe_directory.to_path_buf());
        }
    }
    None
}
pub fn get_curr_directory() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
