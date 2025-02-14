
// TODO: Re-implement wipe log command
// TODO: Update state logic to track if the port is connected
// TODO: Update state logic to quit more gracefully (change command)
// TODO: Implement save as... command
// TODO: Implement set port command
//   TODO: Pause spew while setting port

use std::thread;
// use std::sync::mpsc;

mod settings;
mod state;
mod serial;
mod commands;
mod utils;
mod log;

use utils::clear_console;
use settings::get_settings;
use state::*;

fn main() {
    clear_console();
    let settings = get_settings();
    let state = init_state(settings.timestamps);
    // let (tx, rx) = mpsc::channel();
    let shared_state1 = shared_state(&state);
    let serial_thread = thread::spawn(move || {
        serial::connect_loop(settings, shared_state1);
    });
    let shared_state2 = shared_state(&state);
    let command_thread = thread::spawn(move || {
        commands::command_loop(shared_state2);
    });
    // rx.recv()?;
    let _ = serial_thread.join().map_err(|e| println!("Serial thread panicked: {:?}", e));
    let _ = command_thread.join().map_err(|e| println!("Command thread panicked: {:?}", e));
}
