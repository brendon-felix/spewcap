
// TODO: Expand interactive selection for config
//   TODO: Only ask for settings not set by config
//   TODO: Could probably remove Args after this
// TODO: Propagate timestamps flag using args and settings (possibly combine settings and state)
// TODO: Implement log at start
// TODO: Re-implement arguments (port, baud_rate, log folder)
// TODO: Re-implement wipe log command
// TODO: Re-implement save command
// TODO: Implement set port command
//   TODO: Pause spew while setting port


mod args;
mod settings;
mod state;
mod serial;
mod commands;
mod utils;
mod log;

use args::Args;
use settings::{Settings, Config};
use state::State;

use anyhow::{Result, Context, anyhow, bail};
use clap::Parser;
use dialoguer::Select;
use serialport5::available_ports;
use utils::clear_console;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use colored::Colorize;

fn main() -> Result<()> {
    clear_console();
    let args = Args::parse();
    let port = select_port()?;
    // if args.list {
    //     list_ports()?;
    //     return Ok(());
    // }
    // let log_file = if args.logging {
    //     Some(utils::create_log_file())
    // } else {
    //     None
    // };
    // let config = Config::load("config.toml");
    // let settings = Settings::new(config);
    let settings = Settings {
        port,
        ..Default::default()
    };
    let state = Arc::new(Mutex::new(State::default()));
    let (tx, rx) = mpsc::channel();
        
    let shared_state1 = Arc::clone(&state);
    let serial_thread = thread::spawn(move || {
        serial::connect_loop(settings, shared_state1);
        // Ok::<(), anyhow::Error>(())
    });
    let shared_state2 = Arc::clone(&state);
    let command_thread = thread::spawn(move || {
        commands::command_loop(shared_state2)?;
        tx.send(())?;
        // Ok(())
        Ok::<(), anyhow::Error>(())
    });

    rx.recv()?;
    let _ = serial_thread.join().map_err(|e| anyhow!("Thread panicked: {:?}", e))?;
    let _ = command_thread.join().map_err(|e| anyhow!("Thread panicked: {:?}", e))?;

    Ok(())
}

fn select_port() -> Result<String> {
    let ports = available_ports().context("Could not list ports")?;
    let port_names: Vec<&str> = ports.iter().map(|port| port.port_name.as_str()).collect();
    if ports.is_empty() {
        bail!("No ports found");
    }
    let selection = Select::new()
        .with_prompt("Select serial port")
        .items(&port_names)
        .interact()
        .context("Could not select port")?;
    let port = port_names[selection].to_string();
    Ok(port)
}

fn list_ports() -> Result<()> {
    println!("Connected serial ports:");
    let ports = available_ports().context("Could not list ports")?;
    if ports.len() == 0 {
        println!("{}", "No ports found!".bold().red());
        return Ok(());
    }
    for (index, port) in ports.into_iter().enumerate() {
        println!("{}. {}", index + 1, port.port_name);
    }
    // println!("");
    Ok(())
}