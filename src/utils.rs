use colored::Colorize;
use crossterm::terminal;
use crossterm::execute;
// use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use regex::Regex;
use rfd::FileDialog;
use serialport5::{available_ports, SerialPortType};
use std::fmt::Display;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::constants::SIGNAL_MONITOR_SLEEP;
use crate::log::LogFile;
use crate::settings::Settings;
use crate::state::{LogState, State};
use crate::error::{Result, SpewcapError};

const ANSI_REGEX: &str = r"\x1b\[[0-9;]*[mK]";

pub fn initialize_app(args: crate::settings::Args) -> Result<(crate::settings::Config, State)> {
    let config = crate::settings::get_config(args)?;
    let state = crate::state::init_state();
    
    if let Err(e) = setup_signal_handlers(state.clone()) {
        eprintln!("Warning: Failed to set up signal handlers: {e}");
    }
    
    Ok((config, state))
}

#[cfg(unix)]
pub fn setup_signal_handlers(state: State) -> Result<()> {
    use signal_hook::consts::signal::*;
    let signals = &[SIGINT, SIGTERM, SIGQUIT];
    setup_signal_handlers_common(state, signals)
}

#[cfg(windows)]
pub fn setup_signal_handlers(state: State) -> Result<()> {
    use signal_hook::consts::signal::*;
    let signals = &[SIGINT, SIGTERM];
    setup_signal_handlers_common(state, signals)
}

fn setup_signal_handlers_common(state: State, signals: &[i32]) -> Result<()> {
    use signal_hook::flag;
    use std::sync::atomic::AtomicBool;

    let term_flag = Arc::new(AtomicBool::new(false));
    
    for &signal in signals {
        flag::register(signal, Arc::clone(&term_flag))
            .map_err(|e| SpewcapError::Signal(format!("Failed to register signal {}: {}", signal, e)))?;
    }
    spawn_signal_monitoring_thread(state, term_flag);
    Ok(())
}

fn spawn_signal_monitoring_thread(state: State, term_flag: Arc<std::sync::atomic::AtomicBool>) {
    let state_clone = state.clone();
    let term_flag_clone = Arc::clone(&term_flag);
    std::thread::spawn(move || {
        monitor_signals(state_clone, term_flag_clone);
    });
}

fn monitor_signals(state: State, term_flag: Arc<std::sync::atomic::AtomicBool>) {
    loop {
        if term_flag.load(Ordering::Relaxed) {
            handle_termination_signal(&state);
            break;
        }
        if state.quit_requested.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(SIGNAL_MONITOR_SLEEP);
    }
}

fn handle_termination_signal(state: &State) {
    eprintln!("\nReceived termination signal, shutting down gracefully...");
    emergency_cleanup_logs(state);
    request_quit_with_state(state);
}

pub fn get_log_state(shared_state: &State) -> Result<MutexGuard<'_, LogState>> {
    shared_state
        .log_state
        .lock()
        .map_err(|e| SpewcapError::Log(format!("Failed to acquire lock on log state: {e}")))
}

pub fn sleep_ms(num_ms: u64) {
    std::thread::sleep(Duration::from_millis(num_ms));
}

// pub fn enter_alternate_screen() -> Result<()> {
//     execute!(std::io::stdout(), EnterAlternateScreen)
//         .map_err(|e| SpewcapError::Terminal(format!("Failed to enter alternate screen: {e}")))?;
//     execute!(std::io::stdout(), crossterm::cursor::MoveTo(0, 0))
//         .map_err(|e| SpewcapError::Terminal(format!("Failed to move cursor to top: {e}")))?;
//     Ok(())
// }

// pub fn leave_alternate_screen() -> Result<()> {
//     execute!(std::io::stdout(), LeaveAlternateScreen)
//         .map_err(|e| SpewcapError::Terminal(format!("Failed to leave alternate screen: {e}")))?;
//     Ok(())
// }

pub fn clear_console() {
    if let Err(e) = execute!(std::io::stdout(), terminal::Clear(terminal::ClearType::All)) {
        print_error(&format!("Failed to clear console: {e}"));
    }
    if let Err(e) = execute!(std::io::stdout(), crossterm::cursor::MoveTo(0, 0)) {
        print_error(&format!("Failed to move cursor: {e}"));
    }
}

pub fn print_welcome() {
    println!(
        r"
 __ _  __    __ _  _
(_ |_)|_ | |/  |_||_)
__)|  |__|^|\__| ||

==================================
Press `H` for help and `Q` to quit
==================================
"
    );
}

pub fn ansi_regex() -> Regex {
    Regex::new(ANSI_REGEX).unwrap()
}
pub fn reset_ansi() {
    print!("\x1b[0m")
}

