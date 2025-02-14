
// TODO: Re-implement wipe log command
// TODO: Re-implement log folder setting
// TODO: Update state logic to track if the port is connected
// TODO: Update state logic to quit more gracefully (change command)
// TODO: Implement save as... command
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

    // Spawn thread for serial connection
    let shared_state1 = state::shared_state(&state);
    let serial_thread = std::thread::spawn(move || {
        serial::connect_loop(settings, shared_state1);
    });

    // Spawn thread for command handler
    let shared_state2 = state::shared_state(&state);
    let command_thread = std::thread::spawn(move || {
        commands::command_loop(shared_state2);
    });

    let _ = serial_thread.join().map_err(|e| println!("Serial thread panicked: {:?}", e));
    let _ = command_thread.join().map_err(|e| println!("Command thread panicked: {:?}", e));
    println!("YAY!")
}
