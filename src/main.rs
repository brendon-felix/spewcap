
// TODO: Implement command to pause terminal output as well
// TODO: Fix error handling (when the command loop panics, the serial loop can't end)
// TODO: Print "saved to ..." message before actually quitting when a log is active
// TODO: Re-implement wipe log command
// TODO: Update state logic to track if the port is connected
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
    let mut config = settings::get_config();
    if !config.disable_welcome.unwrap_or(false) {
        utils::print_welcome();
    }
    config.select_missing();
    let settings = settings::get_settings(&config);
    let state = state::init_state(&settings);
    if config.log_on_start.unwrap_or(false) {
        let mut state = state.lock().unwrap();
        state.log = log::try_create_log(settings.timestamps);
    }

    let serial_thread = utils::start_thread(settings.clone(), &state, serial::connect_loop);
    let command_thread = utils::start_thread(settings.clone(), &state, commands::command_loop);

    let _ = serial_thread.join().map_err(|e| println!("Serial thread panicked: {:?}", e));
    let _ = command_thread.join().map_err(|e| println!("Command thread panicked: {:?}", e));
}