pub fn start_thread<F>(settings: Settings, state: &State, task: F) -> JoinHandle<Result<()>>
where
    F: Fn(Settings, State) -> Result<()> + Send + 'static,
{
    let state_clone = Arc::clone(&state);
    std::thread::spawn(move || {
        task(settings, state_clone)
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

pub fn request_quit(settings: &Settings, shared_state: &State) {
    print_message("Quitting...");
    terminal::disable_raw_mode().unwrap_or_else(|_| {
        print_error("Failed to disable raw terminal mode");
    });
    
    emergency_cleanup_logs(shared_state);
    
    let log_state = match get_log_state(shared_state) {
        Ok(state) => state,
        Err(e) => {
            print_error(&format!("Failed to acquire lock on log state during quit: {e}"));
            shared_state.quit_requested.store(true, Ordering::Relaxed);
            return;
        }
    };
    let need_save = log_state
        .active_log
        .as_ref()
        .map_or(false, |log| log.has_unsaved_changes());

    drop(log_state);

    if need_save {
        save_active_log(settings, shared_state);
    }
    shared_state.quit_requested.store(true, Ordering::Relaxed);
}
pub fn quit_requested(state: &State) -> bool {
    state.quit_requested.load(Ordering::Relaxed)
}

pub fn request_quit_with_state(shared_state: &State) {
    shared_state.quit_requested.store(true, Ordering::Relaxed);
    if let Err(_) = terminal::disable_raw_mode() {}
}

pub fn start_new_log(settings: &Settings, shared_state: &State) -> Result<()> {
    let mut log_state = shared_state.log_state.lock()
        .map_err(|e| SpewcapError::Log(format!("Failed to acquire lock: {e}")))?;
    match LogFile::new(settings.timestamps) {
        Ok(log) => {
            let filename = log.get_filename().to_string();
            log_state.active_log = Some(log);
            print_success(&format!("Started new log file: {}", filename));
            Ok(())
        }
        Err(e) => {
            print_error("Failed to create log file");
            Err(SpewcapError::Log(format!("Failed to create log file: {e}")))
        }
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
pub fn save_active_log(settings: &Settings, shared_state: &State) {
    let mut log_state = match get_log_state(shared_state) {
        Ok(state) => state,
        Err(e) => {
            print_error(&format!("Failed to acquire lock on log state during save: {e}"));
            return;
        }
    };
    
    match log_state.active_log {
        Some(ref mut log) => {
            save_log_with_dialog(log, settings);
        }
        None => print_warning("No log started! Press `L` to start one"),
    }
}

fn save_log_with_dialog(log: &mut LogFile, settings: &Settings) {
    let _ = log.force_flush();
    
    if !log.has_unsaved_changes() {
        print_warning("No unsaved changes to save!");
        return;
    }
    
    let log_path = match run_file_dialog(log.get_filename(), &settings.log_folder) {
        Some(path) => path,
        None => {
            print_warning("Save operation was canceled!");
            return;
        }
    };
    
    match log.save_as_and_keep(&log_path) {
        Ok(()) => {
            print_success(&format!("Saved log to {}", log_path.display()));
        }
        Err(e) => print_error(&format!("Failed to save log: {e}")),
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

pub fn list_ports() -> Result<()> {
    let ports = available_ports()
        .map_err(|e| SpewcapError::SerialPort(e))?;
    if ports.is_empty() {
        println!("No serial ports found!");
        return Ok(());
    }
    
    println!("Available serial ports:");
    for port in ports {
        let mut description = format!("  {}", port.port_name);
        if let SerialPortType::UsbPort(info) = &port.port_type {
            let manufacturer = info.manufacturer.as_deref().unwrap_or("Unknown");
            let product = info.product.as_deref().unwrap_or("Unknown");
            if manufacturer != "Unknown" || product != "Unknown" {
                description = format!("  {} - {} {}", port.port_name, manufacturer, product);
            }
        }
        println!("{}", description);
    }
    Ok(())
}

pub fn cleanup_logs(shared_state: &State) {
    if let Ok(mut log_state) = get_log_state(shared_state) {
        if let Some(ref mut log) = log_state.active_log {
            if let Err(e) = log.ensure_flushed() {
                eprintln!("Warning: Failed to flush log during cleanup: {e}");
            }
            
            if log.has_unsaved_changes() {
                print_warning("Active log has unsaved changes. It will be cleaned up unless saved.");
            }
        }
    }
}

pub fn emergency_cleanup_logs(shared_state: &State) {
    match shared_state.log_state.try_lock() {
        Ok(mut log_state) => {
            if let Some(ref mut log) = log_state.active_log {
                let _ = log.ensure_flushed();
                let _ = log.cleanup_temp_file();
            }
        }
        Err(_) => {
            eprintln!("Warning: Could not acquire log state lock during emergency cleanup");
        }
    }
}
