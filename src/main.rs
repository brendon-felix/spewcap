

// TODO: Propagate timestamps flag using args and settings (possibly combine settings and state)
// TODO: Implement log at start
// TODO: Re-implement arguments (port, baud_rate, log folder)
// TODO: Re-implement wipe log command
// TODO: Re-implement save command
// TODO: Possibly re-implement configuration file
// TODO: Implement set port command


mod args;
mod settings;
mod state;
mod serial;
mod commands;
mod utils;
mod log;

use args::Args;
use settings::Settings;
use state::State;

use anyhow::{Result, Context, anyhow};
use clap::Parser;
use serialport5::available_ports;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use colored::Colorize;

fn main() -> Result<()> {
    let args = Args::parse();
    if args.list {
        list_ports()?;
        return Ok(());
    }
    // let log_file = if args.logging {
    //     Some(utils::create_log_file())
    // } else {
    //     None
    // };
    let settings = Settings::default();
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