// TODO: Remove temp log files after they are saved OR experiment with large String buffer
//   TODO: If still using file buffers, implement command to wipe the active file
// TODO: Implement set port command
//   TODO: Pause spew while setting port
// TODO: Support writing to serial port -- maybe use a separate thread for this?

use clap::Parser;

mod commands;
mod error;
mod log;
mod serial;
mod settings;
mod state;
mod utils;
mod validation;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let args = settings::Args::parse();
    
    if args.list {
        utils::list_ports()?;
        return Ok(());
    }
    
    // utils::enter_alternate_screen()?;
    let mut config = settings::get_config(args)?;
    if !config.disable_welcome.unwrap_or(false) {
        utils::print_welcome();
    }
    config.select_missing()?;
    let settings = settings::get_settings(&config)?;
    let state = state::init_state();
    if config.log_on_start.unwrap_or(false) {
        utils::start_new_log(&settings, &state)?;
    }

    let serial_thread = utils::start_thread(settings.clone(), &state, serial::connect_loop);
    let command_thread = utils::start_thread(settings.clone(), &state, commands::command_loop);

    let serial_result = serial_thread
        .join()
        .map_err(|e| error::SpewcapError::ThreadJoin(format!("Serial thread panicked: {:?}", e)))?;
    let command_result = command_thread
        .join()
        .map_err(|e| error::SpewcapError::ThreadJoin(format!("Command thread panicked: {:?}", e)))?;
    
    if let Err(e) = serial_result {
        eprintln!("Serial thread error: {}", e);
    }
    if let Err(e) = command_result {
        eprintln!("Command thread error: {}", e);
    }
    
    // utils::leave_alternate_screen()?;
    Ok(())
}
