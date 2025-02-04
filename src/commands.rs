use anyhow::{Context, Result};
// use serialport5;
// use std::time::Duration;
// use std::fs::{self, File};
// use std::io::{self, Write};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
// use colored::*;
// use rfd::FileDialog;


use std::sync::{Arc, Mutex};
// use crate::state::{self, State};
use crate::state::State;
use crate::log::Log;
use crate::utils::*;

// use crate::{clear_console, print_separator};

// pub fn run_command_thread(state: Arc<Mutex<State>>) -> Result<()> {
//     // let mut state = state.lock().unwrap();
//     command_loop(state)?;
//     Ok(())
// }

pub fn command_loop(shared_state: Arc<Mutex<State>>) -> Result<()> {
    enable_raw_mode().context("couldn't enabled raw mode")?;
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                if kind == KeyEventKind::Press {
                    match code {
                        KeyCode::Char('c') => clear_console(),
                        KeyCode::Char('d') => wipe_output_file(),
                        // KeyCode::Char('l') => list_ports(),
                        KeyCode::Char('l') => toggle_logging(&shared_state),
                        KeyCode::Char('q') => quit(),
                        // KeyCode::Char('q') => state::quit(&shared_state),
                        // KeyCode::Char('q') => return Ok(()),
                        // KeyCode::Char('s') => save(&config.log_folder),
                        KeyCode::Char('p') => set_port(),
                        KeyCode::Char('h') => help_message(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn wipe_output_file() {
    // File::create("log.txt").expect("Failed to truncate the file");
    // print_separator("Output file wiped");
    todo!()
}

fn help_message() {
    print_separator("Help");
    println!("Press the following keys to execute commands:");
    println!("");
    println!("- c: Clear the console");
    // println!("- d: Wipe the log");
    // println!("- l: List available ports");
    // println!("- l: Start/Stop serial logging");
    println!("- q: Quit the application");
    // println!("- s: Save the log");
    // println!("- p: Set the current port");
    println!("- h: Display this help message");
    // println!("");
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

fn set_port() {
    print_separator("Set serial port");
    
}


pub fn toggle_logging(shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    match state.log {
        None => {
            match Log::new() {
                Ok(log) => {
                    state.log = Some(log);
                }
                _ => print_separator("!! Failed to create log file !!"),
            }
        }
        Some(ref mut log) => {
            log.toggle();
        }
    }
}



// fn clear_console() {
//     let _ = std::process::Command::new("cmd").args(["/c", "cls"]).status();
// }

// fn print_separator(text: &str) {
//     if let Some((width, _)) = term_size::dimensions() {
//         let text_length = text.len();
//         match text_length {
//             n if n >= width - 2 => println!("{}", text),
//             n if n == 0 => {
//                 let separator = "-".repeat(width);
//                 println!("{}", separator);
//             }
//             _ => {
//                 let side_length = (width - text_length - 2) / 2;
//                 let separator = "-".repeat(side_length);
//                 println!("{} {} {}", separator, text, separator);
//             }
//         }
//     } else {
//         println!(
//             "----------------------- {} -----------------------",
//             text
//         );
//     }
// }