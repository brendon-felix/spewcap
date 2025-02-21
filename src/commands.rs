use std::sync::{Arc, Mutex};
use std::time::Duration;
use rfd::FileDialog;
use std::path::PathBuf;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{enable_raw_mode, disable_raw_mode},
};

use crate::{
    state::State,
    settings::Settings,
    utils::{self, print_separator, start_quit, did_quit},
    log,
};

const POLL_PERIOD: u64 = 100; // milliseconds

pub fn command_loop(settings: Settings, shared_state: Arc<Mutex<State>>) {
    // println!("COMMAND LOOP");
    enable_raw_mode().expect("Could not enable raw mode");
    loop {
        if let Some((code, kind)) = poll_for_command() {
            if did_quit(&shared_state) {
                disable_raw_mode().expect("Could not disable raw mode");
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
            KeyCode::Char('q') => {
                disable_raw_mode().expect("Could not disable raw mode");
                start_quit(shared_state);
            },
            // KeyCode::Char('x') => quit(shared_state),
            KeyCode::Char('c') => utils::clear_console(),
            KeyCode::Char('l') => create_new_log(shared_state),
            KeyCode::Char('p') => toggle_pause_logging(shared_state),
            // KeyCode::Char('d') => wipe_active_log(),
            KeyCode::Char('s') => save(&settings, shared_state),
            KeyCode::Char('h') => help_message(),
            _ => {}
        }
    }
}

// fn wipe_active_log() {
//     print_separator("Output file wiped");
//     // TODO
// }

fn help_message() {
    print_separator("HELP");
    println!("Use the following keys to execute commands:");
    println!("");
    println!("- `Q`: Quit the application");
    println!("- `C`: Clear the console");
    println!("- `L`: Start a new log");
    println!("- `P`: Pause/resume logging");
    // println!("- `D`: Wipe the active log");
    println!("- `S`: Save active log as...");
    println!("- `H`: Display this help message");
    println!("");
    print_separator("");
}

// fn quit(shared_state: &Arc<Mutex<State>>) {
//     println!("QUITTING");
    
//     println!("Set quitting to true");
//     // std::process::exit(0);
// }

fn save(settings: &Settings, shared_state: &Arc<Mutex<State>>) {
    let state = shared_state.lock().unwrap();
    if let Some(ref log) = state.log {
        print_separator("SAVE OUTPUT FILE");
        if let Some(log_path) = select_log_path(&log.filename, &settings.log_folder) { 
            match log.save_as(&log_path) {
                Ok(_) => println!("Saved to {}", log_path.display()),
                Err(e) => eprintln!("Error saving file: {}", e)
            }
        } else {
            println!("Save operation was canceled!");
        }
    } else {
        print_separator("");
        println!("No log started! Press `L` to start one");
    }
    print_separator("");
}

fn select_log_path(filename: &str, log_folder_setting: &Option<PathBuf>) -> Option<PathBuf> {
    let dialog = FileDialog::new();

    let dialog = if let Some(path) = log_folder_setting {
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

fn create_new_log(shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    state.log = log::try_create_log(state.timestamps);
}

fn toggle_pause_logging(shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    match state.log {
        None => {
            print_separator("");
            println!("No log started! Press `L` to start one");
            print_separator("");
        }
        Some(ref mut log) => {
            log.toggle();
        }
    }
}
