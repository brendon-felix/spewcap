use std::sync::{Arc, Mutex};
use std::time::Duration;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::enable_raw_mode;

use crate::state::State;
use crate::settings::Settings;
use crate::utils::{self, get_state, print_message, print_separator, print_warning, print_error};

const POLL_PERIOD: u64 = 100; // milliseconds

pub fn command_loop(settings: Settings, shared_state: Arc<Mutex<State>>) {
    if let Err(e) = enable_raw_mode() {
        print_warning(&format!("Could not enable raw mode: {e}\nSome key commands may not work properly!"))
    }
    loop {
        let result = match poll_for_command() {
            Ok(Some((code, kind))) => {
                if utils::quit_requested(&shared_state) { break; }
                handle_command(code, kind, &settings, &shared_state)
            },
            Ok(None) => Ok(()),
            Err(e) => Err(e)
        };
        if let Err(e) = result {
            print_error(&format!("Command handler error: {e}"));
        }
    }
}

fn poll_for_command() -> Result<Option<(KeyCode, KeyEventKind)>, String> {
    let key_pressed = event::poll(Duration::from_millis(POLL_PERIOD))
        .map_err(|e| format!("Could not poll for key event: {e}"))?;
    let command = if key_pressed {
        let event = event::read()
            .map_err(|e| format!("Could not read key event: {e}"))?;
        match event {
            Event::Key(KeyEvent {code, kind, ..}) => Some((code, kind)),
            _ => None
        }
    } else { None };
    Ok(command)
}

fn handle_command(code: KeyCode, kind: KeyEventKind, settings: &Settings, shared_state: &Arc<Mutex<State>>) -> Result<(), String> {
    if kind == KeyEventKind::Press {
        match code {
            KeyCode::Char('q') => utils::request_quit(settings, shared_state),
            KeyCode::Char('c') => utils::clear_console(),
            KeyCode::Char('p') => toggle_pause_capture(shared_state)?,
            KeyCode::Char('n') => utils::start_new_log(&settings, shared_state),
            KeyCode::Char('s') => utils::save_active_log(&settings, shared_state),
            KeyCode::Char('l') => toggle_pause_logging(shared_state)?,
            KeyCode::Char('h') => help_message(),
            _ => {}
        }
    }
    Ok(())
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

fn toggle_pause_capture(shared_state: &Arc<Mutex<State>>) -> Result<(), String> {
    let mut state = get_state(shared_state)?;

    state.capture_paused = !state.capture_paused;
    if state.capture_paused { print_message(format!("Capture {}", "paused".yellow())); }
        else                { print_message(format!("Capture {}", "resumed".green())); }
    Ok(())
}

fn toggle_pause_logging(shared_state: &Arc<Mutex<State>>) -> Result<(), String> {
    let mut state = get_state(shared_state)?;
    match state.active_log {
        Some(ref mut log) => log.toggle(),
        None => utils::print_warning("No log started! Press `N` to start one")
    }
    Ok(())
}
