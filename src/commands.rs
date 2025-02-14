use std::sync::{Arc, Mutex};
use std::time::Duration;
// use rfd::FileDialog;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{enable_raw_mode, disable_raw_mode},
};

use crate::{
    state::State,
    utils::*,
};

pub fn command_loop(shared_state: Arc<Mutex<State>>) {
    enable_raw_mode().expect("Could not enable raw mode");
    loop {
        if event::poll(Duration::from_millis(100)).expect("Could not poll for key events") {
            if let Event::Key(KeyEvent { code, kind, .. }) = event::read().expect("Could not read key event") {
                if kind == KeyEventKind::Press {
                    match code {
                        KeyCode::Char('q') => quit(),
                        KeyCode::Char('c') => clear_console(),
                        KeyCode::Char('l') => create_new_log(&shared_state),
                        KeyCode::Char('p') => toggle_pause_logging(&shared_state),
                        KeyCode::Char('d') => wipe_active_log(),
                        // KeyCode::Char('s') => save(&config.log_folder),
                        KeyCode::Char('h') => help_message(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn wipe_active_log() {
    // File::create("log.txt").expect("Failed to truncate the file");
    // print_separator("Output file wiped");
    todo!()
}

fn help_message() {
    print_separator("Help");
    println!("Press the following keys to execute commands:");
    println!("");
    println!("- `Q`: Quit the application");
    println!("- `C`: Clear the console");
    println!("- `L`: Start a new log");
    println!("- `L`: Pause/resume logging");
    println!("- `D`: Wipe the active log");
    println!("- `S`: Save active log as...");
    println!("- `H`: Display this help message");
    println!("");
    print_separator("");
}

fn quit() {
    // println!("QUITTING");
    disable_raw_mode().expect("Could not diable raw mode");
    std::process::exit(0);
}

// fn save(_destination_path: &String) {
//     // print_separator("Save output file");
//     // if let Some(destination_path) = FileDialog::new()
//     //     .add_filter("log", &["txt", "log"])
//     //     .set_title("Save Log File")
//     //     .set_directory(destination_path)
//     //     .set_file_name("log.txt")
//     //     .save_file()
//     // {
//     //     print_separator("");
//     //     match fs::copy("log.txt", &destination_path) {
//     //         Ok(_) => println!("Saved {}", destination_path.display()),
//     //         Err(e) => println!("Error copying file: {}", e),
//     //     }
//     // } else {
//     //     print_separator("");
//     //     println!("Save operation was canceled");
//     // }

//     // // let destination_path = "output.txt";
    
//     // print_separator("");

//     todo!()
// }

pub fn create_new_log(shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    state.log = try_create_log(state.timestamps);
}

pub fn toggle_pause_logging(shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    match state.log {
        None => {
            print_separator("!! No log started! Press `L` to start one !!")
        }
        Some(ref mut log) => {
            log.toggle();
        }
    }
}
