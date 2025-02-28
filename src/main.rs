// TODO: Remove temp log files after they are saved OR experiment with large String buffer
//   TODO: If still using file buffers, implement command to wipe the active file
// TODO: Fix error handling (when the command loop panics, the serial loop can't end)
//   TODO: Try and remove all expects, unwraps and panics
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
    config.select_missing().unwrap_or_else(|e| panic!("Error: {}", e));
    let settings = settings::get_settings(&config).unwrap_or_else(|e| panic!("Error: {}", e));
    let state = state::init_state();
    if config.log_on_start.unwrap_or(false) {
        utils::start_new_log(&settings, &state);
    }

    let serial_thread = utils::start_thread(settings.clone(), &state, serial::connect_loop);
    let command_thread = utils::start_thread(settings.clone(), &state, commands::command_loop);

    let _ = serial_thread.join().map_err(|e| println!("Serial thread panicked: {:?}", e));
    let _ = command_thread.join().map_err(|e| println!("Command thread panicked: {:?}", e));
}

