// TODO: Implement set port command
//   TODO: Pause spew while setting port
// TODO: Support writing to serial port -- maybe use a separate thread for this?
// Note: RAII log file cleanup has been implemented using LogFile wrapper

use clap::Parser;

mod buffer;
mod commands;
mod error;
mod log;
mod serial;
mod settings;
mod state;
mod utils;
mod validation;

fn main() {
    let args = settings::Args::parse();
    
    if args.list {
        if let Err(e) = utils::list_ports() {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        return;
    }
    
    let (config, state) = match utils::initialize_app(args) {
        Ok((config, state)) => (config, state),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    
    if let Err(e) = run(config, state) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(mut config: settings::Config, state: state::State) -> error::Result<()> {
    if !config.disable_welcome.unwrap_or(false) {
        utils::print_welcome();
    }
    config.select_missing()?;
    let settings = settings::get_settings(&config)?;
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
    
    utils::cleanup_logs(&state);
    
    if let Err(e) = serial_result {
        eprintln!("Serial thread error: {e}");
    }
    if let Err(e) = command_result {
        eprintln!("Command thread error: {e}");
    }
    
    // utils::leave_alternate_screen()?;
    Ok(())
}
