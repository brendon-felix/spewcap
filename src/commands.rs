use std::sync::{Arc, Mutex};
use std::time::Duration;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::enable_raw_mode;

use crate::state::State;
use crate::settings::Settings;
use crate::utils::{self, print_separator, print_message};

const POLL_PERIOD: u64 = 100; // milliseconds

pub fn command_loop(settings: Settings, shared_state: Arc<Mutex<State>>) {
    enable_raw_mode().expect("Could not enable raw mode");
    loop {
        if let Some((code, kind)) = poll_for_command() {
            if utils::quit_requested(&shared_state) {
                break;
            }
            handle_key_event(code, kind, &settings, &shared_state);
        }
    }
}

fn poll_for_command() -> Option<(KeyCode, KeyEventKind)> {
    if event::poll(Duration::from_millis(POLL_PERIOD)).expect("Could not poll for key events") {
        if let Event::Key(KeyEvent { code, kind, ..
        }) = event::read().expect("Could not read key event") {
            return Some((code, kind));
        }
    }
    None
}

fn handle_key_event(code: KeyCode, kind: KeyEventKind, settings: &Settings, shared_state: &Arc<Mutex<State>>) {
    if kind == KeyEventKind::Press {
        match code {
            KeyCode::Char('q') => utils::request_quit(settings, shared_state),
            KeyCode::Char('c') => utils::clear_console(),
            KeyCode::Char('p') => toggle_pause_capture(shared_state),
            KeyCode::Char('n') => utils::start_new_log(&settings, shared_state),
            KeyCode::Char('s') => utils::save_active_log(&settings, shared_state),
            KeyCode::Char('l') => toggle_pause_logging(shared_state),
            KeyCode::Char('h') => help_message(),
            _ => {}
        }
    }
}

fn help_message() {
    print_separator();
    println!("Help: Use the following keys to execute commands:");
    println!("");
    println!("- `Q`: Quit the application");
    println!("- `C`: Clear the console");
    println!("- `P`: Pause/resume capture");
    println!("- `N`: Start a new log");
    println!("- `L`: Pause/resume logging");
    println!("- `S`: Save active log as...");
    println!("- `H`: Display this help message");
    println!("");
    print_separator();
}

fn toggle_pause_capture(shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    state.capture_paused = !state.capture_paused;
    if state.capture_paused { print_message(format!("Capture {}", "paused".yellow())); }
        else                { print_message(format!("Capture {}", "resumed".green())); }
}

fn toggle_pause_logging(shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    match state.active_log {
        Some(ref mut log) => log.toggle(),
        None => utils::print_warning("No log started! Press `N` to start one")
    }
}
