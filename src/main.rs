
// TODO: Re-implement wipe log command
// TODO: Update state logic to track if the port is connected
// TODO: Update state logic to quit more gracefully (change command)
// TODO: Implement set port command
//   TODO: Pause spew while setting port

mod utils;
mod settings;
mod state;
mod serial;
mod commands;
mod log;

fn main() {
    utils::clear_console();
    let settings = settings::get_settings();
    let state = state::init_state(settings.timestamps);

    let serial_thread = utils::start_thread(settings.clone(), &state, serial::connect_loop);
    let command_thread = utils::start_thread(settings.clone(), &state, commands::command_loop);

    let _ = serial_thread.join().map_err(|e| println!("Serial thread panicked: {:?}", e));
    let _ = command_thread.join().map_err(|e| println!("Command thread panicked: {:?}", e));
}
