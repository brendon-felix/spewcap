use clap::Parser;
use std::fs;
use toml;
// use serde::{Deserialize, Serialize};
use serde::Deserialize;
// use std::io::{self, Write};
// #[command(author, version, about, long_about = None)]
#[derive(Parser, Debug)]
struct Args {
    /// Port name (eg. COM23 OR /dev/ttyUSB0)
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate
    #[arg(short, long)]
    baud_rate: Option<u32>,

    /// Prepend timestamps in log
    #[arg(short, long)]
    timestamps: Option<bool>,

    /// Folder path to save logs
    #[arg(short, long)]
    log_folder: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    port: Option<String>,
    baud_rate: Option<u32>,
    timestamps: Option<bool>,
    log_folder: Option<String>,
    clear_on_start: Option<bool>,
}

#[derive(Clone)]
pub struct Settings {
    pub port: String,
    pub baud_rate: u32,
    pub timestamps: bool,
    pub log_folder: String,
    pub clear_on_start: bool,
}

fn get_config(filename: String) -> Option<Config> {
    let toml_str = fs::read_to_string(filename).ok()?;
    toml::from_str(&toml_str).ok()
}

pub fn get_settings() -> Settings {
    let args = Args::parse();
    if let Some(config) = get_config("config.toml".to_string()) {
        Settings {
            port: args.port.or(config.port).expect("No port specified!"),
            baud_rate: args.baud_rate.or(config.baud_rate).unwrap_or(115200),
            timestamps: args.timestamps.or(config.timestamps).unwrap_or(false),
            log_folder: args.log_folder.or(config.log_folder)
                .unwrap_or(r"C:\Users\felixb\OneDrive - HP Inc\Debugs\Springs".to_string()),
            clear_on_start: config.clear_on_start.unwrap_or(false),
        }
    } else {
        Settings {
            port: args.port.expect("No port specified!"),
            baud_rate: args.baud_rate.unwrap_or(115200),
            timestamps: args.timestamps.unwrap_or(false),
            log_folder: args.log_folder
                .unwrap_or(r"C:\Users\felixb\OneDrive - HP Inc\Debugs\Springs".to_string()),
            clear_on_start: false,
        }
    }
    
}

// pub fn set_config(filename: String) -> Result<()> {

// }