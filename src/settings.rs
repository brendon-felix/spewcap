use clap::Parser;
use dialoguer::Select;
use serde::Deserialize;
use serialport5::{available_ports, SerialPortType};
use std::fs;
use std::path::PathBuf;
use toml;

use crate::utils;
use crate::error::{Result, SpewcapError};
use crate::validation;

#[derive(Clone, Debug)]
pub struct Settings {
    pub port: String,
    pub baud_rate: u32,
    pub timestamps: bool,
    pub log_folder: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Port name (eg. "COM3", "/dev/ttyUSB0")
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate
    #[arg(short, long)]
    baud_rate: Option<u32>,

    /// Prepend timestamps in log
    #[arg(short, long)]
    timestamps: bool,

    /// Prepend timestamps in log
    #[arg(short, long)]
    log_on_start: bool,

    /// Folder path to save logs
    #[arg(short = 'f', long)]
    log_folder: Option<String>,

    #[arg(long)]
    pub list: bool,
}

#[derive(Default, Deserialize, Debug)]
pub struct Config {
    port: Option<String>,
    baud_rate: Option<u32>,
    timestamps: Option<bool>,
    log_folder: Option<String>,
    pub log_on_start: Option<bool>,
    // pub clear_on_start: Option<bool>,
    pub disable_welcome: Option<bool>,
}
impl Config {
    fn load(file_path: PathBuf) -> Option<Self> {
        let toml_str = fs::read_to_string(file_path).ok()?;
        toml::from_str(&toml_str).ok()?
    }
    fn use_args(&mut self, args: Args) -> Result<()> {
        // Validate arguments before setting them
        if let Some(port) = &args.port {
            validation::validate_port_name(port)?;
            self.port = Some(port.clone());
        }
        
        if let Some(baud_rate) = args.baud_rate {
            validation::validate_baud_rate(baud_rate)?;
            self.baud_rate = Some(baud_rate);
        }
        
        if let Some(log_folder) = &args.log_folder {
            validation::validate_directory_path(log_folder)?;
            self.log_folder = Some(log_folder.clone());
        }
        
        self.timestamps = Some(args.timestamps);
        self.log_on_start = Some(args.log_on_start);
        Ok(())
    }
    pub fn select_missing(&mut self) -> Result<()> {
        if self.port.is_none() {
            self.port = Some(select_port()?);
        }
        if self.baud_rate.is_none() {
            self.baud_rate = Some(select_baud_rate()?);
        }
        Ok(())
    }
}

fn select_port() -> Result<String> {
    let ports = available_ports().map_err(|e| SpewcapError::SerialPort(e))?;
    if ports.is_empty() {
        return Err(SpewcapError::NoPortsFound);
    }
    let port_names: Vec<&str> = ports.iter().map(|port| port.port_name.as_str()).collect();
    let port_descriptions: Vec<String> = ports
        .iter()
        .map(|port| {
            let mut description = format!("{}", port.port_name);
            if let SerialPortType::UsbPort(info) = &port.port_type {
                description = format!(
                    "{} {}",
                    info.manufacturer.as_deref().unwrap_or(""),
                    info.product.as_deref().unwrap_or("")
                )
                .trim()
                .to_string();
            }
            description
        })
        .collect();
    let selection = Select::new()
        .with_prompt("Select serial port")
        .default(0) // default is the first option
        .items(&port_descriptions)
        .interact()
        .map_err(|e| SpewcapError::Dialog(format!("No serial port selected: {e}")))?;
    Ok(port_names[selection].to_string())
}

fn select_baud_rate() -> Result<u32> {
    let options = [
        4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
    ];
    let selection = Select::new()
        .with_prompt("Select baud rate")
        .default(5) // default is 115200 at index 5
        .items(&options)
        .interact()
        .map_err(|e| SpewcapError::Dialog(format!("No baud rate selected: {e}")))?;
    Ok(options[selection])
}

pub fn get_config(args: Args) -> Result<Config> {
    let curr_dir = utils::get_curr_directory();
    let path_in_curr_dir = curr_dir.join("spewcap_config.toml");
    let exe_dir = utils::get_exe_directory().unwrap_or(utils::get_curr_directory());
    let path_in_exe_dir = exe_dir.join("spewcap_config.toml");
    let mut config = Config::load(path_in_curr_dir)
        .unwrap_or(Config::load(path_in_exe_dir).unwrap_or(Config::default()));
    config.use_args(args)?;
    Ok(config)
}

pub fn get_settings(config: &Config) -> Result<Settings> {
    let port = config.port.clone().ok_or_else(|| SpewcapError::Settings("Could not set port".to_string()))?;
    let baud_rate = config.baud_rate.ok_or_else(|| SpewcapError::Settings("Could not set baud rate".to_string()))?;
    
    let validated_port = validation::validate_port_name(&port)?;
    let validated_baud_rate = validation::validate_baud_rate(baud_rate)?;
    
    let timestamps = config.timestamps.unwrap_or(false);
    let log_folder = match &config.log_folder {
        Some(folder_str) => {
            let validated_folder = validation::validate_directory_path(folder_str)?;
            Some(PathBuf::from(validated_folder))
        }
        None => None,
    };
    
    Ok(Settings {
        port: validated_port,
        baud_rate: validated_baud_rate,
        timestamps,
        log_folder,
    })
}
