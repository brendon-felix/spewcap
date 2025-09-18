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
        self.apply_port_arg(args.port)?;
        self.apply_baud_rate_arg(args.baud_rate)?;
        self.apply_log_folder_arg(args.log_folder)?;
        self.apply_bool_args(args.timestamps, args.log_on_start);
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
    
    fn apply_port_arg(&mut self, port: Option<String>) -> Result<()> {
        if let Some(port) = port {
            validation::validate_port_name(&port)?;
            self.port = Some(port);
        }
        Ok(())
    }
    
    fn apply_baud_rate_arg(&mut self, baud_rate: Option<u32>) -> Result<()> {
        if let Some(baud_rate) = baud_rate {
            validation::validate_baud_rate(baud_rate)?;
            self.baud_rate = Some(baud_rate);
        }
        Ok(())
    }
    
    fn apply_log_folder_arg(&mut self, log_folder: Option<String>) -> Result<()> {
        if let Some(log_folder) = log_folder {
            validation::validate_directory_path(&log_folder)?;
            self.log_folder = Some(log_folder);
        }
        Ok(())
    }
    
    fn apply_bool_args(&mut self, timestamps: bool, log_on_start: bool) {
        self.timestamps = Some(timestamps);
        self.log_on_start = Some(log_on_start);
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
    let mut config = load_config_from_files()?;
    config.use_args(args)?;
    Ok(config)
}

fn load_config_from_files() -> Result<Config> {
    let config_paths = get_config_file_paths();
    
    for path in config_paths {
        if let Some(config) = Config::load(path) {
            return Ok(config);
        }
    }
    
    Ok(Config::default())
}

fn get_config_file_paths() -> Vec<PathBuf> {
    let curr_dir = utils::get_curr_directory();
    let exe_dir = utils::get_exe_directory().unwrap_or_else(|| utils::get_curr_directory());
    
    vec![
        curr_dir.join("spewcap_config.toml"),
        exe_dir.join("spewcap_config.toml"),
    ]
}

pub fn get_settings(config: &Config) -> Result<Settings> {
    let port = extract_and_validate_port(config)?;
    let baud_rate = extract_and_validate_baud_rate(config)?;
    let timestamps = config.timestamps.unwrap_or(false);
    let log_folder = extract_and_validate_log_folder(config)?;
    
    Ok(Settings {
        port,
        baud_rate,
        timestamps,
        log_folder,
    })
}

fn extract_and_validate_port(config: &Config) -> Result<String> {
    let port = config.port.clone()
        .ok_or_else(|| SpewcapError::Settings("Could not set port".to_string()))?;
    validation::validate_port_name(&port)
}

fn extract_and_validate_baud_rate(config: &Config) -> Result<u32> {
    let baud_rate = config.baud_rate
        .ok_or_else(|| SpewcapError::Settings("Could not set baud rate".to_string()))?;
    validation::validate_baud_rate(baud_rate)
}

fn extract_and_validate_log_folder(config: &Config) -> Result<Option<PathBuf>> {
    match &config.log_folder {
        Some(folder_str) => {
            let validated_folder = validation::validate_directory_path(folder_str)?;
            Ok(Some(PathBuf::from(validated_folder)))
        }
        None => Ok(None),
    }
}
